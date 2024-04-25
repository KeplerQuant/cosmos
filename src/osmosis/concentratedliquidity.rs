use osmosis_std::types::cosmos::base::query::v1beta1::PageRequest;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::{
    UserPositionsRequest, UserPositionsResponse,
};

use crate::error::CosmosResult;
use crate::{client::CosmosClient, rpc::types::Rpc};

/// A struct representing a client to interact with the Osmosis Pool Manager.
#[derive(Debug, Clone)]
pub struct ConcentratedLiquidity<T: Rpc + Clone + Send + Sync> {
    client: CosmosClient<T>,
}

impl<T: Rpc + Clone + Send + Sync> ConcentratedLiquidity<T> {
    /// Creates a new PoolManager with the provided CosmosClient.
    pub fn new(client: CosmosClient<T>) -> Self {
        Self { client }
    }

    /// Retrieves the current spot price for a given asset pair from a specific pool.
    ///
    /// # Parameters
    ///
    /// * `pool_id`: A unique identifier for the liquidity pool.
    /// * `base_asset_denom`: The denomination of the base asset.
    /// * `quote_asset_denom`: The denomination of the quote asset.
    ///
    /// # Returns
    ///
    /// A SpotPriceResponse containing the current spot price for the given asset pair.
    pub async fn user_positions(
        &self,
        address: &str,
        pool_id: u64,
        pagination: Option<PageRequest>,
    ) -> CosmosResult<UserPositionsResponse> {
        let query = UserPositionsRequest {
            address: address.to_string(),
            pool_id,
            pagination,
        };

        self.client
            .query(
                "/osmosis.concentratedliquidity.v1beta1.Query/UserPositions",
                query,
            )
            .await
    }
}
