// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashSet, str::FromStr};

use alloy::{dyn_abi::Eip712Domain, primitives::Address, signers::local::PrivateKeySigner};
use anyhow::Result;
use axum::{error_handling::HandleError, routing::post_service, BoxError, Router};
use hyper::StatusCode;
use jsonrpsee::{
    proc_macros::rpc,
    server::{ServerBuilder, ServerHandle, TowerService},
};
use lazy_static::lazy_static;
use log::info;
use prometheus::{register_counter, register_int_counter, Counter, IntCounter};
use tap_core::signed_message::EIP712SignedMessage;
use tap_graph::{Receipt, ReceiptAggregateVoucher, SignedReceipt};
use tokio::{net::TcpListener, signal, task::JoinHandle};
use tonic::{codec::CompressionEncoding, service::Routes, Request, Response, Status};
use tower::{layer::util::Identity, make::Shared};

use crate::{
    aggregator::check_and_aggregate_receipts,
    api_versioning::{
        tap_rpc_api_versions_info, TapRpcApiVersion, TapRpcApiVersionsInfo,
        TAP_RPC_API_VERSIONS_DEPRECATED,
    },
    error_codes::{JsonRpcErrorCode, JsonRpcWarningCode},
    grpc::{
        tap_aggregator_server::{TapAggregator, TapAggregatorServer},
        RavRequest, RavResponse,
    },
    jsonrpsee_helpers::{JsonRpcError, JsonRpcResponse, JsonRpcResult, JsonRpcWarning},
};

// Register the metrics into the global metrics registry.
lazy_static! {
    static ref AGGREGATION_SUCCESS_COUNTER: IntCounter = register_int_counter!(
        "aggregation_success_count",
        "Number of successful receipt aggregation requests."
    )
    .unwrap();
    static ref AGGREGATION_FAILURE_COUNTER: IntCounter = register_int_counter!(
        "aggregation_failure_count",
        "Number of failed receipt aggregation requests (for any reason)."
    )
    .unwrap();
    static ref DEPRECATION_WARNING_COUNT: IntCounter = register_int_counter!(
        "deprecation_warning_count",
        "Number of deprecation warnings sent to clients."
    )
    .unwrap();
    static ref VERSION_ERROR_COUNT: IntCounter = register_int_counter!(
        "version_error_count",
        "Number of API version errors sent to clients."
    )
    .unwrap();
    static ref TOTAL_AGGREGATED_RECEIPTS: IntCounter = register_int_counter!(
        "total_aggregated_receipts",
        "Total number of receipts successfully aggregated."
    )
    .unwrap();
// Using float for the GRT value because it can somewhat easily exceed the maximum value of int64.
    static ref TOTAL_GRT_AGGREGATED: Counter = register_counter!(
        "total_aggregated_grt",
        "Total successfully aggregated GRT value (wei)."
    )
    .unwrap();
}

/// Generates the `RpcServer` trait that is used to define the JSON-RPC API.
///
/// Note that because of the way the `rpc` macro works, we cannot document the RpcServer trait here.
/// (So even this very docstring will not appear in the generated documentation...)
/// As a result, we document the JSON-RPC API in the `tap_aggregator/README.md` file.
/// Do not forget to update the documentation there if you make any changes to the JSON-RPC API.
#[rpc(server)]
pub trait Rpc {
    /// Returns the versions of the TAP JSON-RPC API implemented by this server.
    #[method(name = "api_versions")]
    fn api_versions(&self) -> JsonRpcResult<TapRpcApiVersionsInfo>;

    /// Aggregates the given receipts into a receipt aggregate voucher.
    /// Returns an error if the user expected API version is not supported.
    #[method(name = "aggregate_receipts")]
    fn aggregate_receipts(
        &self,
        api_version: String,
        receipts: Vec<EIP712SignedMessage<Receipt>>,
        previous_rav: Option<EIP712SignedMessage<ReceiptAggregateVoucher>>,
    ) -> JsonRpcResult<EIP712SignedMessage<ReceiptAggregateVoucher>>;
}

#[derive(Clone)]
struct RpcImpl {
    wallet: PrivateKeySigner,
    accepted_addresses: HashSet<Address>,
    domain_separator: Eip712Domain,
}

