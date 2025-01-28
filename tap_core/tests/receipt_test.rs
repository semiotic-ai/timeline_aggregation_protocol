// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};

use alloy::{dyn_abi::Eip712Domain, primitives::Address, signers::local::PrivateKeySigner};
use rand::{seq::SliceRandom, thread_rng};
use rstest::*;
use tap_core::{
    manager::{adapters::ReceiptStore, context::memory::InMemoryContext},
    receipt::{checks::StatefulTimestampCheck, state::Checking, ReceiptWithState},
    signed_message::EIP712SignedMessage,
    tap_eip712_domain,
};
use tap_graph::{Receipt, SignedReceipt};

#[fixture]
fn domain_separator() -> Eip712Domain {
    tap_eip712_domain(1, Address::from([0x11u8; 20]))
}

#[fixture]
fn context() -> InMemoryContext {
    let escrow_storage = Arc::new(RwLock::new(HashMap::new()));
    let rav_storage = Arc::new(RwLock::new(None));
    let receipt_storage = Arc::new(RwLock::new(HashMap::new()));

    let timestamp_check = Arc::new(StatefulTimestampCheck::new(0));
    InMemoryContext::new(
        rav_storage,
        receipt_storage.clone(),
        escrow_storage.clone(),
        timestamp_check.clone(),
    )
}

#[rstest]
#[tokio::test]
async fn receipt_adapter_test(domain_separator: Eip712Domain, mut context: InMemoryContext) {
    let wallet = PrivateKeySigner::random();

    let allocation_id = Address::from_str("0xabababababababababababababababababababab").unwrap();

    // Create receipts
    let value = 100u128;
    let received_receipt = ReceiptWithState::new(
        EIP712SignedMessage::new(
            &domain_separator,
            Receipt::new(allocation_id, value).unwrap(),
            &wallet,
        )
        .unwrap(),
    );

    let receipt_store_result = context.store_receipt(received_receipt).await;
    assert!(receipt_store_result.is_ok());
    let receipt_id = receipt_store_result.unwrap();

    // Retreive receipt with id expected to be valid
    assert!(context.retrieve_receipt_by_id(receipt_id).await.is_ok());
    // Retreive receipt with arbitrary id expected to be invalid
    assert!(context.retrieve_receipt_by_id(999).await.is_err());

    // Remove receipt with id expected to be valid
    assert!(context.remove_receipt_by_id(receipt_id).await.is_ok());
    // Remove receipt with arbitrary id expected to be invalid
    assert!(context.remove_receipt_by_id(999).await.is_err());

    // Retreive receipt that was removed previously
    assert!(context.retrieve_receipt_by_id(receipt_id).await.is_err());

    // Remove receipt that was removed previously
    assert!(context.remove_receipt_by_id(receipt_id).await.is_err());
}

#[rstest]
#[tokio::test]
async fn multi_receipt_adapter_test(domain_separator: Eip712Domain, mut context: InMemoryContext) {
    let wallet = PrivateKeySigner::random();

    let allocation_id = Address::from_str("0xabababababababababababababababababababab").unwrap();

    // Create receipts
    let mut received_receipts = Vec::new();
    for value in 50..60 {
        received_receipts.push(ReceiptWithState::new(
            EIP712SignedMessage::new(
                &domain_separator,
                Receipt::new(allocation_id, value).unwrap(),
                &wallet,
            )
            .unwrap(),
        ));
    }
    let mut receipt_ids = Vec::new();
    let mut receipt_timestamps = Vec::new();
    for received_receipt in received_receipts {
        receipt_timestamps.push(received_receipt.signed_receipt().message.timestamp_ns);
        receipt_ids.push(context.store_receipt(received_receipt).await.unwrap());
    }

    // Retreive receipts with timestamp
    assert!(context
        .retrieve_receipts_by_timestamp(receipt_timestamps[0])
        .await
        .is_ok());
    assert!(!context
        .retrieve_receipts_by_timestamp(receipt_timestamps[0])
        .await
        .unwrap()
        .is_empty());

    // Retreive receipts before timestamp
    assert!(context
        .retrieve_receipts_upto_timestamp(receipt_timestamps[3])
        .await
        .is_ok());
    assert!(
        context
            .retrieve_receipts_upto_timestamp(receipt_timestamps[3])
            .await
            .unwrap()
            .len()
            >= 4
    );

    // Remove all receipts with one call
    assert!(context
        .remove_receipts_by_ids(receipt_ids.as_slice())
        .await
        .is_ok());
    // Removal should no longer be valid
    assert!(context
        .remove_receipts_by_ids(receipt_ids.as_slice())
        .await
        .is_err());
    // Retrieval should be invalid
    for receipt_id in receipt_ids {
        assert!(context.retrieve_receipt_by_id(receipt_id).await.is_err());
    }
}

/// The test code will shuffle the input timestamps prior to calling safe_truncate_receipts.
#[rstest]
#[case(vec![1, 2, 3, 4, 5], 3, vec![1, 2, 3])]
#[case(vec![1, 2, 3, 3, 4, 5], 3, vec![1, 2])]
#[case(vec![1, 2, 3, 4, 4, 4], 3, vec![1, 2, 3])]
#[case(vec![1, 1, 1, 1, 2, 3], 3, vec![])]
#[test]
fn safe_truncate_receipts_test(
    domain_separator: Eip712Domain,
    #[case] input: Vec<u64>,
    #[case] limit: u64,
    #[case] expected: Vec<u64>,
) {
    let wallet = PrivateKeySigner::random();

    // Vec of (id, receipt)
    let mut receipts_orig: Vec<ReceiptWithState<Checking, SignedReceipt>> = Vec::new();

    for timestamp in input.iter() {
        // The contents of the receipt only need to be unique for this test (so we can check)
        receipts_orig.push(ReceiptWithState::new(
            EIP712SignedMessage::new(
                &domain_separator,
                Receipt {
                    allocation_id: Address::ZERO,
                    timestamp_ns: *timestamp,
                    nonce: 0,
                    value: 0,
                },
                &wallet,
            )
            .unwrap(),
        ));
    }

    let mut receipts_truncated = receipts_orig;

    // shuffle the input receipts
    receipts_truncated.shuffle(&mut thread_rng());

    tap_core::manager::adapters::safe_truncate_receipts(&mut receipts_truncated, limit);

    assert_eq!(receipts_truncated.len(), expected.len());

    for (elem_trun, expected_timestamp) in receipts_truncated.iter().zip(expected.iter()) {
        // Check timestamps
        assert_eq!(
            elem_trun.signed_receipt().message.timestamp_ns,
            *expected_timestamp
        );
    }
}
