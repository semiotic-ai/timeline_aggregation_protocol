// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::marker::PhantomData;

use alloy::{dyn_abi::Eip712Domain, sol_types::SolStruct};
use tap_receipt::rav::Aggregate;

use super::adapters::{
    RavRead, RavStore, ReceiptDelete, ReceiptRead, ReceiptStore, SignatureChecker,
};
use crate::{
    rav_request::RavRequest,
    receipt::{
        checks::{CheckBatch, CheckList, TimestampCheck, UniqueCheck},
        state::{Checked, Failed},
        Context, ReceiptError, ReceiptWithState, WithUniqueId, WithValueAndTimestamp,
    },
    signed_message::EIP712SignedMessage,
    Error,
};

pub struct Manager<E, Rcpt, Rav> {
    /// Context that implements adapters
    context: E,

    /// Checks that must be completed for each receipt before being confirmed or denied for rav request
    checks: CheckList<Rcpt>,

    /// Struct responsible for doing checks for receipt. Ownership stays with manager allowing manager
    /// to update configuration ( like minimum timestamp ).
    domain_separator: Eip712Domain,

    _phantom: PhantomData<Rav>,
}

impl<E, Rcpt, Rav> Manager<E, Rcpt, Rav> {
    /// Creates new manager with provided `adapters`, any receipts received by this manager
    /// will complete all `required_checks` before being accepted or declined from RAV.
    /// `starting_min_timestamp` will be used as min timestamp until the first RAV request is created.
    ///
    pub fn new(
        domain_separator: Eip712Domain,
        context: E,
        checks: impl Into<CheckList<Rcpt>>,
    ) -> Self {
        Self {
            context,
            domain_separator,
            checks: checks.into(),
            _phantom: PhantomData,
        }
    }
}

impl<E, Rcpt, Rav> Manager<E, Rcpt, Rav>
where
    E: RavStore<Rav> + SignatureChecker,
    Rav: SolStruct + PartialEq<Rav> + Sync + std::fmt::Debug,
{
    /// Verify `signed_rav` matches all values on `expected_rav`, and that `signed_rav` has a valid signer.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AdapterError`] if there are any errors while storing RAV
    ///
    pub async fn verify_and_store_rav(
        &self,
        expected_rav: Rav,
        signed_rav: EIP712SignedMessage<Rav>,
    ) -> std::result::Result<(), Error> {
        self.context
            .check_signature(&signed_rav, &self.domain_separator)
            .await?;

        if signed_rav.message != expected_rav {
            return Err(Error::InvalidReceivedRav {
                received_rav: format!("{:?}", signed_rav.message),
                expected_rav: format!("{:?}", expected_rav),
            });
        }

        self.context
            .update_last_rav(signed_rav)
            .await
            .map_err(|err| Error::AdapterError {
                source_error: anyhow::Error::new(err),
            })?;

        Ok(())
    }
}

impl<E, Rcpt, Rav> Manager<E, Rcpt, Rav>
where
    E: RavRead<Rav>,
    Rav: SolStruct,
{
    async fn get_previous_rav(&self) -> Result<Option<EIP712SignedMessage<Rav>>, Error> {
        let previous_rav = self
            .context
            .last_rav()
            .await
            .map_err(|err| Error::AdapterError {
                source_error: anyhow::Error::new(err),
            })?;
        Ok(previous_rav)
    }
}

impl<E, Rcpt, Rav> Manager<E, Rcpt, Rav>
where
    E: ReceiptRead<Rcpt> + SignatureChecker,
    Rcpt: WithUniqueId + WithValueAndTimestamp + Sync,
{
    async fn collect_receipts(
        &self,
        ctx: &Context,
        timestamp_buffer_ns: u64,
        min_timestamp_ns: u64,
        limit: Option<u64>,
    ) -> Result<
        (
            Vec<ReceiptWithState<Checked, Rcpt>>,
            Vec<ReceiptWithState<Failed, Rcpt>>,
        ),
        Error,
    > {
        let max_timestamp_ns = crate::get_current_timestamp_u64_ns()? - timestamp_buffer_ns;

        if min_timestamp_ns > max_timestamp_ns {
            return Err(Error::TimestampRangeError {
                min_timestamp_ns,
                max_timestamp_ns,
            });
        }
        let checking_receipts = self
            .context
            .retrieve_receipts_in_timestamp_range(min_timestamp_ns..max_timestamp_ns, limit)
            .await
            .map_err(|err| Error::AdapterError {
                source_error: anyhow::Error::new(err),
            })?;

        let mut checked_receipts = vec![];
        let mut failed_receipts = vec![];

        // check for timestamp
        let (checking_receipts, already_failed) =
            TimestampCheck(min_timestamp_ns).check_batch(checking_receipts);
        failed_receipts.extend(already_failed);

        // check for uniqueness
        let (checking_receipts, already_failed) = UniqueCheck.check_batch(checking_receipts);
        failed_receipts.extend(already_failed);

        for receipt in checking_receipts.into_iter() {
            let receipt = receipt
                .finalize_receipt_checks(ctx, &self.checks)
                .await
                .map_err(|e| Error::ReceiptError(ReceiptError::RetryableCheck(e)))?;

            match receipt {
                Ok(checked) => checked_receipts.push(checked),
                Err(failed) => failed_receipts.push(failed),
            }
        }

        Ok((checked_receipts, failed_receipts))
    }
}

