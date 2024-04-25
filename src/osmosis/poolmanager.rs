use osmosis_std::types::osmosis::poolmanager::v1beta1::{
    EstimateSwapExactAmountInRequest, EstimateSwapExactAmountInResponse, PoolRequest, PoolResponse,
    SpotPriceRequest, SwapAmountInRoute,
};
use osmosis_std::types::osmosis::poolmanager::v2::SpotPriceResponse;

use crate::error::CosmosResult;
use crate::{client::CosmosClient, rpc::types::Rpc};

/// A struct representing a client to interact with the Osmosis Pool Manager.
#[derive(Debug, Clone)]
pub struct PoolManager<T: Rpc + Clone + Send + Sync> {
    client: CosmosClient<T>,
}

impl<T: Rpc + Clone + Send + Sync> PoolManager<T> {
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
    pub async fn spot_price(
        &self,
        pool_id: u64,
        base_asset_denom: &str,
        quote_asset_denom: &str,
    ) -> CosmosResult<SpotPriceResponse> {
        let query = SpotPriceRequest {
            pool_id,
            base_asset_denom: base_asset_denom.to_owned(),
            quote_asset_denom: quote_asset_denom.to_owned(),
        };
        self.client
            .query("/osmosis.poolmanager.v1beta1.Query/SpotPrice", query)
            .await
    }

    /// Estimates the output amount for a given input amount of a specific asset pair.
    ///
    /// # Parameters
    ///
    /// * `pool_id`: A unique identifier for the liquidity pool.
    /// * `token_in_amount`: The amount of the input asset.
    /// * `token_in_denom`: The denomination of the input asset.
    /// * `routes`: A list of routes for the asset swap.
    ///
    /// # Returns
    ///
    /// An EstimateSwapExactAmountInResponse containing the estimated output amount for the given input amount.
    pub async fn estimate_swap_exact_amount_in(
        &self,
        pool_id: u64,
        token_in_amount: f64,
        token_in_denom: &str,
        routes: Vec<SwapAmountInRoute>,
    ) -> CosmosResult<EstimateSwapExactAmountInResponse> {
        #[allow(deprecated)]
        let query = EstimateSwapExactAmountInRequest {
            pool_id,
            routes,
            token_in: format!("{}{}", token_in_amount, token_in_denom),
        };
        self.client
            .query(
                "/osmosis.gamm.v1beta1.Query/EstimateSwapExactAmountIn",
                query,
            )
            .await
    }

    /// Retrieves information about a specific pool.
    ///
    /// # Arguments
    ///
    /// * `pool_id` - The ID of the pool to get the information from.
    ///
    /// # Returns
    ///
    /// * A CosmosResult containing a PoolResponse with the pool information, or an error if the request failed.
    pub async fn pool(&self, pool_id: u64) -> CosmosResult<PoolResponse> {
        let query = PoolRequest { pool_id };
        self.client
            .query("/osmosis.poolmanager.v1beta1.Query/Pool", query)
            .await
    }
}
