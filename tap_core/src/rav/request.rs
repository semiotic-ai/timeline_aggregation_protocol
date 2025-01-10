// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{
    rav::{ReceiptAggregateVoucher, SignedRAV},
    receipt::{
        state::{Failed, Reserved},
        ReceiptWithState,
    },
    Error,
};

/// Request to `tap_aggregator` to aggregate receipts into a Signed RAV.
#[derive(Debug)]
pub struct RAVRequest {
    /// List of checked and reserved receipts to aggregate
    pub valid_receipts: Vec<ReceiptWithState<Reserved>>,
    /// Optional previous RAV to aggregate with
    pub previous_rav: Option<SignedRAV>,
    /// List of failed receipt used to log invalid receipts
    pub invalid_receipts: Vec<ReceiptWithState<Failed>>,
    /// Expected RAV to be created
    pub expected_rav: Result<ReceiptAggregateVoucher, Error>,
}
