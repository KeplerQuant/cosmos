use cosmrs::proto::cosmos::bank::v1beta1::{
    MsgSend, QueryDenomMetadataRequest, QueryDenomMetadataResponse,
};
use cosmrs::proto::cosmos::base::v1beta1::Coin;
use cosmrs::tx::{Body, BodyBuilder};
use cosmrs::Any;

use crate::client::CosmosClient;
use crate::error::{CosmosResult, Error};
use crate::rpc::types::Rpc;

/// Main struct providing access to Bank module functions.
#[derive(Debug, Clone)]
pub struct Bank<T: Rpc + Clone + Send + Sync> {
    client: CosmosClient<T>,
}

/// Provides functionality for interacting with the bank module on a Cosmos chain.
impl<T: Rpc + Clone + Send + Sync> Bank<T> {
    /// Creates a new `Bank` instance using the provided Cosmos client.
    ///
    /// # Arguments
    ///
    /// * `client`: The Cosmos client to use for interacting with the chain.
    pub fn new(client: CosmosClient<T>) -> Self {
        Self { client }
    }

    /// Sends tokens from the account associated with the attached signer to the specified address.
    ///
    /// # Arguments
    ///
    /// * `to_address`: The address to send the tokens to.
    /// * `amount`: The amount of tokens to send, denominated in various denominations.
    /// * `memo`: An optional memo to attach to the transaction.
    ///
    /// # Returns
    ///
    /// A CosmosResult containing a `Body` representing the constructed transaction, or an error if the
    /// operation fails.
    ///
    /// # Errors
    ///
    /// * Returns a `CosmosResult::Err` with `Error::NoSignerAttached` if no signer is attached to the client.
    /// * Returns a `CosmosResult::Err` with other errors that might occur during transaction construction.
    pub async fn send(
        &self,
        to_address: &str,
        amount: Vec<Coin>,
        memo: Option<&str>,
    ) -> CosmosResult<Body> {
        let signer = self.client.signer().ok_or(Error::NoSignerAttached)?;
        let msg = Any::from_msg(&MsgSend {
            from_address: signer.public_address.to_string(),
            to_address: to_address.to_string(),
            amount,
        })?;
        let mut builder = BodyBuilder::new();
        let mut builder = builder.msg(msg);

        if let Some(memo) = memo {
            builder = builder.memo(memo);
        }

        Ok(builder.finish())
    }

    /// Fetches the metadata of a given token denomination from the Cosmos blockchain.
    ///
    /// # Arguments
    ///
    /// * `denom`: The denomination of the token to fetch metadata for.
    ///
    /// # Returns
    ///
    /// A CosmosResult containing the metadata of the given token denomination, or an error if the
    /// operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let metadata = bank.denom_metadata("atom").await;
    /// ```
    pub async fn denom_metadata(&self, denom: &str) -> CosmosResult<QueryDenomMetadataResponse> {
        let query = QueryDenomMetadataRequest {
            denom: denom.to_string(),
        };

        self.client
            .query("/cosmos.bank.v1beta1.Query/DenomMetadata", query)
            .await
    }
}
