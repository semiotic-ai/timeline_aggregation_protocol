// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0
#![doc = include_str!("../README.md")]
//! ## Getting started
//!
//! To get started with the TAP protocol, take a look on the [`manager`] module
//! to see how to manage the state channel and implement the needed adapters.

use std::time::{SystemTime, UNIX_EPOCH};

use alloy::{dyn_abi::Eip712Domain, sol_types::eip712_domain};
use thiserror::Error;

mod error;
pub mod manager;
pub mod rav_request;
pub mod receipt;
pub mod signed_message;

pub use error::Error;
use error::Result;

fn get_current_timestamp_u64_ns() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| Error::InvalidSystemTime {
            source_error_message: err.to_string(),
        })?
        .as_nanos() as u64)
}

/// The EIP712 domain separator builder for the TAP protocol.
///
/// This is the current domain separator that is used for the [EIP712](https://eips.ethereum.org/EIPS/eip-712) signature scheme.
///
///
/// It's used to validate the signature of the `ReceiptAggregateVoucher` and `Receipt` structs.
///
/// You can take a look on deployed [TAPVerfiers](https://github.com/semiotic-ai/timeline-aggregation-protocol-contracts/blob/4dc87fc616680c924b99dbaf285bdd449c777261/src/TAPVerifier.sol)
/// contracts [here](https://github.com/semiotic-ai/timeline-aggregation-protocol-contracts/blob/4dc87fc616680c924b99dbaf285bdd449c777261/addresses.json)
///
/// The domain separator is defined as:
/// - `name`: "TAP"
/// - `version`: "1"
/// - `chain_id`: The chain ID of the chain where the domain separator is deployed.
/// - `verifying_contract`: The address of the contract that is verifying the signature.
pub fn tap_eip712_domain(
    chain_id: u64,
    verifying_contract_address: alloy::primitives::Address,
) -> Eip712Domain {
    eip712_domain! {
        name: "TAP",
        version: "1",
        chain_id: chain_id,
        verifying_contract: verifying_contract_address,
    }
}

#[cfg(test)]
mod tap_tests {
    use std::str::FromStr;

    use alloy::{dyn_abi::Eip712Domain, primitives::Address, signers::local::PrivateKeySigner};
    use rstest::*;
    use tap_graph::{Receipt, ReceiptAggregateVoucher};

    use crate::{signed_message::EIP712SignedMessage, tap_eip712_domain};

    #[fixture]
    fn keys() -> (PrivateKeySigner, Address) {
        let wallet = PrivateKeySigner::random();
        let address = wallet.address();

        (wallet, address)
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

    #[rstest]
    #[case::basic_rav_test (vec![45,56,34,23])]
    #[case::rav_from_zero_valued_receipts (vec![0,0,0,0])]
    #[test]
    fn signed_rav_is_valid_with_no_previous_rav(
        keys: (PrivateKeySigner, Address),
        allocation_ids: Vec<Address>,
        domain_separator: Eip712Domain,
        #[case] values: Vec<u128>,
    ) {
        // Create receipts
        let mut receipts = Vec::new();
        for value in values {
            receipts.push(
                EIP712SignedMessage::new(
                    &domain_separator,
                    Receipt::new(allocation_ids[0], value).unwrap(),
                    &keys.0,
                )
                .unwrap(),
            );
        }

        // Skipping receipts validation in this test, aggregate_receipts assumes receipts are valid.

        let rav = ReceiptAggregateVoucher::aggregate_receipts(allocation_ids[0], &receipts, None)
            .unwrap();
        let signed_rav = EIP712SignedMessage::new(&domain_separator, rav, &keys.0).unwrap();
        assert!(signed_rav.recover_signer(&domain_separator).unwrap() == keys.1);
    }

    #[rstest]
    #[case::basic_rav_test(vec![45,56,34,23])]
    #[case::rav_from_zero_valued_receipts(vec![0,0,0,0])]
    #[test]
    fn signed_rav_is_valid_with_previous_rav(
        keys: (PrivateKeySigner, Address),
        allocation_ids: Vec<Address>,
        domain_separator: Eip712Domain,
        #[case] values: Vec<u128>,
    ) {
        // Create receipts
        let mut receipts = Vec::new();
        for value in values {
            receipts.push(
                EIP712SignedMessage::new(
                    &domain_separator,
                    Receipt::new(allocation_ids[0], value).unwrap(),
                    &keys.0,
                )
                .unwrap(),
            );
        }

        // Create previous RAV from first half of receipts
        let prev_rav = ReceiptAggregateVoucher::aggregate_receipts(
            allocation_ids[0],
            &receipts[0..receipts.len() / 2],
            None,
        )
        .unwrap();
        let signed_prev_rav =
            EIP712SignedMessage::new(&domain_separator, prev_rav, &keys.0).unwrap();

        // Create new RAV from last half of receipts and prev_rav
        let rav = ReceiptAggregateVoucher::aggregate_receipts(
            allocation_ids[0],
            &receipts[receipts.len() / 2..receipts.len()],
            Some(signed_prev_rav),
        )
        .unwrap();
        let signed_rav = EIP712SignedMessage::new(&domain_separator, rav, &keys.0).unwrap();

        assert!(signed_rav.recover_signer(&domain_separator).unwrap() == keys.1);
    }

    #[rstest]
    #[test]
    fn verify_signature(
        keys: (PrivateKeySigner, Address),
        allocation_ids: Vec<Address>,
        domain_separator: Eip712Domain,
    ) {
        let signed_message = EIP712SignedMessage::new(
            &domain_separator,
            Receipt::new(allocation_ids[0], 42).unwrap(),
            &keys.0,
        )
        .unwrap();

        assert!(signed_message.verify(&domain_separator, keys.1).is_ok());
        assert!(signed_message
            .verify(
                &domain_separator,
                Address::from_str("0x76f4eeD9fE41262669D0250b2A97db79712aD855").unwrap()
            )
            .is_err());
    }
}
