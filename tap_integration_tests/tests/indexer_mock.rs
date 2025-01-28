// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use alloy::dyn_abi::Eip712Domain;
use anyhow::{Error, Result};
use jsonrpsee::{
    core::async_trait,
    http_client::{HttpClient, HttpClientBuilder},
    proc_macros::rpc,
    rpc_params,
    server::{ServerBuilder, ServerHandle},
};
use jsonrpsee_core::client::ClientT;
use tap_aggregator::jsonrpsee_helpers;
use tap_core::{
    manager::{
        adapters::{RavRead, RavStore, ReceiptRead, ReceiptStore, SignatureChecker},
        Manager,
    },
    receipt::{checks::CheckList, Context},
};
use tap_graph::{ReceiptAggregateVoucher, SignedRav, SignedReceipt};
/// Rpc trait represents a JSON-RPC server that has a single async method `request`.
/// This method is designed to handle incoming JSON-RPC requests.
#[rpc(server)]
pub trait Rpc {
    // This async method is designed to handle incoming JSON-RPC requests.
    #[method(name = "request")]
    async fn request(
        &self,
        receipt: SignedReceipt, // Signed receipt associated with the request
    ) -> Result<(), jsonrpsee::types::ErrorObjectOwned>; // The result of the request, a JSON-RPC error if it fails
}

/// RpcManager is a struct that implements the `Rpc` trait and it represents a JSON-RPC server manager.
/// It includes a manager, initial_checks, receipt_count, threshold and aggregator_client.
/// Manager holds an Arc to an instance of a generic `Manager` object which is shared and can be accessed by multiple threads.
/// initial_checks is a list of checks that needs to be performed for every incoming request.
/// receipt_count is a thread-safe counter that increments with each receipt verified and stored.
/// threshold is a limit to which receipt_count can increment, after reaching which RAV request is triggered.
/// aggregator_client is an HTTP client used for making JSON-RPC requests to another server.
pub struct RpcManager<E> {
    manager: Arc<Manager<E, SignedReceipt, ReceiptAggregateVoucher>>, // Manager object reference counted with an Arc
    receipt_count: Arc<AtomicU64>, // Thread-safe atomic counter for receipts
    threshold: u64,                // The count at which a RAV request will be triggered
    aggregator_client: (HttpClient, String), // HTTP client for sending requests to the aggregator server
}

/// Implementation for `RpcManager`, includes the constructor and the `request` method.
/// Constructor initializes a new instance of `RpcManager`.
/// `request` method handles incoming JSON-RPC requests and it verifies and stores the receipt from the request.
impl<E> RpcManager<E>
where
    E: Clone,
{
    pub fn new(
        domain_separator: Eip712Domain,
        context: E,
        required_checks: CheckList<SignedReceipt>,
        threshold: u64,
        aggregate_server_address: String,
        aggregate_server_api_version: String,
    ) -> Result<Self> {
        Ok(Self {
            manager: Arc::new(Manager::<E, SignedReceipt, ReceiptAggregateVoucher>::new(
                domain_separator,
                context,
                required_checks,
            )),
            receipt_count: Arc::new(AtomicU64::new(0)),
            threshold,
            aggregator_client: (
                HttpClientBuilder::default().build(aggregate_server_address)?,
                aggregate_server_api_version,
            ),
        })
    }
}

#[async_trait]
impl<E> RpcServer for RpcManager<E>
where
    E: ReceiptStore<SignedReceipt>
        + ReceiptRead<SignedReceipt>
        + RavStore<ReceiptAggregateVoucher>
        + RavRead<ReceiptAggregateVoucher>
        + SignatureChecker
        + Send
        + Sync
        + 'static,
{
    async fn request(
        &self,
        receipt: SignedReceipt,
    ) -> Result<(), jsonrpsee::types::ErrorObjectOwned> {
        let verify_result = match self
            .manager
            .verify_and_store_receipt(&Context::new(), receipt)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(to_rpc_error(
                Box::new(e),
                "Failed to verify and store receipt",
            )),
        };

        // Increment the receipt count
        self.receipt_count.fetch_add(1, Ordering::Relaxed);
        let rav_request_valid = if self.receipt_count.load(Ordering::SeqCst) >= self.threshold {
            // Reset the counter after reaching the threshold
            self.receipt_count.store(0, Ordering::SeqCst);

            // Create the aggregate_receipts request params
            let time_stamp_buffer = 0;
            match request_rav(
                &self.manager,
                time_stamp_buffer,
                &self.aggregator_client,
                self.threshold as usize,
            )
            .await
            {
                Ok(_) => Ok(()),
                Err(e) => Err(to_rpc_error(e.into(), "Failed to request rav")),
            }
        } else {
            Ok(())
        };

        // Combine the results
        match (verify_result, rav_request_valid) {
            (Ok(_), Ok(_)) => Ok(()),
            (Err(e), _) | (_, Err(e)) => Err(e),
        }
    }
}

