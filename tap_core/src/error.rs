// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Module containing Error type and Result typedef
//!

use std::result::Result as StdResult;

use alloy::primitives::Address;
use thiserror::Error as ThisError;

use crate::receipt::ReceiptError;

/// Error type for the TAP protocol
#[derive(ThisError, Debug)]
pub enum Error {
    /// Error when trying to aggregate receipts and the result overflows
    #[error("Aggregating receipt results in overflow")]
    AggregateOverflow,
    /// Error when Rust fails to get the current system time
    #[error("Failed to get current system time: {source_error_message} ")]
    InvalidSystemTime { source_error_message: String },
    /// `alloy` wallet error
    #[error(transparent)]
    WalletError(#[from] alloy::signers::Error),

    /// `alloy` wallet error
    #[error(transparent)]
    SignatureError(#[from] tap_eip712_message::Eip712Error),

    /// Error when signature verification fails
    #[error("Expected address {expected} but received {received}")]
    VerificationFailed {
        expected: Address,
        received: Address,
    },

    /// Error when the received RAV does not match the expected RAV
    #[error("Received RAV does not match expected RAV")]
    InvalidReceivedRav {
        received_rav: String,
        expected_rav: String,
    },
    /// Generic error from the adapter
    #[error("Error from adapter.\n Caused by: {source_error}")]
    AdapterError { source_error: anyhow::Error },
    /// Error when no valid receipts are found for a RAV request
    #[error("Failed to produce rav request, no valid receipts")]
    NoValidReceiptsForRavRequest,

    /// Error when the previous RAV allocation id does not match the allocation id from the new receipt
    #[error("Previous RAV allocation id ({prev_id}) doesn't match the allocation id from the new receipt ({new_id}).")]
    RavAllocationIdMismatch { prev_id: String, new_id: String },

    /// Error when all receipts do not have the same allocation id
    ///
    /// Used in tap_aggregator
    #[error("All receipts should have the same allocation id, but they don't")]
    RavAllocationIdNotUniform,
    /// Error when the receipt signature is duplicated.
    ///
    /// Used in tap_aggregator
    #[error("Duplicate receipt signature: {0}")]
    DuplicateReceiptSignature(String),
    #[error(
        "Receipt timestamp ({receipt_ts}) is less or equal than previous rav timestamp ({rav_ts})"
    )]
    ReceiptTimestampLowerThanRav { rav_ts: u64, receipt_ts: u64 },

    /// Error when the min timestamp is greater than the max timestamp
    /// Used by [`crate::manager::Manager::create_rav_request()`]
    #[error("Timestamp range error: min_timestamp_ns: {min_timestamp_ns}, max_timestamp_ns: {max_timestamp_ns}. Adjust timestamp buffer.")]
    TimestampRangeError {
        min_timestamp_ns: u64,
        max_timestamp_ns: u64,
    },

    /// Error on the receipt side
    #[error("Receipt error: {0}")]
    ReceiptError(#[from] ReceiptError),

    /// Error when the recovered signer address is invalid
    /// Used by [`crate::manager::adapters::EscrowHandler`]
    #[error("Recovered sender address invalid {address}")]
    InvalidRecoveredSigner { address: Address },

    /// Indicates a failure while verifying the signer
    /// Used by [`crate::manager::adapters::EscrowHandler`]
    #[error("Failed to check the signer: {0}")]
    FailedToVerifySigner(String),
}

pub type Result<T> = StdResult<T, Error>;
