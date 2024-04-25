pub use cosmrs::proto::cosmos::auth::v1beta1::{
    QueryAccountRequest, QueryAccountResponse, QueryAccountsRequest, QueryAccountsResponse,
    QueryParamsRequest, QueryParamsResponse,
};
pub use cosmrs::proto::cosmos::base::query::v1beta1::PageRequest;

use crate::client::CosmosClient;
use crate::error::CosmosResult;
use crate::rpc::types::Rpc;

/// Main struct providing access to Auth module functions.
#[derive(Debug, Clone)]
pub struct Auth<T: Rpc + Clone + Send + Sync> {
    client: CosmosClient<T>,
}

impl<T: Rpc + Clone + Send + Sync> Auth<T> {
    /// Creates a new `Auth` instance with the provided Cosmos client.
    pub fn new(client: CosmosClient<T>) -> Self {
        Self { client }
    }

    /// Fetches a list of accounts with optional pagination.
    pub async fn accounts(
        &self,
        pagination: Option<PageRequest>,
    ) -> CosmosResult<QueryAccountsResponse> {
        let query = QueryAccountsRequest { pagination };
        self.client
            .query("/cosmos.auth.v1beta1.Query/Accounts", query)
            .await
    }

    /// Fetches information about a specific account.
    pub async fn account(&self, address: &str) -> CosmosResult<QueryAccountResponse> {
        let query = QueryAccountRequest {
            address: address.to_string(),
        };
        self.client
            .query("/cosmos.auth.v1beta1.Query/Account", query)
            .await
    }

    /// Fetches the Auth module parameters.
    pub async fn params(&self) -> CosmosResult<QueryParamsResponse> {
        let query = QueryParamsRequest {};
        self.client
            .query("/cosmos.auth.v1beta1.Query/Params", query)
            .await
    }
}
