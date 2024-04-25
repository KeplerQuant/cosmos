use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use cosmrs::bip32::secp256k1::elliptic_curve::rand_core::OsRng;
use cosmrs::bip32::{Language, Mnemonic, XPrv};
use cosmrs::crypto::secp256k1::SigningKey;
use cosmrs::crypto::PublicKey;
use cosmrs::tendermint::chain;
use cosmrs::tx::{AccountNumber, Body, SequenceNumber};
use cosmrs::tx::{Fee, SignDoc, SignerInfo};
use cosmrs::{AccountId, Coin, Gas};
use hex::decode;

use crate::error::CosmosResult;

/// Represents a signer with mnemonic, private key, and public key information.
#[derive(Clone)]
pub struct Signer {
    /// Mnemonic phrase (optional).
    pub mnemonic: Option<String>,
    /// Denomination of the cryptocurrency.
    pub denom: String,
    /// Public address associated with the signer.
    pub public_address: AccountId,
    /// Private key for signing transactions.
    pub private_key: Arc<SigningKey>,
    /// Public key associated with the private key.
    pub public_key: PublicKey,
    /// Gas adjustment percentage for transaction fees.
    pub gas_adjustment_percent: u8,
    /// Gas price for transactions.
    pub gas_price: u128,
}

impl Signer {
    /// Loads signer information from a mnemonic phrase, prefix, and optional derivation path.
    fn load_from_mnemonic(
        phrase: &str,
        prefix: &str,
        derivation: Option<&str>,
    ) -> CosmosResult<(SigningKey, PublicKey, AccountId)> {
        let derivation = derivation.unwrap_or("m/44'/118'/0'/0/0");
        let mnemonic = Mnemonic::new(phrase, Language::English)?;
        let pri = XPrv::derive_from_path(&mnemonic.to_seed(""), &derivation.parse()?)?;
        let private_key = SigningKey::from(pri);
        let public_key = private_key.public_key();
        let public_address = public_key.account_id(prefix)?;

        Ok((private_key, public_key, public_address))
    }

    /// Generates a new signer with a random mnemonic phrase.
    pub fn generate_signer(
        prefix: &str,
        denom: &str,
        derivation: Option<&str>,
        gas_adjustment_percent: u8,
        gas_price: u128,
    ) -> CosmosResult<Self> {
        let mnemonic = Mnemonic::random(OsRng, Language::English);
        let (private_key, public_key, public_address) =
            Signer::load_from_mnemonic(mnemonic.phrase(), prefix, derivation)?;

        Ok(Signer {
            mnemonic: Some(mnemonic.phrase().to_string()),
            public_address,
            gas_adjustment_percent,
            gas_price,
            denom: denom.to_string(),
            private_key: Arc::new(private_key),
            public_key,
        })
    }

    /// Creates a signer from a provided private key.
    pub fn from_private_key(
        private_key: &str,
        prefix: &str,
        denom: &str,
        gas_adjustment_percent: u8,
        gas_price: u128,
    ) -> CosmosResult<Self> {
        let private_key = SigningKey::from_slice(decode(private_key)?.as_slice())?;
        let public_key = private_key.public_key();
        let public_address = public_key.account_id(prefix)?;

        Ok(Signer {
            public_address,
            gas_adjustment_percent,
            gas_price,
            public_key,
            mnemonic: None,
            denom: denom.to_string(),
            private_key: Arc::new(private_key),
        })
    }

    /// Creates a signer from a provided mnemonic phrase.
    pub fn from_mnemonic(
        phrase: &str,
        prefix: &str,
        denom: &str,
        derivation: Option<&str>,
        gas_adjustment_percent: u8,
        gas_price: u128,
    ) -> CosmosResult<Self> {
        let (private_key, public_key, public_address) =
            Signer::load_from_mnemonic(phrase, prefix, derivation)?;

        Ok(Signer {
            mnemonic: Some(phrase.to_string()),
            public_address,
            gas_adjustment_percent,
            gas_price,
            public_key,
            denom: denom.to_string(),
            private_key: Arc::new(private_key),
        })
    }

    pub async fn sign(
        &mut self,
        chain_id: &str,
        account_number: AccountNumber,
        sequence_id: SequenceNumber,
        gas_info: Gas,
        body: Body,
    ) -> CosmosResult<Vec<u8>> {
        let auth_info = SignerInfo::single_direct(Some(self.public_key), sequence_id).auth_info(
            Fee::from_amount_and_gas(
                Coin {
                    amount: self.gas_price,
                    denom: self.denom.parse()?,
                },
                gas_info,
            ),
        );

        let sign_doc = SignDoc::new(
            &body,
            &auth_info,
            &chain::Id::from_str(chain_id)?,
            account_number,
        )?;

        Ok(sign_doc.sign(&self.private_key)?.to_bytes()?)
    }
}

impl Debug for Signer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
