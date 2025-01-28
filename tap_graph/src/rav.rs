// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Receipt Aggregate Voucher
//!
//! Receipts Aggregate Voucher or RAV is the struct that is sent to the
//! blockchain to redeem the aggregated receipts. Receipts are aggregated
//! into a single RAV via a [`RAVRequest`] and then sent to `tap_aggregator`.
//! The request is verified and signed by the aggregator and the response
//! is stored on the indexer side.
//!
//! Every time you have enough receipts to aggregate, you can send another
//! RAV request to the aggregator. The aggregator will verify the request and
//! increment the total amount that has been aggregated.
//!
//! Once the allocation is closed or anytime the user doesn't want to serve
//! anymore(sender considered malicious, not enough in escrow to cover the RAV, etc),
//! the user can redeem the RAV on the blockchain and get the aggregated amount.
//!
//! The system is considered to have minimal trust because you only need to trust
//! the sender until you receive the RAV. The value of non-aggregated receipts must
//! be less than the value you are willing to lose if the sender is malicious.
//!
//! # How to send a request to the aggregator
//!
//! 1. Create a [`RAVRequest`] with the valid receipts and the previous RAV.
//! 2. Send the request to the aggregator.
//! 3. The aggregator will verify the request and increment the total amount that
//!    has been aggregated.
//! 4. The aggregator will return a [`SignedRAV`].
//! 5. Store the [`SignedRAV`].
//! 6. Repeat the process until the allocation is closed.
//! 7. Redeem the RAV on the blockchain and get the aggregated amount.
//!
//! # How to create RAV Requests
//!
//! Rav requests should be created using the
//! [`crate::manager::Manager::create_rav_request`] function.

use std::cmp;

use alloy::{primitives::Address, sol};
use serde::{Deserialize, Serialize};
use tap_eip712_message::EIP712SignedMessage;
use tap_receipt::{
    rav::{Aggregate, AggregationError},
    state::Checked,
    ReceiptWithState, WithValueAndTimestamp,
};

use crate::{receipt::Receipt, SignedReceipt};

/// EIP712 signed message for ReceiptAggregateVoucher
pub type SignedRav = EIP712SignedMessage<ReceiptAggregateVoucher>;

sol! {
    /// Holds information needed for promise of payment signed with ECDSA
    ///
    /// We use camelCase for field names to match the Ethereum ABI encoding
    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct ReceiptAggregateVoucher {
        /// Unique allocation id this RAV belongs to
        address allocationId;
        /// Unix Epoch timestamp in nanoseconds (Truncated to 64-bits)
        /// corresponding to max timestamp from receipt batch aggregated
        uint64 timestampNs;
        /// Aggregated value from receipt batch and any previous RAV provided
        /// (truncate to lower bits)
        uint128 valueAggregate;
    }
}

impl ReceiptAggregateVoucher {
    /// Aggregates a batch of validated receipts with optional validated previous RAV,
    /// returning a new RAV if all provided items are valid or an error if not.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AggregateOverflow`] if any receipt value causes aggregate
    /// value to overflow
    pub fn aggregate_receipts(
        allocation_id: Address,
        receipts: &[EIP712SignedMessage<Receipt>],
        previous_rav: Option<EIP712SignedMessage<Self>>,
    ) -> Result<Self, AggregationError> {
        //TODO(#29): When receipts in flight struct in created check that the state
        // of every receipt is OK with all checks complete (relies on #28)
        // If there is a previous RAV get initialize values from it, otherwise get default values
        let mut timestamp_max = 0u64;
        let mut value_aggregate = 0u128;

        if let Some(prev_rav) = previous_rav {
            timestamp_max = prev_rav.message.timestampNs;
            value_aggregate = prev_rav.message.valueAggregate;
        }

        for receipt in receipts {
            value_aggregate = value_aggregate
                .checked_add(receipt.message.value)
                .ok_or(AggregationError::AggregateOverflow)?;

            timestamp_max = cmp::max(timestamp_max, receipt.message.timestamp_ns)
        }

        Ok(Self {
            allocationId: allocation_id,
            timestampNs: timestamp_max,
            valueAggregate: value_aggregate,
        })
    }
}

impl Aggregate<SignedReceipt> for ReceiptAggregateVoucher {
    fn aggregate_receipts(
        receipts: &[ReceiptWithState<Checked, SignedReceipt>],
        previous_rav: Option<EIP712SignedMessage<Self>>,
    ) -> Result<Self, AggregationError> {
        if receipts.is_empty() {
            return Err(AggregationError::NoValidReceiptsForRavRequest);
        }
        let allocation_id = receipts[0].signed_receipt().message.allocation_id;
        let receipts = receipts
            .iter()
            .map(|rx_receipt| rx_receipt.signed_receipt().clone())
            .collect::<Vec<_>>();
        ReceiptAggregateVoucher::aggregate_receipts(
            allocation_id,
            receipts.as_slice(),
            previous_rav,
        )
    }
}

impl WithValueAndTimestamp for ReceiptAggregateVoucher {
    fn value(&self) -> u128 {
        self.valueAggregate
    }

    fn timestamp_ns(&self) -> u64 {
        self.timestampNs
    }
}
