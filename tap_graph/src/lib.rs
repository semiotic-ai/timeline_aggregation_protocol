// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::time::{SystemTime, UNIX_EPOCH};

mod rav;
mod receipt;

pub use rav::{ReceiptAggregateVoucher, SignedRav};
pub use receipt::{Receipt, SignedReceipt};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error when Rust fails to get the current system time
    #[error("Failed to get current system time: {source_error_message} ")]
    InvalidSystemTime { source_error_message: String },
}

fn get_current_timestamp_u64_ns() -> Result<u64, Error> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| Error::InvalidSystemTime {
            source_error_message: err.to_string(),
        })?
        .as_nanos() as u64)
}
