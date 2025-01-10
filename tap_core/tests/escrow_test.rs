// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use alloy::signers::local::PrivateKeySigner;
use rstest::*;
use tap_core::{
    manager::{adapters::EscrowHandler, context::memory::InMemoryContext},
    receipt::checks::StatefulTimestampCheck,
};

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
        timestamp_check,
    )
}

#[rstest]
#[tokio::test]
async fn escrow_handler_test(mut context: InMemoryContext) {
    let wallet = PrivateKeySigner::random();
    let sender_id = wallet.address();

    let invalid_wallet = PrivateKeySigner::random();
    let invalid_sender_id = invalid_wallet.address();

    let initial_value = 500u128;

    context.increase_escrow(sender_id, initial_value);

    // Check that sender exists and has valid value through adapter
    assert!(context.get_available_escrow(sender_id).await.is_ok());
    assert_eq!(
        context.get_available_escrow(sender_id).await.unwrap(),
        initial_value
    );

    // Check that subtracting is valid for valid sender, and results in expected value
    assert!(context
        .subtract_escrow(sender_id, initial_value)
        .await
        .is_ok());
    assert!(context.get_available_escrow(sender_id).await.is_ok());
    assert_eq!(context.get_available_escrow(sender_id).await.unwrap(), 0);

    // Check that subtracting to negative escrow results in err
    assert!(context
        .subtract_escrow(sender_id, initial_value)
        .await
        .is_err());

    // Check that accessing non initialized sender results in err
    assert!(context
        .get_available_escrow(invalid_sender_id)
        .await
        .is_err());
}