impl<E, Rcpt, Rav> Manager<E, Rcpt, Rav>
where
    E: ReceiptRead<Rcpt> + RavRead<Rav> + SignatureChecker,
    Rav: SolStruct + WithValueAndTimestamp + Clone + Aggregate<Rcpt>,
    Rcpt: WithUniqueId + WithValueAndTimestamp + Sync,
{
    /// Completes remaining checks on all receipts up to
    /// (current time - `timestamp_buffer_ns`). Returns them in two lists
    /// (valid receipts and invalid receipts) along with the expected RAV that
    /// should be received for aggregating list of valid receipts.
    ///
    /// Returns [`Error::AggregateOverflow`] if any receipt value causes
    /// aggregate value to overflow while generating expected RAV
    ///
    /// Returns [`Error::AdapterError`] if unable to fetch previous RAV or
    /// if unable to fetch previous receipts
    ///
    /// Returns [`Error::TimestampRangeError`] if the max timestamp of the
    /// previous RAV is greater than the min timestamp. Caused by timestamp
    /// buffer being too large, or requests coming too soon.
    ///
    pub async fn create_rav_request(
        &self,
        ctx: &Context,
        timestamp_buffer_ns: u64,
        receipts_limit: Option<u64>,
    ) -> Result<RavRequest<Rcpt, Rav>, Error> {
        let previous_rav = self.get_previous_rav().await?;
        let min_timestamp_ns = previous_rav
            .as_ref()
            .map(|rav| rav.message.timestamp_ns() + 1)
            .unwrap_or(0);

        let (valid_receipts, invalid_receipts) = self
            .collect_receipts(ctx, timestamp_buffer_ns, min_timestamp_ns, receipts_limit)
            .await?;

        let expected_rav = Rav::aggregate_receipts(&valid_receipts, previous_rav.clone());

        Ok(RavRequest {
            valid_receipts,
            previous_rav,
            invalid_receipts,
            expected_rav,
        })
    }
}

impl<E, Rcpt, Rav> Manager<E, Rcpt, Rav>
where
    E: ReceiptDelete + RavRead<Rav>,
    Rav: SolStruct + WithValueAndTimestamp,
{
    /// Removes obsolete receipts from storage. Obsolete receipts are receipts
    /// that are older than the last RAV, and therefore already aggregated into the RAV.
    /// This function should be called after a new RAV is received to limit the
    /// number of receipts stored. No-op if there is no last RAV.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AdapterError`] if there are any errors while retrieving
    /// last RAV or removing receipts
    ///
    pub async fn remove_obsolete_receipts(&self) -> Result<(), Error> {
        match self.get_previous_rav().await? {
            Some(last_rav) => {
                self.context
                    .remove_receipts_in_timestamp_range(..=last_rav.message.timestamp_ns())
                    .await
                    .map_err(|err| Error::AdapterError {
                        source_error: anyhow::Error::new(err),
                    })?;
                Ok(())
            }
            None => Ok(()),
        }
    }
}

impl<E, Rcpt, Rav> Manager<E, Rcpt, Rav>
where
    E: ReceiptStore<Rcpt>,
{
    /// Runs `initial_checks` on `signed_receipt` for initial verification,
    /// then stores received receipt.
    /// The provided `query_id` will be used as a key when chaecking query appraisal.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AdapterError`] if there are any errors while storing receipts
    ///
    pub async fn verify_and_store_receipt(
        &self,
        ctx: &Context,
        signed_receipt: Rcpt,
    ) -> std::result::Result<(), Error> {
        let mut received_receipt = ReceiptWithState::new(signed_receipt);

        // perform checks
        received_receipt.perform_checks(ctx, &self.checks).await?;

        // store the receipt
        self.context
            .store_receipt(received_receipt)
            .await
            .map_err(|err| Error::AdapterError {
                source_error: anyhow::Error::new(err),
            })?;
        Ok(())
    }
}