/// Helper method that checks if the given API version is supported.
/// Returns an error if the API version is not supported.
fn parse_api_version(api_version: &str) -> Result<TapRpcApiVersion, JsonRpcError> {
    TapRpcApiVersion::from_str(api_version).map_err(|_| {
        jsonrpsee::types::ErrorObject::owned(
            JsonRpcErrorCode::InvalidVersion as i32,
            format!("Unsupported API version: \"{}\".", api_version),
            Some(tap_rpc_api_versions_info()),
        )
    })
}

/// Helper method that checks if the given API version has a deprecation warning.
/// Returns a warning if the API version is deprecated.
fn check_api_version_deprecation(api_version: &TapRpcApiVersion) -> Option<JsonRpcWarning> {
    if TAP_RPC_API_VERSIONS_DEPRECATED.contains(api_version) {
        Some(JsonRpcWarning::new(
            JsonRpcWarningCode::DeprecatedVersion as i32,
            format!(
                "The API version {} will be deprecated. \
                Please check https://github.com/semiotic-ai/timeline_aggregation_protocol for more information.",
                api_version
            ),
            Some(tap_rpc_api_versions_info()),
        ))
    } else {
        None
    }
}

fn aggregate_receipts_(
    api_version: String,
    wallet: &PrivateKeySigner,
    accepted_addresses: &HashSet<Address>,
    domain_separator: &Eip712Domain,
    receipts: Vec<EIP712SignedMessage<Receipt>>,
    previous_rav: Option<EIP712SignedMessage<ReceiptAggregateVoucher>>,
) -> JsonRpcResult<EIP712SignedMessage<ReceiptAggregateVoucher>> {
    // Return an error if the API version is not supported.
    let api_version = match parse_api_version(api_version.as_str()) {
        Ok(v) => v,
        Err(e) => {
            VERSION_ERROR_COUNT.inc();
            return Err(e);
        }
    };

    // Add a warning if the API version is to be deprecated.
    let mut warnings: Vec<JsonRpcWarning> = Vec::new();
    if let Some(w) = check_api_version_deprecation(&api_version) {
        warnings.push(w);
        DEPRECATION_WARNING_COUNT.inc();
    }

    let res = match api_version {
        TapRpcApiVersion::V0_0 => check_and_aggregate_receipts(
            domain_separator,
            &receipts,
            previous_rav,
            wallet,
            accepted_addresses,
        ),
    };

    // Handle aggregation error
    match res {
        Ok(res) => Ok(JsonRpcResponse::warn(res, warnings)),
        Err(e) => Err(jsonrpsee::types::ErrorObject::owned(
            JsonRpcErrorCode::Aggregation as i32,
            e.to_string(),
            None::<()>,
        )),
    }
}

#[tonic::async_trait]
impl TapAggregator for RpcImpl {
    async fn aggregate_receipts(
        &self,
        request: Request<RavRequest>,
    ) -> Result<Response<RavResponse>, Status> {
        let rav_request = request.into_inner();
        let receipts: Vec<SignedReceipt> = rav_request
            .receipts
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<_, _>>()
            .map_err(|_| Status::invalid_argument("Error while getting list of signed_receipts"))?;

        let previous_rav = rav_request
            .previous_rav
            .map(TryFrom::try_from)
            .transpose()
            .map_err(|_| Status::invalid_argument("Error while getting previous rav"))?;

        let receipts_grt: u128 = receipts.iter().map(|r| r.message.value).sum();
        let receipts_count: u64 = receipts.len() as u64;

        match check_and_aggregate_receipts(
            &self.domain_separator,
            receipts.as_slice(),
            previous_rav,
            &self.wallet,
            &self.accepted_addresses,
        ) {
            Ok(res) => {
                TOTAL_GRT_AGGREGATED.inc_by(receipts_grt as f64);
                TOTAL_AGGREGATED_RECEIPTS.inc_by(receipts_count);
                AGGREGATION_SUCCESS_COUNTER.inc();

                let response = RavResponse {
                    rav: Some(res.into()),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                AGGREGATION_FAILURE_COUNTER.inc();
                Err(Status::failed_precondition(e.to_string()))
            }
        }
    }
}

impl RpcServer for RpcImpl {
    fn api_versions(&self) -> JsonRpcResult<TapRpcApiVersionsInfo> {
        Ok(JsonRpcResponse::ok(tap_rpc_api_versions_info()))
    }

