// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{atomic::AtomicBool, Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::{dyn_abi::Eip712Domain, primitives::Address, signers::local::PrivateKeySigner};
use anyhow::anyhow;
use rstest::*;

fn get_current_timestamp_u64_ns() -> anyhow::Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64)
}

use tap_core::{
    manager::{
        adapters::ReceiptRead,
        context::memory::{
            checks::get_full_list_of_checks, EscrowStorage, InMemoryContext, QueryAppraisals,
        },
        Manager,
    },
    receipt::{
        checks::{Check, CheckError, CheckList, StatefulTimestampCheck},
        state::Checking,
        Context, ReceiptWithState,
    },
    signed_message::EIP712SignedMessage,
    tap_eip712_domain,
};
use tap_graph::{Receipt, ReceiptAggregateVoucher, SignedReceipt};

#[fixture]
fn signer() -> PrivateKeySigner {
    PrivateKeySigner::random()
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
fn sender_ids(signer: PrivateKeySigner) -> (PrivateKeySigner, Vec<Address>) {
    let address = signer.address();
    (
        signer,
        vec![
            Address::from_str("0xfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfb").unwrap(),
            Address::from_str("0xfafafafafafafafafafafafafafafafafafafafa").unwrap(),
            Address::from_str("0xadadadadadadadadadadadadadadadadadadadad").unwrap(),
            address,
        ],
    )
}

#[fixture]
fn domain_separator() -> Eip712Domain {
    tap_eip712_domain(1, Address::from([0x11u8; 20]))
}

struct ContextFixture {
    context: InMemoryContext,
    escrow_storage: EscrowStorage,
    query_appraisals: QueryAppraisals,
    checks: CheckList<SignedReceipt>,
    signer: PrivateKeySigner,
}

#[fixture]
fn context(
    domain_separator: Eip712Domain,
    allocation_ids: Vec<Address>,
    sender_ids: (PrivateKeySigner, Vec<Address>),
) -> ContextFixture {
    let (signer, sender_ids) = sender_ids;
    let escrow_storage = Arc::new(RwLock::new(HashMap::new()));
    let rav_storage = Arc::new(RwLock::new(None));
    let query_appraisals = Arc::new(RwLock::new(HashMap::new()));
    let receipt_storage = Arc::new(RwLock::new(HashMap::new()));
    let timestamp_check = Arc::new(StatefulTimestampCheck::new(0));
    let context = InMemoryContext::new(
        rav_storage,
        receipt_storage.clone(),
        escrow_storage.clone(),
        timestamp_check.clone(),
    )
    .with_sender_address(signer.address());

    let mut checks = get_full_list_of_checks(
        domain_separator,
        sender_ids.iter().cloned().collect(),
        Arc::new(RwLock::new(allocation_ids.iter().cloned().collect())),
        query_appraisals.clone(),
    );
    checks.push(timestamp_check);
    let checks = CheckList::new(checks);

    ContextFixture {
        signer,
        context,
        escrow_storage,
        query_appraisals,
        checks,
    }
}

#[rstest]
#[tokio::test]
async fn manager_verify_and_store_varying_initial_checks(
    allocation_ids: Vec<Address>,
    domain_separator: Eip712Domain,
    context: ContextFixture,
) {
    let ContextFixture {
        context,
        checks,
        query_appraisals,
        escrow_storage,
        signer,
        ..
    } = context;
    let manager =
        Manager::<_, _, ReceiptAggregateVoucher>::new(domain_separator.clone(), context, checks);

    let value = 20u128;
    let signed_receipt = EIP712SignedMessage::new(
        &domain_separator,
        Receipt::new(allocation_ids[0], value).unwrap(),
        &signer,
    )
    .unwrap();
    let query_id = signed_receipt.unique_hash();
    query_appraisals.write().unwrap().insert(query_id, value);
    escrow_storage
        .write()
        .unwrap()
        .insert(signer.address(), 999999);

    assert!(manager
        .verify_and_store_receipt(&Context::new(), signed_receipt)
        .await
        .is_ok());
}

#[rstest]
#[tokio::test]
async fn manager_create_rav_request_all_valid_receipts(
    allocation_ids: Vec<Address>,
    domain_separator: Eip712Domain,
    context: ContextFixture,
) {
    let ContextFixture {
        context,
        checks,
        query_appraisals,
        escrow_storage,
        signer,
        ..
    } = context;
    let manager = Manager::new(domain_separator.clone(), context, checks);
    escrow_storage
        .write()
        .unwrap()
        .insert(signer.address(), 999999);

    let mut stored_signed_receipts = Vec::new();
    for _ in 0..10 {
        let value = 20u128;
        let signed_receipt = EIP712SignedMessage::new(
            &domain_separator,
            Receipt::new(allocation_ids[0], value).unwrap(),
            &signer,
        )
        .unwrap();
        let query_id = signed_receipt.unique_hash();
        stored_signed_receipts.push(signed_receipt.clone());
        query_appraisals.write().unwrap().insert(query_id, value);
        assert!(manager
            .verify_and_store_receipt(&Context::new(), signed_receipt)
            .await
            .is_ok());
    }
    let rav_request_result = manager.create_rav_request(&Context::new(), 0, None).await;
    assert!(rav_request_result.is_ok());

    let rav_request = rav_request_result.unwrap();
    // all passing
    assert_eq!(
        rav_request.valid_receipts.len(),
        stored_signed_receipts.len()
    );
    // no failing
    assert_eq!(rav_request.invalid_receipts.len(), 0);

    let expected_rav = rav_request.expected_rav.unwrap();

    let signed_rav =
        EIP712SignedMessage::new(&domain_separator, expected_rav.clone(), &signer).unwrap();
    assert!(manager
        .verify_and_store_rav(expected_rav, signed_rav)
        .await
        .is_ok());
}

#[rstest]
#[tokio::test]
async fn deny_rav_due_to_wrong_value(domain_separator: Eip712Domain, context: ContextFixture) {
    let ContextFixture {
        context,
        checks,
        signer,
        ..
    } = context;
    let manager = Manager::new(domain_separator.clone(), context, checks);

    let rav = ReceiptAggregateVoucher {
        allocationId: Address::from_str("0xabababababababababababababababababababab").unwrap(),
        timestampNs: 1232442,
        valueAggregate: 20u128,
    };

    let rav_wrong_value = ReceiptAggregateVoucher {
        allocationId: Address::from_str("0xabababababababababababababababababababab").unwrap(),
        timestampNs: 1232442,
        valueAggregate: 10u128,
    };

    let signed_rav_with_wrong_aggregate =
        EIP712SignedMessage::new(&domain_separator, rav_wrong_value, &signer).unwrap();

    assert!(manager
        .verify_and_store_rav(rav, signed_rav_with_wrong_aggregate)
        .await
        .is_err());
}

#[rstest]
#[tokio::test]
async fn manager_create_multiple_rav_requests_all_valid_receipts(
    allocation_ids: Vec<Address>,
    domain_separator: Eip712Domain,
    context: ContextFixture,
) {
    let ContextFixture {
        context,
        checks,
        query_appraisals,
        escrow_storage,
        signer,
        ..
    } = context;

    let manager = Manager::new(domain_separator.clone(), context, checks);

    escrow_storage
        .write()
        .unwrap()
        .insert(signer.address(), 999999);

    let mut stored_signed_receipts = Vec::new();
    let mut expected_accumulated_value = 0;
    for _ in 0..10 {
        let value = 20u128;
        let signed_receipt = EIP712SignedMessage::new(
            &domain_separator,
            Receipt::new(allocation_ids[0], value).unwrap(),
            &signer,
        )
        .unwrap();
        let query_id = signed_receipt.unique_hash();
        stored_signed_receipts.push(signed_receipt.clone());
        query_appraisals.write().unwrap().insert(query_id, value);
        assert!(manager
            .verify_and_store_receipt(&Context::new(), signed_receipt)
            .await
            .is_ok());
        expected_accumulated_value += value;
    }
    let rav_request_result = manager.create_rav_request(&Context::new(), 0, None).await;
    assert!(rav_request_result.is_ok());

    let rav_request = rav_request_result.unwrap();
    // all receipts passing
    assert_eq!(
        rav_request.valid_receipts.len(),
        stored_signed_receipts.len()
    );
    // no receipts failing
    assert_eq!(rav_request.invalid_receipts.len(), 0);

    let expected_rav = rav_request.expected_rav.unwrap();
    // accumulated value is correct
    assert_eq!(expected_rav.valueAggregate, expected_accumulated_value);
    // no previous rav
    assert!(rav_request.previous_rav.is_none());

    let signed_rav =
        EIP712SignedMessage::new(&domain_separator, expected_rav.clone(), &signer).unwrap();
    assert!(manager
        .verify_and_store_rav(expected_rav, signed_rav)
        .await
        .is_ok());

    stored_signed_receipts.clear();
    for _ in 10..20 {
        let value = 20u128;
        let signed_receipt = EIP712SignedMessage::new(
            &domain_separator,
            Receipt::new(allocation_ids[0], value).unwrap(),
            &signer,
        )
        .unwrap();

        let query_id = signed_receipt.unique_hash();
        stored_signed_receipts.push(signed_receipt.clone());
        query_appraisals.write().unwrap().insert(query_id, value);
        assert!(manager
            .verify_and_store_receipt(&Context::new(), signed_receipt)
            .await
            .is_ok());
        expected_accumulated_value += value;
    }
    let rav_request_result = manager.create_rav_request(&Context::new(), 0, None).await;
    assert!(rav_request_result.is_ok());

    let rav_request = rav_request_result.unwrap();
    // all receipts passing
    assert_eq!(
        rav_request.valid_receipts.len(),
        stored_signed_receipts.len()
    );
    // no receipts failing
    assert_eq!(rav_request.invalid_receipts.len(), 0);

    let expected_rav = rav_request.expected_rav.unwrap();
    // accumulated value is correct
    assert_eq!(expected_rav.valueAggregate, expected_accumulated_value);
    // Verify there is a previous rav
    assert!(rav_request.previous_rav.is_some());

    let signed_rav =
        EIP712SignedMessage::new(&domain_separator, expected_rav.clone(), &signer).unwrap();
    assert!(manager
        .verify_and_store_rav(expected_rav, signed_rav)
        .await
        .is_ok());
}

#[rstest]
#[tokio::test]
async fn manager_create_multiple_rav_requests_all_valid_receipts_consecutive_timestamps(
    allocation_ids: Vec<Address>,
    domain_separator: Eip712Domain,
    #[values(true, false)] remove_old_receipts: bool,
    context: ContextFixture,
) {
    let ContextFixture {
        context,
        checks,
        query_appraisals,
        escrow_storage,
        signer,
        ..
    } = context;
    let starting_min_timestamp = get_current_timestamp_u64_ns().unwrap() - 500000000;

    let manager = Manager::new(domain_separator.clone(), context.clone(), checks);

    escrow_storage
        .write()
        .unwrap()
        .insert(signer.address(), 999999);

    let mut stored_signed_receipts = Vec::new();
    let mut expected_accumulated_value = 0;
    for query_id in 0..10 {
        let value = 20u128;
        let mut receipt = Receipt::new(allocation_ids[0], value).unwrap();
        receipt.timestamp_ns = starting_min_timestamp + query_id + 1;
        let signed_receipt = EIP712SignedMessage::new(&domain_separator, receipt, &signer).unwrap();

        let query_id = signed_receipt.unique_hash();
        stored_signed_receipts.push(signed_receipt.clone());
        query_appraisals.write().unwrap().insert(query_id, value);
        assert!(manager
            .verify_and_store_receipt(&Context::new(), signed_receipt)
            .await
            .is_ok());
        expected_accumulated_value += value;
    }

    // Remove old receipts if requested
    // This shouldn't do anything since there has been no rav created yet
    if remove_old_receipts {
        manager.remove_obsolete_receipts().await.unwrap();
    }

    let rav_request_1_result = manager.create_rav_request(&Context::new(), 0, None).await;
    assert!(rav_request_1_result.is_ok());

    let rav_request_1 = rav_request_1_result.unwrap();
    // all receipts passing
    assert_eq!(
        rav_request_1.valid_receipts.len(),
        stored_signed_receipts.len()
    );
    // no receipts failing
    assert_eq!(rav_request_1.invalid_receipts.len(), 0);

    let expected_rav_1 = rav_request_1.expected_rav.unwrap();
    // accumulated value is correct
    assert_eq!(expected_rav_1.valueAggregate, expected_accumulated_value);
    // no previous rav
    assert!(rav_request_1.previous_rav.is_none());

    let signed_rav_1 =
        EIP712SignedMessage::new(&domain_separator, expected_rav_1.clone(), &signer).unwrap();
    assert!(manager
        .verify_and_store_rav(expected_rav_1, signed_rav_1)
        .await
        .is_ok());

    stored_signed_receipts.clear();
    for query_id in 10..20 {
        let value = 20u128;
        let mut receipt = Receipt::new(allocation_ids[0], value).unwrap();
        receipt.timestamp_ns = starting_min_timestamp + query_id + 1;
        let signed_receipt = EIP712SignedMessage::new(&domain_separator, receipt, &signer).unwrap();
        let query_id = signed_receipt.unique_hash();
        stored_signed_receipts.push(signed_receipt.clone());
        query_appraisals.write().unwrap().insert(query_id, value);
        assert!(manager
            .verify_and_store_receipt(&Context::new(), signed_receipt)
            .await
            .is_ok());
        expected_accumulated_value += value;
    }

    // Remove old receipts if requested
    if remove_old_receipts {
        manager.remove_obsolete_receipts().await.unwrap();
        // We expect to have 10 receipts left in receipt storage
        assert_eq!(
            context
                .retrieve_receipts_in_timestamp_range(.., None)
                .await
                .unwrap()
                .len(),
            10
        );
    }

    let rav_request_2_result = manager.create_rav_request(&Context::new(), 0, None).await;
    assert!(rav_request_2_result.is_ok());

    let rav_request_2 = rav_request_2_result.unwrap();
    // all receipts passing
    assert_eq!(
        rav_request_2.valid_receipts.len(),
        stored_signed_receipts.len()
    );
    // no receipts failing
    assert_eq!(rav_request_2.invalid_receipts.len(), 0);

    let expected_rav_2 = rav_request_2.expected_rav.unwrap();
    // accumulated value is correct
    assert_eq!(expected_rav_2.valueAggregate, expected_accumulated_value);
    // Verify there is a previous rav
    assert!(rav_request_2.previous_rav.is_some());

    let signed_rav_2 =
        EIP712SignedMessage::new(&domain_separator, expected_rav_2.clone(), &signer).unwrap();
    assert!(manager
        .verify_and_store_rav(expected_rav_2, signed_rav_2)
        .await
        .is_ok());
}

#[rstest]
#[tokio::test]
async fn manager_create_rav_and_ignore_invalid_receipts(
    allocation_ids: Vec<Address>,
    domain_separator: Eip712Domain,
    context: ContextFixture,
) {
    let ContextFixture {
        context,
        checks,
        escrow_storage,
        signer,
        ..
    } = context;

    let manager = Manager::new(domain_separator.clone(), context.clone(), checks);

    escrow_storage
        .write()
        .unwrap()
        .insert(signer.address(), 999999);

    let mut stored_signed_receipts = Vec::new();
    //Forcing all receipts but one to be invalid by making all the same
    for _ in 0..10 {
        let receipt = Receipt {
            allocation_id: allocation_ids[0],
            timestamp_ns: 1,
            nonce: 1,
            value: 20u128,
        };
        let signed_receipt = EIP712SignedMessage::new(&domain_separator, receipt, &signer).unwrap();
        stored_signed_receipts.push(signed_receipt.clone());
        manager
            .verify_and_store_receipt(&Context::new(), signed_receipt)
            .await
            .unwrap();
    }

    let rav_request = manager
        .create_rav_request(&Context::new(), 0, None)
        .await
        .unwrap();
    let expected_rav = rav_request.expected_rav.unwrap();

    assert_eq!(rav_request.valid_receipts.len(), 1);
    // All receipts but one being invalid
    assert_eq!(rav_request.invalid_receipts.len(), 9);
    //Rav Value corresponds only to value of one receipt
    assert_eq!(expected_rav.valueAggregate, 20);
}

#[rstest]
#[tokio::test]
async fn test_retryable_checks(
    allocation_ids: Vec<Address>,
    domain_separator: Eip712Domain,
    context: ContextFixture,
) {
    struct RetryableCheck(Arc<AtomicBool>);

    #[async_trait::async_trait]
    impl Check<SignedReceipt> for RetryableCheck {
        async fn check(
            &self,
            _: &Context,
            receipt: &ReceiptWithState<Checking, SignedReceipt>,
        ) -> Result<(), CheckError> {
            // we want to fail only if nonce is 5 and if is create rav step
            if self.0.load(std::sync::atomic::Ordering::SeqCst)
                && receipt.signed_receipt().message.nonce == 5
            {
                Err(CheckError::Retryable(anyhow!("Retryable error")))
            } else {
                Ok(())
            }
        }
    }

    let ContextFixture {
        context,
        checks,
        escrow_storage,
        signer,
        ..
    } = context;

    let is_create_rav = Arc::new(AtomicBool::new(false));

    let mut checks: Vec<Arc<dyn Check<SignedReceipt> + Send + Sync>> =
        checks.iter().cloned().collect();
    checks.push(Arc::new(RetryableCheck(is_create_rav.clone())));

    let manager = Manager::new(
        domain_separator.clone(),
        context.clone(),
        CheckList::new(checks),
    );

    escrow_storage
        .write()
        .unwrap()
        .insert(signer.address(), 999999);

    let mut stored_signed_receipts = Vec::new();
    for i in 0..10 {
        let receipt = Receipt {
            allocation_id: allocation_ids[0],
            timestamp_ns: i + 1,
            nonce: i,
            value: 20u128,
        };
        let signed_receipt = EIP712SignedMessage::new(&domain_separator, receipt, &signer).unwrap();
        stored_signed_receipts.push(signed_receipt.clone());
        manager
            .verify_and_store_receipt(&Context::new(), signed_receipt)
            .await
            .unwrap();
    }

    is_create_rav.store(true, std::sync::atomic::Ordering::SeqCst);

    let rav_request = manager.create_rav_request(&Context::new(), 0, None).await;

    assert_eq!(
        rav_request.expect_err("Didn't fail").to_string(),
        tap_core::Error::ReceiptError(tap_core::receipt::ReceiptError::RetryableCheck(
            "Retryable error".to_string()
        ))
        .to_string()
    );
}
