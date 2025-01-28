// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use alloy::sol_types::SolStruct;
use tap_eip712_message::EIP712SignedMessage;

use crate::{state::Checked, ReceiptWithState};

pub trait Aggregate<T>: SolStruct {
    /// Aggregates a batch of validated receipts with optional validated previous RAV,
    /// returning a new RAV if all provided items are valid or an error if not.
    fn aggregate_receipts(
        receipts: &[ReceiptWithState<Checked, T>],
        previous_rav: Option<EIP712SignedMessage<Self>>,
    ) -> Result<Self, AggregationError>;
}

#[derive(Debug, thiserror::Error)]
pub enum AggregationError {
    /// Error when trying to aggregate receipts and the result overflows
    #[error("Aggregating receipt results in overflow")]
    AggregateOverflow,

    /// Error when no valid receipts are found for a RAV request
    #[error("Failed to produce rav request, no valid receipts")]
    NoValidReceiptsForRavRequest,
}
