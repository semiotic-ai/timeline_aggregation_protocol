// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::{hash_set, HashSet};

use alloy::{
    dyn_abi::Eip712Domain, primitives::Address, signers::local::PrivateKeySigner,
    sol_types::SolStruct,
};
use anyhow::{bail, Ok, Result};
use rayon::prelude::*;
use tap_core::signed_message::{EIP712SignedMessage, SignatureBytes, SignatureBytesExt};
use tap_graph::{Receipt, ReceiptAggregateVoucher};

pub fn check_and_aggregate_receipts(
    domain_separator: &Eip712Domain,
    receipts: &[EIP712SignedMessage<Receipt>],
    previous_rav: Option<EIP712SignedMessage<ReceiptAggregateVoucher>>,
    wallet: &PrivateKeySigner,
    accepted_addresses: &HashSet<Address>,
) -> Result<EIP712SignedMessage<ReceiptAggregateVoucher>> {
    check_signatures_unique(receipts)?;

    // Check that the receipts are signed by an accepted signer address
    receipts.par_iter().try_for_each(|receipt| {
        check_signature_is_from_one_of_addresses(receipt, domain_separator, accepted_addresses)
    })?;

    // Check that the previous rav is signed by an accepted signer address
    if let Some(previous_rav) = &previous_rav {
        check_signature_is_from_one_of_addresses(
            previous_rav,
            domain_separator,
            accepted_addresses,
        )?;
    }

    // Check that the receipts timestamp is greater than the previous rav
    check_receipt_timestamps(receipts, previous_rav.as_ref())?;

    // Get the allocation id from the first receipt, return error if there are no receipts
    let allocation_id = match receipts.first() {
        Some(receipt) => receipt.message.allocation_id,
        None => return Err(tap_core::Error::NoValidReceiptsForRavRequest.into()),
    };

    // Check that the receipts all have the same allocation id
    check_allocation_id(receipts, allocation_id)?;

    // Check that the rav has the correct allocation id
    if let Some(previous_rav) = &previous_rav {
        let prev_id = previous_rav.message.allocationId;
        if prev_id != allocation_id {
            return Err(tap_core::Error::RavAllocationIdMismatch {
                prev_id: format!("{prev_id:#X}"),
                new_id: format!("{allocation_id:#X}"),
            }
            .into());
        }
    }

    // Aggregate the receipts
    let rav = ReceiptAggregateVoucher::aggregate_receipts(allocation_id, receipts, previous_rav)?;

    // Sign the rav and return
    Ok(EIP712SignedMessage::new(domain_separator, rav, wallet)?)
}

fn check_signature_is_from_one_of_addresses<M: SolStruct>(
    message: &EIP712SignedMessage<M>,
    domain_separator: &Eip712Domain,
    accepted_addresses: &HashSet<Address>,
) -> Result<()> {
    let recovered_address = message.recover_signer(domain_separator)?;
    if !accepted_addresses.contains(&recovered_address) {
        bail!(tap_core::Error::InvalidRecoveredSigner {
            address: recovered_address,
        });
    }
    Ok(())
}

fn check_allocation_id(
    receipts: &[EIP712SignedMessage<Receipt>],
    allocation_id: Address,
) -> Result<()> {
    for receipt in receipts.iter() {
        let receipt = &receipt.message;
        if receipt.allocation_id != allocation_id {
            return Err(tap_core::Error::RavAllocationIdNotUniform.into());
        }
    }
    Ok(())
}

fn check_signatures_unique(receipts: &[EIP712SignedMessage<Receipt>]) -> Result<()> {
    let mut receipt_signatures: hash_set::HashSet<SignatureBytes> = hash_set::HashSet::new();
    for receipt in receipts.iter() {
        let signature = receipt.signature.get_signature_bytes();
        if !receipt_signatures.insert(signature) {
            return Err(tap_core::Error::DuplicateReceiptSignature(format!(
                "{:?}",
                receipt.signature
            ))
            .into());
        }
    }
    Ok(())
}

