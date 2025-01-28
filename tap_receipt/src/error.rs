// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

/// Error type for receipts
#[derive(thiserror::Error, Debug, Clone, Serialize, Deserialize)]
pub enum ReceiptError {
    #[error("invalid allocation ID: {received_allocation_id}")]
    InvalidAllocationID { received_allocation_id: Address },
    #[error("Signature check failed:\n{source_error_message}")]
    InvalidSignature { source_error_message: String },
    #[error("invalid timestamp: {received_timestamp} (expected min {timestamp_min})")]
    InvalidTimestamp {
        received_timestamp: u64,
        timestamp_min: u64,
    },
    #[error("Invalid Value: {received_value} ")]
    InvalidValue { received_value: u128 },
    #[error("Receipt is not unique")]
    NonUniqueReceipt,
    #[error("Attempt to collect escrow failed")]
    SubtractEscrowFailed,
    #[error("Issue encountered while performing check: {0}")]
    CheckFailure(String),
    #[error("Retryable check error encountered: {0}")]
    RetryableCheck(String),
}
