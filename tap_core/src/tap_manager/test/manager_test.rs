// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0
use std::{collections::HashMap, ops::Range, str::FromStr, sync::Arc};

use alloy_primitives::Address;
use alloy_sol_types::Eip712Domain;
use ethers::signers::{coins_bip39::English, LocalWallet, MnemonicBuilder, Signer};
use rstest::*;
use tokio::sync::RwLock;

use super::super::Manager;
use crate::{
    adapters::{
        executor_mock::{EscrowStorage, ExecutorMock, QueryAppraisals, RAVStorage, ReceiptStorage},
        receipt_storage_adapter::ReceiptRead,
    },
    checks::{tests::get_full_list_of_checks, ReceiptCheck},
    eip_712_signed_message::EIP712SignedMessage,
    get_current_timestamp_u64_ns, tap_eip712_domain,
    tap_receipt::Receipt,
};

#[fixture]
fn keys() -> (LocalWallet, Address) {
    let wallet: LocalWallet = MnemonicBuilder::<English>::default()
        .phrase("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about")
        .build()
        .unwrap();
    // Alloy library does not have feature parity with ethers library (yet) This workaround is needed to get the address
    // to convert to an alloy Address. This will not be needed when the alloy library has wallet support.
    let address: [u8; 20] = wallet.address().into();

    (wallet, address.into())
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
fn sender_ids() -> Vec<Address> {
    vec![
        Address::from_str("0xfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfbfb").unwrap(),
        Address::from_str("0xfafafafafafafafafafafafafafafafafafafafa").unwrap(),
        Address::from_str("0xadadadadadadadadadadadadadadadadadadadad").unwrap(),
        keys().1,
    ]
}

#[fixture]
fn receipt_storage() -> ReceiptStorage {
    Arc::new(RwLock::new(HashMap::new()))
}

#[fixture]
fn query_appraisal_storage() -> QueryAppraisals {
    Arc::new(RwLock::new(HashMap::new()))
}

#[fixture]
fn rav_storage() -> RAVStorage {
    Arc::new(RwLock::new(None))
}

#[fixture]
fn escrow_storage() -> EscrowStorage {
    Arc::new(RwLock::new(HashMap::new()))
}

#[fixture]
fn domain_separator() -> Eip712Domain {
    tap_eip712_domain(1, Address::from([0x11u8; 20]))
}

struct ExecutorFixture {
    executor: ExecutorMock,
    escrow_storage: EscrowStorage,
    query_appraisals: QueryAppraisals,
    checks: Vec<ReceiptCheck>,
}

#[fixture]
fn executor_mock(
    domain_separator: Eip712Domain,
    allocation_ids: Vec<Address>,
    sender_ids: Vec<Address>,
    receipt_storage: ReceiptStorage,
    query_appraisal_storage: QueryAppraisals,
    rav_storage: RAVStorage,
    escrow_storage: EscrowStorage,
) -> ExecutorFixture {
    let executor = ExecutorMock::new(rav_storage, receipt_storage.clone(), escrow_storage.clone());

    let checks = get_full_list_of_checks(
        domain_separator,
        sender_ids.iter().cloned().collect(),
        Arc::new(RwLock::new(allocation_ids.iter().cloned().collect())),
        receipt_storage,
        query_appraisal_storage.clone(),
    );

    ExecutorFixture {
        executor,
        escrow_storage,
        query_appraisals: query_appraisal_storage,
        checks,
    }
}

#[rstest]
#[case::full_checks(0..5)]
#[case::partial_checks(0..2)]
#[case::no_checks(0..0)]
#[tokio::test]
async fn manager_verify_and_store_varying_initial_checks(
    keys: (LocalWallet, Address),
    allocation_ids: Vec<Address>,
    domain_separator: Eip712Domain,
    #[case] range: Range<usize>,
    executor_mock: ExecutorFixture,
) {
    let ExecutorFixture {
        executor,
        checks,
        query_appraisals,
        escrow_storage,
        ..
    } = executor_mock;
    // give receipt 5 second variance for min start time
    let starting_min_timestamp = get_current_timestamp_u64_ns().unwrap() - 500000000;

    let manager = Manager::new(
        domain_separator.clone(),
        executor,
        checks.clone(),
        starting_min_timestamp,
    );

    let query_id = 1;
    let value = 20u128;
    let signed_receipt = EIP712SignedMessage::new(
        &domain_separator,
        Receipt::new(allocation_ids[0], value).unwrap(),
        &keys.0,
    )
    .unwrap();
    query_appraisals.write().await.insert(query_id, value);
    escrow_storage.write().await.insert(keys.1, 999999);

    assert!(manager
        .verify_and_store_receipt(signed_receipt, query_id, &checks[range])
        .await
        .is_ok());
}

#[rstest]
#[case::full_checks(0..5)]
#[case::partial_checks(0..2)]
#[case::no_checks(0..0)]
#[tokio::test]
async fn manager_create_rav_request_all_valid_receipts(
    keys: (LocalWallet, Address),
    allocation_ids: Vec<Address>,
    domain_separator: Eip712Domain,
    #[case] range: Range<usize>,
    executor_mock: ExecutorFixture,
) {
    let ExecutorFixture {
        executor,
        checks,
        query_appraisals,
        escrow_storage,
        ..
    } = executor_mock;
    let initial_checks = &checks[range];
    // give receipt 5 second variance for min start time
    let starting_min_timestamp = get_current_timestamp_u64_ns().unwrap() - 500000000;

    let manager = Manager::new(
        domain_separator.clone(),
        executor,
        checks.clone(),
        starting_min_timestamp,
    );
    escrow_storage.write().await.insert(keys.1, 999999);

    let mut stored_signed_receipts = Vec::new();
    for query_id in 0..10 {
        let value = 20u128;
        let signed_receipt = EIP712SignedMessage::new(
            &domain_separator,
            Receipt::new(allocation_ids[0], value).unwrap(),
            &keys.0,
        )
        .unwrap();
        stored_signed_receipts.push(signed_receipt.clone());
        query_appraisals.write().await.insert(query_id, value);
        assert!(manager
            .verify_and_store_receipt(signed_receipt, query_id, initial_checks)
            .await
            .is_ok());
    }
    let rav_request_result = manager.create_rav_request(0, None).await;
    println!("{:?}", rav_request_result);
    assert!(rav_request_result.is_ok());

    let rav_request = rav_request_result.unwrap();
    // all passing
    assert_eq!(
        rav_request.valid_receipts.len(),
        stored_signed_receipts.len()
    );
    // no failing
    assert_eq!(rav_request.invalid_receipts.len(), 0);

    let signed_rav =
        EIP712SignedMessage::new(&domain_separator, rav_request.expected_rav.clone(), &keys.0)
            .unwrap();
    assert!(manager
        .verify_and_store_rav(
            rav_request.expected_rav,
            signed_rav,
            |address: Address| async move { Ok(keys.1 == address) }
        )
        .await
        .is_ok());
}

#[rstest]
#[case::full_checks(0..5)]
#[case::partial_checks(0..2)]
#[case::no_checks(0..0)]
#[tokio::test]
async fn manager_create_multiple_rav_requests_all_valid_receipts(
    keys: (LocalWallet, Address),
    allocation_ids: Vec<Address>,
    domain_separator: Eip712Domain,
    #[case] range: Range<usize>,
    executor_mock: ExecutorFixture,
) {
    let ExecutorFixture {
        executor,
        checks,
        query_appraisals,
        escrow_storage,
        ..
    } = executor_mock;
    let initial_checks = &checks[range];
    // give receipt 5 second variance for min start time
    let starting_min_timestamp = get_current_timestamp_u64_ns().unwrap() - 500000000;

    let manager = Manager::new(
        domain_separator.clone(),
        executor,
        checks.clone(),
        starting_min_timestamp,
    );

    escrow_storage.write().await.insert(keys.1, 999999);

    let mut stored_signed_receipts = Vec::new();
    let mut expected_accumulated_value = 0;
    for query_id in 0..10 {
        let value = 20u128;
        let signed_receipt = EIP712SignedMessage::new(
            &domain_separator,
            Receipt::new(allocation_ids[0], value).unwrap(),
            &keys.0,
        )
        .unwrap();
        stored_signed_receipts.push(signed_receipt.clone());
        query_appraisals.write().await.insert(query_id, value);
        assert!(manager
            .verify_and_store_receipt(signed_receipt, query_id, initial_checks)
            .await
            .is_ok());
        expected_accumulated_value += value;
    }
    let rav_request_result = manager.create_rav_request(0, None).await;
    assert!(rav_request_result.is_ok());

    let rav_request = rav_request_result.unwrap();
    // all receipts passing
    assert_eq!(
        rav_request.valid_receipts.len(),
        stored_signed_receipts.len()
    );
    // no receipts failing
    assert_eq!(rav_request.invalid_receipts.len(), 0);
    // accumulated value is correct
    assert_eq!(
        rav_request.expected_rav.valueAggregate,
        expected_accumulated_value
    );
    // no previous rav
    assert!(rav_request.previous_rav.is_none());

    let signed_rav =
        EIP712SignedMessage::new(&domain_separator, rav_request.expected_rav.clone(), &keys.0)
            .unwrap();
    assert!(manager
        .verify_and_store_rav(
            rav_request.expected_rav,
            signed_rav,
            |address: Address| async move { Ok(keys.1 == address) }
        )
        .await
        .is_ok());

    stored_signed_receipts.clear();
    for query_id in 10..20 {
        let value = 20u128;
        let signed_receipt = EIP712SignedMessage::new(
            &domain_separator,
            Receipt::new(allocation_ids[0], value).unwrap(),
            &keys.0,
        )
        .unwrap();
        stored_signed_receipts.push(signed_receipt.clone());
        query_appraisals.write().await.insert(query_id, value);
        assert!(manager
            .verify_and_store_receipt(signed_receipt, query_id, initial_checks)
            .await
            .is_ok());
        expected_accumulated_value += value;
    }
    let rav_request_result = manager.create_rav_request(0, None).await;
    assert!(rav_request_result.is_ok());

    let rav_request = rav_request_result.unwrap();
    // all receipts passing
    assert_eq!(
        rav_request.valid_receipts.len(),
        stored_signed_receipts.len()
    );
    // no receipts failing
    assert_eq!(rav_request.invalid_receipts.len(), 0);
    // accumulated value is correct
    assert_eq!(
        rav_request.expected_rav.valueAggregate,
        expected_accumulated_value
    );
    // Verify there is a previous rav
    assert!(rav_request.previous_rav.is_some());

    let signed_rav =
        EIP712SignedMessage::new(&domain_separator, rav_request.expected_rav.clone(), &keys.0)
            .unwrap();
    assert!(manager
        .verify_and_store_rav(
            rav_request.expected_rav,
            signed_rav,
            |address: Address| async move { Ok(keys.1 == address) }
        )
        .await
        .is_ok());
}

#[rstest]
#[tokio::test]
async fn manager_create_multiple_rav_requests_all_valid_receipts_consecutive_timestamps(
    keys: (LocalWallet, Address),
    allocation_ids: Vec<Address>,
    domain_separator: Eip712Domain,
    #[values(0..0, 0..2, 0..5)] range: Range<usize>,
    #[values(true, false)] remove_old_receipts: bool,
    executor_mock: ExecutorFixture,
) {
    let ExecutorFixture {
        executor,
        checks,
        query_appraisals,
        escrow_storage,
        ..
    } = executor_mock;
    let initial_checks = &checks[range];
    // give receipt 5 second variance for min start time
    let starting_min_timestamp = get_current_timestamp_u64_ns().unwrap() - 500000000;

    let manager = Manager::new(
        domain_separator.clone(),
        executor,
        checks.clone(),
        starting_min_timestamp,
    );

    escrow_storage.write().await.insert(keys.1, 999999);

    let mut stored_signed_receipts = Vec::new();
    let mut expected_accumulated_value = 0;
    for query_id in 0..10 {
        let value = 20u128;
        let mut receipt = Receipt::new(allocation_ids[0], value).unwrap();
        receipt.timestamp_ns = starting_min_timestamp + query_id + 1;
        let signed_receipt = EIP712SignedMessage::new(&domain_separator, receipt, &keys.0).unwrap();
        stored_signed_receipts.push(signed_receipt.clone());
        query_appraisals.write().await.insert(query_id, value);
        assert!(manager
            .verify_and_store_receipt(signed_receipt, query_id, initial_checks)
            .await
            .is_ok());
        expected_accumulated_value += value;
    }

    // Remove old receipts if requested
    // This shouldn't do anything since there has been no rav created yet
    if remove_old_receipts {
        manager.remove_obsolete_receipts().await.unwrap();
    }

    let rav_request_1_result = manager.create_rav_request(0, None).await;
    assert!(rav_request_1_result.is_ok());

    let rav_request_1 = rav_request_1_result.unwrap();
    // all receipts passing
    assert_eq!(
        rav_request_1.valid_receipts.len(),
        stored_signed_receipts.len()
    );
    // no receipts failing
    assert_eq!(rav_request_1.invalid_receipts.len(), 0);
    // accumulated value is correct
    assert_eq!(
        rav_request_1.expected_rav.valueAggregate,
        expected_accumulated_value
    );
    // no previous rav
    assert!(rav_request_1.previous_rav.is_none());

    let signed_rav_1 = EIP712SignedMessage::new(
        &domain_separator,
        rav_request_1.expected_rav.clone(),
        &keys.0,
    )
    .unwrap();
    assert!(manager
        .verify_and_store_rav(
            rav_request_1.expected_rav,
            signed_rav_1,
            |address: Address| async move { Ok(keys.1 == address) }
        )
        .await
        .is_ok());

    stored_signed_receipts.clear();
    for query_id in 10..20 {
        let value = 20u128;
        let mut receipt = Receipt::new(allocation_ids[0], value).unwrap();
        receipt.timestamp_ns = starting_min_timestamp + query_id + 1;
        let signed_receipt = EIP712SignedMessage::new(&domain_separator, receipt, &keys.0).unwrap();
        stored_signed_receipts.push(signed_receipt.clone());
        query_appraisals.write().await.insert(query_id, value);
        assert!(manager
            .verify_and_store_receipt(signed_receipt, query_id, initial_checks)
            .await
            .is_ok());
        expected_accumulated_value += value;
    }

    // Remove old receipts if requested
    if remove_old_receipts {
        manager.remove_obsolete_receipts().await.unwrap();
        // We expect to have 10 receipts left in receipt storage
        assert_eq!(
            manager
                .executor
                .retrieve_receipts_in_timestamp_range(.., None)
                .await
                .unwrap()
                .len(),
            10
        );
    }

    let rav_request_2_result = manager.create_rav_request(0, None).await;
    assert!(rav_request_2_result.is_ok());

    let rav_request_2 = rav_request_2_result.unwrap();
    // all receipts passing
    assert_eq!(
        rav_request_2.valid_receipts.len(),
        stored_signed_receipts.len()
    );
    // no receipts failing
    assert_eq!(rav_request_2.invalid_receipts.len(), 0);
    // accumulated value is correct
    assert_eq!(
        rav_request_2.expected_rav.valueAggregate,
        expected_accumulated_value
    );
    // Verify there is a previous rav
    assert!(rav_request_2.previous_rav.is_some());

    let signed_rav_2 = EIP712SignedMessage::new(
        &domain_separator,
        rav_request_2.expected_rav.clone(),
        &keys.0,
    )
    .unwrap();
    assert!(manager
        .verify_and_store_rav(
            rav_request_2.expected_rav,
            signed_rav_2,
            |address: Address| async move { Ok(keys.1 == address) }
        )
        .await
        .is_ok());
}
