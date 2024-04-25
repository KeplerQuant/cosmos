use cosmrs::proto::cosmos::params::v1beta1::{QueryParamsRequest, QueryParamsResponse};
use cosmrs::proto::cosmos::params::v1beta1::{QuerySubspacesRequest, QuerySubspacesResponse};

use crate::error::CosmosResult;
use crate::{client::CosmosClient, rpc::types::Rpc};

/// Main struct providing access to Params module functions.
#[derive(Debug, Clone)]
pub struct Params<T: Rpc + Clone + Send + Sync> {
    client: CosmosClient<T>,
}

impl<T: Rpc + Clone + Send + Sync> Params<T> {
    /// Creates a new `Params` instance with the provided Cosmos client.
    pub fn new(client: CosmosClient<T>) -> Self {
        Self { client }
    }

    /// Fetches parameters for a given subspace and key.
    pub async fn params(&self, subspace: &str, key: &str) -> CosmosResult<QueryParamsResponse> {
        let query = QueryParamsRequest {
            subspace: subspace.to_string(),
            key: key.to_string(),
        };
        self.client
            .query("/cosmos.params.v1beta1.Query/Params", query)
            .await
    }

    /// Fetches a list of all available subspaces.
    pub async fn subspaces(&self) -> CosmosResult<QuerySubspacesResponse> {
        let query = QuerySubspacesRequest {};
        self.client
            .query("/cosmos.params.v1beta1.Query/Subspaces", query)
            .await
    }
}
