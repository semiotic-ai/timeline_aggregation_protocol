// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashSet, str::FromStr};

use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use jsonrpsee::{core::client::ClientT, http_client::HttpClientBuilder, rpc_params};
use tap_aggregator::{
    grpc::v1::{tap_aggregator_client::TapAggregatorClient, RavRequest},
    jsonrpsee_helpers::JsonRpcResponse,
    server,
};
use tap_core::{signed_message::Eip712SignedMessage, tap_eip712_domain};
use tap_graph::{Receipt, ReceiptAggregateVoucher};
use tonic::codec::CompressionEncoding;

#[tokio::test]
async fn aggregation_test() {
    let domain_separator = tap_eip712_domain(1, Address::ZERO);

    let wallet = PrivateKeySigner::random();

    let max_request_body_size = 1024 * 100;
    let max_response_body_size = 1024 * 100;
    let max_concurrent_connections = 1;

    let accepted_addresses = HashSet::from([wallet.address()]);

    let (join_handle, local_addr) = server::run_server(
        0,
        wallet.clone(),
        accepted_addresses,
        domain_separator.clone(),
        max_request_body_size,
        max_response_body_size,
        max_concurrent_connections,
        None,
    )
    .await
    .unwrap();

    let endpoint = format!("http://127.0.0.1:{}", local_addr.port());

    let mut client = TapAggregatorClient::connect(endpoint.clone())
        .await
        .unwrap()
        .send_compressed(CompressionEncoding::Zstd);

    let allocation_id = Address::from_str("0xabababababababababababababababababababab").unwrap();

    // Create receipts
    let mut receipts = Vec::new();
    for value in 50..60 {
        receipts.push(
            Eip712SignedMessage::new(
                &domain_separator,
                Receipt::new(allocation_id, value).unwrap(),
                &wallet,
            )
            .unwrap(),
        );
    }

    let rav_request = RavRequest::new(receipts.clone(), None);
    let res = client.aggregate_receipts(rav_request).await.unwrap();
    let signed_rav: tap_graph::SignedRav = res.into_inner().signed_rav().unwrap();

    let sender_aggregator = HttpClientBuilder::default().build(&endpoint).unwrap();

    let previous_rav: Option<tap_graph::SignedRav> = None;

    let response: JsonRpcResponse<Eip712SignedMessage<ReceiptAggregateVoucher>> = sender_aggregator
        .request(
            "aggregate_receipts",
            rpc_params!(
                "0.0", // TODO: Set the version in a smarter place.
                receipts,
                previous_rav
            ),
        )
        .await
        .unwrap();
    let response = response.data;
    assert_eq!(signed_rav, response);
    join_handle.abort();
}