    fn aggregate_receipts(
        &self,
        api_version: String,
        receipts: Vec<EIP712SignedMessage<Receipt>>,
        previous_rav: Option<EIP712SignedMessage<ReceiptAggregateVoucher>>,
    ) -> JsonRpcResult<EIP712SignedMessage<ReceiptAggregateVoucher>> {
        // Values for Prometheus metrics
        let receipts_grt: u128 = receipts.iter().map(|r| r.message.value).sum();
        let receipts_count: u64 = receipts.len() as u64;

        match aggregate_receipts_(
            api_version,
            &self.wallet,
            &self.accepted_addresses,
            &self.domain_separator,
            receipts,
            previous_rav,
        ) {
            Ok(res) => {
                TOTAL_GRT_AGGREGATED.inc_by(receipts_grt as f64);
                TOTAL_AGGREGATED_RECEIPTS.inc_by(receipts_count);
                AGGREGATION_SUCCESS_COUNTER.inc();
                Ok(res)
            }
            Err(e) => {
                AGGREGATION_FAILURE_COUNTER.inc();
                Err(e)
            }
        }
    }
}

pub async fn run_server(
    port: u16,
    wallet: PrivateKeySigner,
    accepted_addresses: HashSet<Address>,
    domain_separator: Eip712Domain,
    max_request_body_size: u32,
    max_response_body_size: u32,
    max_concurrent_connections: u32,
) -> Result<(JoinHandle<()>, std::net::SocketAddr)> {
    // Setting up the JSON RPC server
    let rpc_impl = RpcImpl {
        wallet,
        accepted_addresses,
        domain_separator,
    };
    let (json_rpc_service, _) = create_json_rpc_service(
        rpc_impl.clone(),
        max_request_body_size,
        max_response_body_size,
        max_concurrent_connections,
    )?;

    async fn handle_anyhow_error(err: BoxError) -> (StatusCode, String) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )
    }
    let json_rpc_router = Router::new().route_service(
        "/",
        HandleError::new(post_service(json_rpc_service), handle_anyhow_error),
    );

    let grpc_service = create_grpc_service(rpc_impl)?;

    let service = tower::steer::Steer::new(
        [json_rpc_router, grpc_service.into_axum_router()],
        |req: &hyper::Request<_>, _services: &[_]| {
            if req
                .headers()
                .get(hyper::header::CONTENT_TYPE)
                .map(|content_type| content_type.as_bytes())
                .filter(|content_type| content_type.starts_with(b"application/grpc"))
                .is_some()
            {
                // route to the gRPC service (second service element) when the
                // header is set
                1
            } else {
                // otherwise route to the REST service
                0
            }
        },
    );

    // Create a `TcpListener` using tokio.
    let listener = TcpListener::bind(&format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind to tap-aggregator port");

    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, Shared::new(service))
            .with_graceful_shutdown(shutdown_handler())
            .await
        {
            log::error!("Tap Aggregator error: {e}");
        }
    });

    Ok((handle, addr))
}

/// Graceful shutdown handler
async fn shutdown_handler() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Signal received, starting graceful shutdown");
}

fn create_grpc_service(rpc_impl: RpcImpl) -> Result<Routes> {
    let grpc_service = Routes::new(
        TapAggregatorServer::new(rpc_impl).accept_compressed(CompressionEncoding::Zstd),
    )
    .prepare();

    Ok(grpc_service)
}

