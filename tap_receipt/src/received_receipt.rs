// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Module containing Receipt with metadata for tracking receipt throught its lifecycle
//!
//! This module contains the `ReceivedReceipt` struct and associated enums `ReceiptState` and
//! `RAVStatus`. The `ReceivedReceipt` struct is a wrapper class for a signed receipt, which
//! includes metadata and state information for tracking the progress of a received receipt
//! throughout its lifecycle. `ReceiptState` represents the different states a receipt can be in,
//! while `RAVStatus` defines the status of a receipt with respect to its inclusion in RAV requests
//! and received RAVs.
//!
//! This module is useful for managing and tracking the state of received receipts, as well as
//! their progress through various checks and stages of inclusion in RAV requests and received RAVs.

use super::{checks::CheckError, Context, ReceiptError, ReceiptResult};
use crate::{
    checks::ReceiptCheck,
    state::{Checked, Checking, Failed, ReceiptState},
};

pub type ResultReceipt<S, Rcpt> =
    std::result::Result<ReceiptWithState<S, Rcpt>, ReceiptWithState<Failed, Rcpt>>;

/// Typestate pattern for tracking the state of a receipt
///
/// - The [ `ReceiptState` ] trait represents the different states a receipt
///   can be in.
/// - The [ `Checking` ] state is used to represent a receipt that is currently
///   being checked.
/// - The [ `Failed` ] state is used to represent a receipt that has failed a
///   check or validation.
/// - The [ `AwaitingReserve` ] state is used to represent a receipt that has
///   passed all checks and is awaiting escrow reservation.
/// - The [ `Reserved` ] state is used to represent a receipt that has
///   successfully reserved escrow.
#[derive(Debug, Clone)]
pub struct ReceiptWithState<S, Rcpt>
where
    S: ReceiptState,
{
    /// An EIP712 signed receipt message
    pub(crate) receipt: Rcpt,
    /// The current state of the receipt (e.g., received, checking, failed, accepted, etc.)
    pub(crate) _state: S,
}

impl<Rcpt> ReceiptWithState<Checking, Rcpt> {
    /// Creates a new `ReceiptWithState` in the `Checking` state
    pub fn new(receipt: Rcpt) -> ReceiptWithState<Checking, Rcpt> {
        ReceiptWithState {
            receipt,
            _state: Checking,
        }
    }

    /// Performs a list of checks on the receipt
    ///
    /// # Errors
    ///
    /// Returns [`ReceiptError::CheckFailedToComplete`] if the requested check
    /// cannot be comleted in the receipts current internal state.
    /// All other checks must be complete before `CheckAndReserveEscrow`.
    ///
    pub async fn perform_checks(
        &mut self,
        ctx: &Context,
        checks: &[ReceiptCheck<Rcpt>],
    ) -> ReceiptResult<()> {
        for check in checks {
            // return early on an error
            check.check(ctx, self).await.map_err(|e| match e {
                CheckError::Retryable(e) => ReceiptError::RetryableCheck(e.to_string()),
                CheckError::Failed(e) => ReceiptError::CheckFailure(e.to_string()),
            })?;
        }
        Ok(())
    }

    /// Completes all checks and transitions the receipt to the next state
    ///
    /// Returns `Err` with a [`ReceiptWithState<Failed>`] in case of error,
    /// returns `Ok` with a [`ReceiptWithState<AwaitingReserve>`] in case of success.
    ///
    pub async fn finalize_receipt_checks(
        mut self,
        ctx: &Context,
        checks: &[ReceiptCheck<Rcpt>],
    ) -> Result<ResultReceipt<Checked, Rcpt>, String> {
        let all_checks_passed = self.perform_checks(ctx, checks).await;
        if let Err(ReceiptError::RetryableCheck(e)) = all_checks_passed {
            Err(e.to_string())
        } else if let Err(e) = all_checks_passed {
            Ok(Err(self.perform_state_error(e)))
        } else {
            let checked = self.perform_state_changes(Checked);
            Ok(Ok(checked))
        }
    }
}

impl<Rcpt> ReceiptWithState<Failed, Rcpt> {
    pub fn error(self) -> ReceiptError {
        self._state.error
    }
}

impl<S, Rcpt> ReceiptWithState<S, Rcpt>
where
    S: ReceiptState,
{
    pub(super) fn perform_state_error(self, error: ReceiptError) -> ReceiptWithState<Failed, Rcpt> {
        ReceiptWithState {
            receipt: self.receipt,
            _state: Failed { error },
        }
    }

    fn perform_state_changes<T>(self, new_state: T) -> ReceiptWithState<T, Rcpt>
    where
        T: ReceiptState,
    {
        ReceiptWithState {
            receipt: self.receipt,
            _state: new_state,
        }
    }

    /// Returns the signed receipt
    pub fn signed_receipt(&self) -> &Rcpt {
        &self.receipt
    }
}
