// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Module containing Receipt type used for providing and verifying a payment
//!
//! Receipts are used as single transaction promise of payment. A payment sender
//! creates a receipt and ECDSA signs it, then sends it to a payment receiver.
//! The payment receiver would verify the received receipt and store it to be
//! accumulated with other received receipts in the future.

use alloy::{primitives::Address, sol};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tap_eip712_message::EIP712SignedMessage;
use tap_receipt::WithValueAndTimestamp;

/// A signed receipt message
pub type SignedReceipt = EIP712SignedMessage<Receipt>;

sol! {
    /// Holds information needed for promise of payment signed with ECDSA
    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct Receipt {
        /// Unique allocation id this receipt belongs to
        address allocation_id;
        /// Unix Epoch timestamp in nanoseconds (Truncated to 64-bits)
        uint64 timestamp_ns;
        /// Random value used to avoid collisions from multiple receipts with one timestamp
        uint64 nonce;
        /// GRT value for transaction (truncate to lower bits)
        uint128 value;
    }
}

impl Receipt {
    /// Returns a receipt with provided values
    pub fn new(allocation_id: Address, value: u128) -> Result<Self, crate::Error> {
        let timestamp_ns = crate::get_current_timestamp_u64_ns()?;
        let nonce = thread_rng().gen::<u64>();
        Ok(Self {
            allocation_id,
            timestamp_ns,
            nonce,
            value,
        })
    }
}

impl WithValueAndTimestamp for Receipt {
    fn value(&self) -> u128 {
        self.value
    }

    fn timestamp_ns(&self) -> u64 {
        self.timestamp_ns
    }
}

#[cfg(test)]
mod receipt_unit_test {
    use std::{
        str::FromStr,
        time::{SystemTime, UNIX_EPOCH},
    };

    use rstest::*;

    use super::*;

    #[fixture]
    fn allocation_ids() -> Vec<Address> {
        vec![
            Address::from_str("0xabababababababababababababababababababab").unwrap(),
            Address::from_str("0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddead").unwrap(),
            Address::from_str("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef").unwrap(),
            Address::from_str("0x1234567890abcdef1234567890abcdef12345678").unwrap(),
        ]
    }

    #[rstest]
    fn test_new_receipt(allocation_ids: Vec<Address>) {
        let value = 1234;

        let receipt = Receipt::new(allocation_ids[0], value).unwrap();

        assert_eq!(receipt.allocation_id, allocation_ids[0]);
        assert_eq!(receipt.value, value);

        // Check that the timestamp is within a reasonable range
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current system time should be greater than `UNIX_EPOCH`")
            .as_nanos() as u64;
        assert!(receipt.timestamp_ns <= now);
        assert!(receipt.timestamp_ns >= now - 5000000); // 5 second tolerance
    }

    #[rstest]
    fn test_unique_nonce_and_timestamp(allocation_ids: Vec<Address>) {
        let value = 1234;

        let receipt1 = Receipt::new(allocation_ids[0], value).unwrap();
        let receipt2 = Receipt::new(allocation_ids[0], value).unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Current system time should be greater than `UNIX_EPOCH`")
            .as_nanos() as u64;

        // Check that nonces are different
        // Note: This test has an *extremely low* (~1/2^64) probability of false failure, if a failure happens
        //       once it is not neccessarily a sign of an issue. If this test fails more than once, especially
        //       in a short period of time (within a ) then there may be an issue with randomness
        //       of the nonce generation.
        assert_ne!(receipt1.nonce, receipt2.nonce);

        assert!(receipt1.timestamp_ns <= now);
        assert!(receipt1.timestamp_ns >= now - 5000000); // 5 second tolerance

        assert!(receipt2.timestamp_ns <= now);
        assert!(receipt2.timestamp_ns >= now - 5000000); // 5 second tolerance
    }
}