fn create_json_rpc_service(
    rpc_impl: RpcImpl,
    max_request_body_size: u32,
    max_response_body_size: u32,
    max_concurrent_connections: u32,
) -> Result<(TowerService<Identity, Identity>, ServerHandle)> {
    let service_builder = ServerBuilder::new()
        .max_request_body_size(max_request_body_size)
        .max_response_body_size(max_response_body_size)
        .max_connections(max_concurrent_connections)
        .http_only()
        .to_service_builder();
    use jsonrpsee::server::stop_channel;
    let (stop_handle, server_handle) = stop_channel();
    let handle = service_builder.build(rpc_impl.into_rpc(), stop_handle);
    Ok((handle, server_handle))
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
mod tests {
    use std::{collections::HashSet, str::FromStr};

    use alloy::{dyn_abi::Eip712Domain, primitives::Address, signers::local::PrivateKeySigner};
    use jsonrpsee::{core::client::ClientT, http_client::HttpClientBuilder, rpc_params};
    use rand::{prelude::*, seq::SliceRandom};
    use rstest::*;
    use tap_core::{signed_message::EIP712SignedMessage, tap_eip712_domain};
    use tap_graph::{Receipt, ReceiptAggregateVoucher};

    use crate::server;

    #[derive(Clone)]
    struct Keys {
        wallet: PrivateKeySigner,
        address: Address,
    }

    fn keys() -> Keys {
        let wallet = PrivateKeySigner::random();
        let address = wallet.address();
        Keys { wallet, address }
    }

    #[fixture]
    fn allocation_ids() -> Vec<Address> {
        vec![
            Address::from_str("0xabababababababababababababababababababab").unwrap(),
            Address::from_str("0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddead").unwrap(),
            Address::from_str("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef").unwrap(),
            Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
        ]
    }

    #[fixture]
    fn domain_separator() -> Eip712Domain {
        tap_eip712_domain(1, Address::from([0x11u8; 20]))
    }

    #[fixture]
    fn http_request_size_limit() -> u32 {
        100 * 1024
    }

    #[fixture]
    fn http_response_size_limit() -> u32 {
        100 * 1024
    }

    #[fixture]
    fn http_max_concurrent_connections() -> u32 {
        1
    }

    #[rstest]
    #[tokio::test]
    async fn protocol_version(
        domain_separator: Eip712Domain,
        http_request_size_limit: u32,
        http_response_size_limit: u32,
        http_max_concurrent_connections: u32,
    ) {
        // The keys that will be used to sign the new RAVs
        let keys_main = keys();

        // Start the JSON-RPC server.
        let (handle, local_addr) = server::run_server(
            0,
            keys_main.wallet,
            HashSet::from([keys_main.address]),
            domain_separator,
            http_request_size_limit,
            http_response_size_limit,
            http_max_concurrent_connections,
        )
        .await
        .unwrap();

        // Start the JSON-RPC client.
        let client = HttpClientBuilder::default()
            .build(format!("http://127.0.0.1:{}", local_addr.port()))
            .unwrap();
        let _: server::JsonRpcResponse<server::TapRpcApiVersionsInfo> = client
            .request("api_versions", rpc_params!(None::<()>))
            .await
            .unwrap();

        handle.abort();
    }

    #[rstest]
    #[case::basic_rav_test (vec![45,56,34,23])]
    #[case::rav_from_zero_valued_receipts (vec![0,0,0,0])]
    #[tokio::test]
    async fn signed_rav_is_valid_with_no_previous_rav(
        domain_separator: Eip712Domain,
        http_request_size_limit: u32,
        http_response_size_limit: u32,
        http_max_concurrent_connections: u32,
        allocation_ids: Vec<Address>,
        #[case] values: Vec<u128>,
        #[values("0.0")] api_version: &str,
        #[values(0, 1, 2)] random_seed: u64,
    ) {
        // The keys that will be used to sign the new RAVs
        let keys_main = keys();
        // Extra keys to test the server's ability to accept multiple signers as input
        let keys_0 = keys();
        let keys_1 = keys();
        // Vector of all wallets to make it easier to select one randomly
        let all_wallets = vec![keys_main.clone(), keys_0.clone(), keys_1.clone()];
        // PRNG for selecting a random wallet
        let mut rng = StdRng::seed_from_u64(random_seed);

        // Start the JSON-RPC server.
        let (handle, local_addr) = server::run_server(
            0,
            keys_main.wallet.clone(),
            HashSet::from([keys_main.address, keys_0.address, keys_1.address]),
            domain_separator.clone(),
            http_request_size_limit,
            http_response_size_limit,
            http_max_concurrent_connections,
        )
        .await
        .unwrap();

        // Start the JSON-RPC client.
        let client = HttpClientBuilder::default()
            .build(format!("http://127.0.0.1:{}", local_addr.port()))
            .unwrap();

        // Create receipts
        let mut receipts = Vec::new();
        for value in values {
            receipts.push(
                EIP712SignedMessage::new(
                    &domain_separator,
                    Receipt::new(allocation_ids[0], value).unwrap(),
                    &all_wallets.choose(&mut rng).unwrap().wallet,
                )
                .unwrap(),
            );
        }

        // Skipping receipts validation in this test, aggregate_receipts assumes receipts are valid.
        // Create RAV through the JSON-RPC server.
        let res: server::JsonRpcResponse<EIP712SignedMessage<ReceiptAggregateVoucher>> = client
            .request(
                "aggregate_receipts",
                rpc_params!(api_version, &receipts, None::<()>),
            )
            .await
            .unwrap();

        let remote_rav = res.data;

        let local_rav =
            ReceiptAggregateVoucher::aggregate_receipts(allocation_ids[0], &receipts, None)
                .unwrap();

        assert!(remote_rav.message.allocationId == local_rav.allocationId);
        assert!(remote_rav.message.timestampNs == local_rav.timestampNs);
        assert!(remote_rav.message.valueAggregate == local_rav.valueAggregate);

        assert!(remote_rav.recover_signer(&domain_separator).unwrap() == keys_main.address);

        handle.abort();
    }

    #[rstest]
    #[case::basic_rav_test (vec![45,56,34,23])]
    #[case::rav_from_zero_valued_receipts (vec![0,0,0,0])]
    #[tokio::test]
    async fn signed_rav_is_valid_with_previous_rav(
        domain_separator: Eip712Domain,
        http_request_size_limit: u32,
        http_response_size_limit: u32,
        http_max_concurrent_connections: u32,
        allocation_ids: Vec<Address>,
        #[case] values: Vec<u128>,
        #[values("0.0")] api_version: &str,
        #[values(0, 1, 2, 3, 4)] random_seed: u64,
    ) {
        // The keys that will be used to sign the new RAVs
        let keys_main = keys();
        // Extra keys to test the server's ability to accept multiple signers as input
        let keys_0 = keys();
        let keys_1 = keys();
        // Vector of all wallets to make it easier to select one randomly
        let all_wallets = vec![keys_main.clone(), keys_0.clone(), keys_1.clone()];
        // PRNG for selecting a random wallet
        let mut rng = StdRng::seed_from_u64(random_seed);

        // Start the JSON-RPC server.
        let (handle, local_addr) = server::run_server(
            0,
            keys_main.wallet.clone(),
            HashSet::from([keys_main.address, keys_0.address, keys_1.address]),
            domain_separator.clone(),
            http_request_size_limit,
            http_response_size_limit,
            http_max_concurrent_connections,
        )
        .await
        .unwrap();

        // Start the JSON-RPC client.
        let client = HttpClientBuilder::default()
            .build(format!("http://127.0.0.1:{}", local_addr.port()))
            .unwrap();

        // Create receipts
        let mut receipts = Vec::new();
        for value in values {
            receipts.push(
                EIP712SignedMessage::new(
                    &domain_separator,
                    Receipt::new(allocation_ids[0], value).unwrap(),
                    &all_wallets.choose(&mut rng).unwrap().wallet,
                )
                .unwrap(),
            );
        }

        // Create previous RAV from first half of receipts locally
        let prev_rav = ReceiptAggregateVoucher::aggregate_receipts(
            allocation_ids[0],
            &receipts[0..receipts.len() / 2],
            None,
        )
        .unwrap();
        let signed_prev_rav = EIP712SignedMessage::new(
            &domain_separator,
            prev_rav,
            &all_wallets.choose(&mut rng).unwrap().wallet,
        )
        .unwrap();

        // Create new RAV from last half of receipts and prev_rav through the JSON-RPC server
        let res: server::JsonRpcResponse<EIP712SignedMessage<ReceiptAggregateVoucher>> = client
            .request(
                "aggregate_receipts",
                rpc_params!(
                    api_version,
                    &receipts[receipts.len() / 2..receipts.len()],
                    Some(signed_prev_rav)
                ),
            )
            .await
            .unwrap();

        let rav = res.data;

        assert!(rav.recover_signer(&domain_separator).unwrap() == keys_main.address);

        handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn invalid_api_version(
        domain_separator: Eip712Domain,
        http_request_size_limit: u32,
        http_response_size_limit: u32,
        http_max_concurrent_connections: u32,
        allocation_ids: Vec<Address>,
    ) {
        // The keys that will be used to sign the new RAVs
        let keys_main = keys();

        // Start the JSON-RPC server.
        let (handle, local_addr) = server::run_server(
            0,
            keys_main.wallet.clone(),
            HashSet::from([keys_main.address]),
            domain_separator.clone(),
            http_request_size_limit,
            http_response_size_limit,
            http_max_concurrent_connections,
        )
        .await
        .unwrap();

        // Start the JSON-RPC client.
        let client = HttpClientBuilder::default()
            .build(format!("http://127.0.0.1:{}", local_addr.port()))
            .unwrap();

        // Create receipts
        let receipts = vec![EIP712SignedMessage::new(
            &domain_separator,
            Receipt::new(allocation_ids[0], 42).unwrap(),
            &keys_main.wallet,
        )
        .unwrap()];

        // Skipping receipts validation in this test, aggregate_receipts assumes receipts are valid.
        // Create RAV through the JSON-RPC server.
        let res: Result<
            server::JsonRpcResponse<EIP712SignedMessage<ReceiptAggregateVoucher>>,
            jsonrpsee::core::ClientError,
        > = client
            .request(
                "aggregate_receipts",
                rpc_params!("invalid version string", &receipts, None::<()>),
            )
            .await;

        assert!(res.is_err());

        // Make sure the JSON-RPC error is "invalid version"
        assert!(res
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("Unsupported API version"));

        // Check the API versions returned by the server
        match res.expect_err("Expected an error") {
            jsonrpsee::core::ClientError::Call(err) => {
                let versions: server::TapRpcApiVersionsInfo =
                    serde_json::from_str(err.data().unwrap().get()).unwrap();
                assert!(versions
                    .versions_supported
                    .contains(&server::TapRpcApiVersion::V0_0));
            }
            _ => panic!("Expected data in error"),
        }

        handle.abort();
    }

    /// Test that the server returns an error when the request size exceeds the limit.
    /// The server should return HTTP 413 (Request Entity Too Large).
    /// In this test, the request size limit is set to 100 kB, and we are expecting
    /// that to fit about 250 receipts. We also test with 300 receipts, which should
    /// exceed the limit.
    /// We conclude that a limit of 10MB should fit about 25k receipts, and thus
    /// the TAP spec will require that the aggregator supports up to 15k receipts
    /// per aggregation request as a safe limit.
    #[rstest]
    #[tokio::test]
    async fn request_size_limit(
        domain_separator: Eip712Domain,
        http_response_size_limit: u32,
        http_max_concurrent_connections: u32,
        allocation_ids: Vec<Address>,
        #[values("0.0")] api_version: &str,
    ) {
        // The keys that will be used to sign the new RAVs
        let keys_main = keys();

        // Set the request byte size limit to a value that easily triggers the HTTP 413
        // error.
        let http_request_size_limit = 100 * 1024;

        // Number of receipts that is just above the number that would fit within the
        // request size limit. This value is hard-coded here because it supports the
        // maximum number of receipts per aggregate value we wrote in the spec / docs.
        let number_of_receipts_to_exceed_limit = 300;

        // Start the JSON-RPC server.
        let (handle, local_addr) = server::run_server(
            0,
            keys_main.wallet.clone(),
            HashSet::from([keys_main.address]),
            domain_separator.clone(),
            http_request_size_limit,
            http_response_size_limit,
            http_max_concurrent_connections,
        )
        .await
        .unwrap();

        // Start the JSON-RPC client.
        let client = HttpClientBuilder::default()
            .build(format!("http://127.0.0.1:{}", local_addr.port()))
            .unwrap();

        // Create receipts
        let mut receipts = Vec::new();
        for _ in 1..number_of_receipts_to_exceed_limit {
            receipts.push(
                EIP712SignedMessage::new(
                    &domain_separator,
                    Receipt::new(allocation_ids[0], u128::MAX / 1000).unwrap(),
                    &keys_main.wallet,
                )
                .unwrap(),
            );
        }

        // Skipping receipts validation in this test, aggregate_receipts assumes receipts are valid.
        // Create RAV through the JSON-RPC server.
        // Test with a number of receipts that stays within request size limit
        let res: Result<
            server::JsonRpcResponse<EIP712SignedMessage<ReceiptAggregateVoucher>>,
            jsonrpsee::core::ClientError,
        > = client
            .request(
                "aggregate_receipts",
                rpc_params!(
                    api_version,
                    &receipts[..number_of_receipts_to_exceed_limit - 50],
                    None::<()>
                ),
            )
            .await;
        assert!(res.is_ok());

        // Create RAV through the JSON-RPC server.
        // Test with all receipts to exceed request size limit
        let res: Result<
            server::JsonRpcResponse<EIP712SignedMessage<ReceiptAggregateVoucher>>,
            jsonrpsee::core::ClientError,
        > = client
            .request(
                "aggregate_receipts",
                rpc_params!(api_version, &receipts, None::<()>),
            )
            .await;

        assert!(res.is_err());
        // Make sure the error is a HTTP 413 Content Too Large
        assert!(res.unwrap_err().to_string().contains("413"));

        handle.abort();
    }
}
