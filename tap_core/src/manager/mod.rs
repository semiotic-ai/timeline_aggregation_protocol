// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Point of entry for managing TAP receipts and RAVs.
//!
//! The [`crate::manager`] module provides facilities for managing TAP receipt
//! and RAV validation, as well as storage flow.
//!
//! This module should be the primary interface for the receiver of funds to
//! verify, store, and manage TAP receipts and RAVs.
//! The [`Manager`] struct within this module allows the user to specify what
//! checks should be performed on receipts, as well as
//! when these checks should occur (either when a receipt is first received,
//! or when it is being added to a RAV request).
//!
//! The [`Manager`] uses a context that implements user-defined [`adapters`]
//! for storage handling.
//! This design offers a high degree of flexibility, letting the user define
//! their own behavior for these critical operations.
//!
//! This solution is flexible enough to enable certain methods depending on the
//! context provided. This is important because the context can be customized
//! to include only the necessary adapters for the user's specific use case.
//! For example, if the user wants to use two different applications, one to
//! handle receipt storage and another to handle RAV storage, they
//! can create two different contexts with the appropriate adapters.
//!
//! # Adapters
//! There are 6 main adapters that can be implemented to customize the behavior
//! of the [`Manager`].
//! You can find more information about these adapters in the [`adapters`] module.
//!
//! # Example
//!
//! ```rust
//! use async_trait::async_trait;
//! use tap_core::{
//!     receipt::{
//!         ReceiptWithState,
//!         state::Checking,
//!         checks::CheckList,
//!         ReceiptError,
//!         Context
//!     },
//!     manager::{
//!         Manager,
//!         adapters::ReceiptStore
//!     },
//! };
//!
//! struct MyContext;
//!
//! #[async_trait]
//! impl<T: Send + 'static> ReceiptStore<T> for MyContext {
//!     type AdapterError = ReceiptError;
//!
//!     async fn store_receipt(&self, receipt: ReceiptWithState<Checking, T>) -> Result<u64, Self::AdapterError> {
//!         // ...
//!         # Ok(0)
//!     }
//! }
//! # #[tokio::main]
//! # async fn main() {
//! # use alloy::{dyn_abi::Eip712Domain, primitives::Address, signers::local::PrivateKeySigner};
//! # use tap_graph::{Receipt, SignedReceipt, ReceiptAggregateVoucher};
//! # use tap_core::signed_message::EIP712SignedMessage;
//! # let domain_separator = Eip712Domain::default();
//! # let wallet = PrivateKeySigner::random();
//! # let message = Receipt::new(Address::from([0x11u8; 20]), 100).unwrap();
//!
//! let receipt = EIP712SignedMessage::new(&domain_separator, message, &wallet).unwrap();
//!
//! let manager = Manager::<_, _, ReceiptAggregateVoucher>::new(domain_separator, MyContext, CheckList::empty());
//! manager.verify_and_store_receipt(&Context::new(), receipt).await.unwrap()
//! # }
//! ```
//!

pub mod adapters;
#[cfg(feature = "in_memory")]
pub mod context;
mod tap_manager;

pub use tap_manager::Manager;
