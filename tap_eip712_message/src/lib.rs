// Copyright 2023-, Semiotic AI, Inc.
// SPDX-License-Identifier: Apache-2.0

//! # EIP712 message and signature
//!
//! This module contains the `EIP712SignedMessage` struct which is used to sign and verify messages
//! using EIP712 standard.
//!
//! # Example
//! ```rust
//! # use alloy::{dyn_abi::Eip712Domain, primitives::Address, signers::local::PrivateKeySigner};
//! # let domain_separator = Eip712Domain::default();
//! use tap_eip712_message::EIP712SignedMessage;
//! # let wallet = PrivateKeySigner::random();
//! # let wallet_address = wallet.address();
//! # let message = msg::Receipt::new(Address::from([0x11u8; 20]), 100).unwrap();
//!
//! let signed_message = EIP712SignedMessage::new(&domain_separator, message, &wallet).unwrap();
//! let signer = signed_message.recover_signer(&domain_separator).unwrap();
//!
//! assert_eq!(signer, wallet_address);
//! ```
//!

use alloy::{
    dyn_abi::Eip712Domain,
    primitives::{Address, PrimitiveSignature as Signature},
    signers::{local::PrivateKeySigner, SignerSync},
    sol_types::SolStruct,
};
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Eip712Error {
    /// `alloy` wallet error
    #[error(transparent)]
    WalletError(#[from] alloy::signers::Error),

    /// `alloy` wallet error
    #[error(transparent)]
    SignatureError(#[from] alloy::primitives::SignatureError),

    /// Error when signature verification fails
    #[error("Expected address {expected} but received {received}")]
    VerificationFailed {
        expected: Address,
        received: Address,
    },
}

/// EIP712 signed message
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EIP712SignedMessage<M: SolStruct> {
    /// Message to be signed
    pub message: M,
    /// ECDSA Signature of eip712 hash of message
    pub signature: Signature,
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignatureBytes([u8; 65]);

pub trait SignatureBytesExt {
    fn get_signature_bytes(&self) -> SignatureBytes;
}

impl SignatureBytesExt for Signature {
    fn get_signature_bytes(&self) -> SignatureBytes {
        SignatureBytes(self.as_bytes())
    }
}

/// Unique identifier for a message
///
/// This is equal to the hash of the contents of a message, excluding the signature.
/// This means that two receipts signed by two different signers will have the same id.
///
///
/// This cannot be used as a unique identifier for a message, but can be used as a key
/// for a hashmap where the value is the message.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct MessageId(pub [u8; 32]);

impl<M: SolStruct> EIP712SignedMessage<M> {
    /// Creates a signed message with signed EIP712 hash of `message` using `signing_wallet`
    pub fn new(
        domain_separator: &Eip712Domain,
        message: M,
        signing_wallet: &PrivateKeySigner,
    ) -> Result<Self, Eip712Error> {
        let recovery_message_hash = message.eip712_signing_hash(domain_separator);

        let signature = signing_wallet.sign_hash_sync(&recovery_message_hash)?;

        Ok(Self { message, signature })
    }

    /// Recovers and returns the signer of the message from the signature.
    pub fn recover_signer(&self, domain_separator: &Eip712Domain) -> Result<Address, Eip712Error> {
        let recovery_message_hash = self.message.eip712_signing_hash(domain_separator);
        let recovered_address = self
            .signature
            .recover_address_from_prehash(&recovery_message_hash)?;
        Ok(recovered_address)
    }

    /// Checks that receipts signature is valid for given verifying key, returns `Ok` if it is valid.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::SignatureError`] if the recovered address from the
    /// signature is not equal to `expected_address`
    ///
    pub fn verify(
        &self,
        domain_separator: &Eip712Domain,
        expected_address: Address,
    ) -> Result<(), Eip712Error> {
        let recovered_address = self.recover_signer(domain_separator)?;
        if recovered_address != expected_address {
            Err(Eip712Error::VerificationFailed {
                expected: expected_address,
                received: recovered_address,
            })
        } else {
            Ok(())
        }
    }

    /// Use this as a simple key for testing
    pub fn unique_hash(&self) -> MessageId {
        MessageId(self.message.eip712_hash_struct().into())
    }
}
