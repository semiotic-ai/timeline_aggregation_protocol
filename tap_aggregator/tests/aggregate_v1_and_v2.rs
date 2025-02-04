// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;

use alloy::{
    primitives::{address, Address},
    signers::local::PrivateKeySigner,
};
use tap_aggregator::{
    grpc::{
        v1::{tap_aggregator_client::TapAggregatorClient as ClientV1, RavRequest as ReqV1},
        v2::{tap_aggregator_client::TapAggregatorClient as ClientV2, RavRequest as ReqV2},
    },
    server,
};
use tap_core::{signed_message::Eip712SignedMessage, tap_eip712_domain};
use tap_graph::{v2::Receipt as ReceiptV2, Receipt as ReceiptV1};
use tonic::codec::CompressionEncoding;

#[tokio::test]
async fn aggregation_test() {
    let domain_separator = tap_eip712_domain(1, Address::ZERO);

    let wallet = PrivateKeySigner::random();

    let max_request_body_size = 1024 * 100;
    let max_response_body_size = 1024 * 100;
    let max_concurrent_connections = 1;

    let accepted_addresses = HashSet::from([wallet.address()]);

    let (_, local_addr) = server::run_server(
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

    let mut client = ClientV1::connect(endpoint.clone())
        .await
        .unwrap()
        .send_compressed(CompressionEncoding::Zstd);

    let allocation_id = address!("abababababababababababababababababababab");

    // Create receipts
    let mut receipts = Vec::new();
    for value in 50..60 {
        receipts.push(
            Eip712SignedMessage::new(
                &domain_separator,
                ReceiptV1::new(allocation_id, value).unwrap(),
                &wallet,
            )
            .unwrap(),
        );
    }

    let rav_request = ReqV1::new(receipts.clone(), None);
    let res = client.aggregate_receipts(rav_request).await;

    assert!(res.is_ok());

    let mut client = ClientV2::connect(endpoint.clone())
        .await
        .unwrap()
        .send_compressed(CompressionEncoding::Zstd);

    let payer = address!("abababababababababababababababababababab");
    let data_service = address!("abababababababababababababababababababab");
    let service_provider = address!("abababababababababababababababababababab");

    // Create receipts
    let mut receipts = Vec::new();
    for value in 50..60 {
        receipts.push(
            Eip712SignedMessage::new(
                &domain_separator,
                ReceiptV2::new(allocation_id, payer, data_service, service_provider, value)
                    .unwrap(),
                &wallet,
            )
            .unwrap(),
        );
    }

    let rav_request = ReqV2::new(receipts.clone(), None);
    let res = client.aggregate_receipts(rav_request).await;

    assert!(res.is_ok());
}