fn check_receipt_timestamps(
    receipts: &[EIP712SignedMessage<Receipt>],
    previous_rav: Option<&EIP712SignedMessage<ReceiptAggregateVoucher>>,
) -> Result<()> {
    if let Some(previous_rav) = &previous_rav {
        for receipt in receipts.iter() {
            let receipt = &receipt.message;
            if previous_rav.message.timestampNs >= receipt.timestamp_ns {
                return Err(tap_core::Error::ReceiptTimestampLowerThanRav {
                    rav_ts: previous_rav.message.timestampNs,
                    receipt_ts: receipt.timestamp_ns,
                }
                .into());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::{dyn_abi::Eip712Domain, primitives::Address, signers::local::PrivateKeySigner};
    use rstest::*;
    use tap_core::{signed_message::EIP712SignedMessage, tap_eip712_domain};
    use tap_graph::{Receipt, ReceiptAggregateVoucher};

    use crate::aggregator;

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
    #[test]
    fn check_signatures_unique_fail(
        keys: (PrivateKeySigner, Address),
        allocation_ids: Vec<Address>,
        domain_separator: Eip712Domain,
    ) {
        // Create the same receipt twice (replay attack)
        let mut receipts = Vec::new();
        let receipt = EIP712SignedMessage::new(
            &domain_separator,
            Receipt::new(allocation_ids[0], 42).unwrap(),
            &keys.0,
        )
        .unwrap();
        receipts.push(receipt.clone());
        receipts.push(receipt);

        let res = aggregator::check_signatures_unique(&receipts);
        assert!(res.is_err());
    }

    #[rstest]
    #[test]
    fn check_signatures_unique_ok(
        keys: (PrivateKeySigner, Address),
        allocation_ids: Vec<Address>,
        domain_separator: Eip712Domain,
    ) {
        // Create 2 different receipts
        let receipts = vec![
            EIP712SignedMessage::new(
                &domain_separator,
                Receipt::new(allocation_ids[0], 42).unwrap(),
                &keys.0,
            )
            .unwrap(),
            EIP712SignedMessage::new(
                &domain_separator,
                Receipt::new(allocation_ids[0], 43).unwrap(),
                &keys.0,
            )
            .unwrap(),
        ];

        let res = aggregator::check_signatures_unique(&receipts);
        assert!(res.is_ok());
    }

    #[rstest]
    #[test]
    /// Test that a receipt with a timestamp greater then the rav timestamp passes
    fn check_receipt_timestamps(
        keys: (PrivateKeySigner, Address),
        allocation_ids: Vec<Address>,
        domain_separator: Eip712Domain,
    ) {
        // Create receipts with consecutive timestamps
        let receipt_timestamp_range = 10..20;
        let mut receipts = Vec::new();
        for i in receipt_timestamp_range.clone() {
            receipts.push(
                EIP712SignedMessage::new(
                    &domain_separator,
                    Receipt {
                        allocation_id: allocation_ids[0],
                        timestamp_ns: i,
                        nonce: 0,
                        value: 42,
                    },
                    &keys.0,
                )
                .unwrap(),
            );
        }

        // Create rav with max_timestamp below the receipts timestamps
        let rav = EIP712SignedMessage::new(
            &domain_separator,
            ReceiptAggregateVoucher {
                allocationId: allocation_ids[0],
                timestampNs: receipt_timestamp_range.clone().min().unwrap() - 1,
                valueAggregate: 42,
            },
            &keys.0,
        )
        .unwrap();
        assert!(aggregator::check_receipt_timestamps(&receipts, Some(&rav)).is_ok());

        // Create rav with max_timestamp equal to the lowest receipt timestamp
        // Aggregation should fail
        let rav = EIP712SignedMessage::new(
            &domain_separator,
            ReceiptAggregateVoucher {
                allocationId: allocation_ids[0],
                timestampNs: receipt_timestamp_range.clone().min().unwrap(),
                valueAggregate: 42,
            },
            &keys.0,
        )
        .unwrap();
        assert!(aggregator::check_receipt_timestamps(&receipts, Some(&rav)).is_err());

        // Create rav with max_timestamp above highest receipt timestamp
        // Aggregation should fail
        let rav = EIP712SignedMessage::new(
            &domain_separator,
            ReceiptAggregateVoucher {
                allocationId: allocation_ids[0],
                timestampNs: receipt_timestamp_range.clone().max().unwrap() + 1,
                valueAggregate: 42,
            },
            &keys.0,
        )
        .unwrap();
        assert!(aggregator::check_receipt_timestamps(&receipts, Some(&rav)).is_err());
    }

    #[rstest]
    #[test]
    /// Test check_allocation_id with 2 receipts that have the correct allocation id
    /// and 1 receipt that has the wrong allocation id
    fn check_allocation_id_fail(
        keys: (PrivateKeySigner, Address),
        allocation_ids: Vec<Address>,
        domain_separator: Eip712Domain,
    ) {
        let receipts = vec![
            EIP712SignedMessage::new(
                &domain_separator,
                Receipt::new(allocation_ids[0], 42).unwrap(),
                &keys.0,
            )
            .unwrap(),
            EIP712SignedMessage::new(
                &domain_separator,
                Receipt::new(allocation_ids[0], 43).unwrap(),
                &keys.0,
            )
            .unwrap(),
            EIP712SignedMessage::new(
                &domain_separator,
                Receipt::new(allocation_ids[1], 44).unwrap(),
                &keys.0,
            )
            .unwrap(),
        ];

        let res = aggregator::check_allocation_id(&receipts, allocation_ids[0]);

        assert!(res.is_err());
    }

    #[rstest]
    #[test]
    /// Test check_allocation_id with 3 receipts that have the correct allocation id
    fn check_allocation_id_ok(
        keys: (PrivateKeySigner, Address),
        allocation_ids: Vec<Address>,
        domain_separator: Eip712Domain,
    ) {
        let receipts = vec![
            EIP712SignedMessage::new(
                &domain_separator,
                Receipt::new(allocation_ids[0], 42).unwrap(),
                &keys.0,
            )
            .unwrap(),
            EIP712SignedMessage::new(
                &domain_separator,
                Receipt::new(allocation_ids[0], 43).unwrap(),
                &keys.0,
            )
            .unwrap(),
            EIP712SignedMessage::new(
                &domain_separator,
                Receipt::new(allocation_ids[0], 44).unwrap(),
                &keys.0,
            )
            .unwrap(),
        ];

        let res = aggregator::check_allocation_id(&receipts, allocation_ids[0]);

        assert!(res.is_ok());
    }
}