/// run_server function initializes and starts a JSON-RPC server that handles incoming requests.
pub async fn run_server<E>(
    port: u16,                                 // Port on which the server will listen
    domain_separator: Eip712Domain,            // EIP712 domain separator
    context: E,                                // context instance
    required_checks: CheckList<SignedReceipt>, // Vector of required checks to be performed on each request
    threshold: u64,                            // The count at which a RAV request will be triggered
    aggregate_server_address: String,          // Address of the aggregator server
    aggregate_server_api_version: String,      // API version of the aggregator server
) -> Result<(ServerHandle, std::net::SocketAddr)>
where
    E: ReceiptStore<SignedReceipt>
        + ReceiptRead<SignedReceipt>
        + RavStore<ReceiptAggregateVoucher>
        + RavRead<ReceiptAggregateVoucher>
        + SignatureChecker
        + Clone
        + Send
        + Sync
        + 'static,
{
    // Setting up the JSON RPC server
    println!("Starting server...");
    let server = ServerBuilder::new()
        .http_only()
        .build(format!("127.0.0.1:{}", port))
        .await?;
    let addr = server.local_addr()?;
    println!("Listening on: {}", addr);
    let rpc_manager = RpcManager::new(
        domain_separator,
        context,
        required_checks,
        threshold,
        aggregate_server_address,
        aggregate_server_api_version,
    )?;

    let handle = server.start(rpc_manager.into_rpc());
    Ok((handle, addr))
}

// request_rav function creates a request for aggregate receipts (RAV), sends it to another server and verifies the result.
async fn request_rav<E>(
    manager: &Arc<Manager<E, SignedReceipt, ReceiptAggregateVoucher>>,
    time_stamp_buffer: u64, // Buffer for timestamping, see tap_core for details
    aggregator_client: &(HttpClient, String), // HttpClient for making requests to the tap_aggregator server
    threshold: usize,
) -> Result<()>
where
    E: ReceiptRead<SignedReceipt>
        + RavRead<ReceiptAggregateVoucher>
        + RavStore<ReceiptAggregateVoucher>
        + SignatureChecker,
{
    // Create the aggregate_receipts request params
    let rav_request = manager
        .create_rav_request(&Context::new(), time_stamp_buffer, None)
        .await?;

    // To-do: Need to add previous RAV, when tap_manager supports replacing receipts
    let params = rpc_params!(
        &aggregator_client.1,
        &rav_request
            .valid_receipts
            .iter()
            .map(|receipt| receipt.signed_receipt())
            .collect::<Vec<_>>(),
        rav_request.previous_rav
    );

    // Call the aggregate_receipts method on the other server
    let remote_rav_result: jsonrpsee_helpers::JsonRpcResponse<SignedRav> = aggregator_client
        .0
        .request("aggregate_receipts", params)
        .await?;
    manager
        .verify_and_store_rav(rav_request.expected_rav?, remote_rav_result.data)
        .await?;

    // For these tests, we expect every receipt to be valid, i.e. there should be no invalid receipts, nor any missing receipts (less than the expected threshold).
    // If there is throw an error.
    match rav_request.invalid_receipts.is_empty() && (rav_request.valid_receipts.len() == threshold)
    {
        true => Ok(()),
        false => Err(Error::msg("Invalid receipts found")),
    }?;
    Ok(())
}

fn to_rpc_error(e: Box<dyn std::error::Error>, msg: &str) -> jsonrpsee::types::ErrorObjectOwned {
    jsonrpsee::types::ErrorObject::owned(-32000, format!("{} - {}", e, msg), None::<()>)
}
