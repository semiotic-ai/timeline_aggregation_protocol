// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Receipts states and checks
//!
//! Receipts are used as single transaction promise of payment. A payment sender
//! creates a receipt and ECDSA signs it, then sends it to a payment receiver.
//! The payment receiver would verify the received receipt and store it to be
//! accumulated with other received receipts in the future.
//!
//! A list of checks are performed on the received receipts to ensure they are valid.
//! The checks are performed when storing the receipt and when aggregating the receipts
//! into a Receipt Aggregate Voucher (RAV).
//!
//! Each receipt is wrapped into a State Machine that can be in one of the following states:
//! - `Checking`: The receipt is being checked.
//! - `Failed`: The receipt has failed a check or validation.
//! - `AwaitingReserve`: The receipt has passed all checks and is awaiting escrow reservation.
//! - `Reserved`: The receipt has successfully reserved escrow.
//!
//!
pub mod checks;
mod error;
pub mod rav;
mod received_receipt;
pub mod state;

use alloy::sol_types::SolStruct;
pub use error::ReceiptError;
pub use received_receipt::ReceiptWithState;
use tap_eip712_message::{EIP712SignedMessage, SignatureBytes, SignatureBytesExt};

/// Result type for receipt
pub type ReceiptResult<T> = Result<T, ReceiptError>;

pub type Context = anymap3::Map<dyn std::any::Any + Send + Sync>;

pub trait WithValueAndTimestamp {
    fn value(&self) -> u128;
    fn timestamp_ns(&self) -> u64;
}

pub trait WithUniqueId {
    type Output: Eq + std::hash::Hash;
    fn unique_id(&self) -> Self::Output;
}

impl<T> WithValueAndTimestamp for EIP712SignedMessage<T>
where
    T: SolStruct + WithValueAndTimestamp,
{
    fn value(&self) -> u128 {
        self.message.value()
    }

    fn timestamp_ns(&self) -> u64 {
        self.message.timestamp_ns()
    }
}

impl<T> WithUniqueId for EIP712SignedMessage<T>
where
    T: SolStruct,
{
    type Output = SignatureBytes;

    fn unique_id(&self) -> Self::Output {
        self.signature.get_signature_bytes()
    }
}
