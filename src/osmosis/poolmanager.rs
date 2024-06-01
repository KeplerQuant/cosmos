use cosmrs::proto::prost::Message;
use osmosis_std::types::osmosis::concentratedliquidity::v1beta1::Pool as ConcentratedAmm;
use osmosis_std::types::osmosis::cosmwasmpool::v1beta1::CosmWasmPool;
use osmosis_std::types::osmosis::gamm::v1beta1::Pool as Amm;
use osmosis_std::types::osmosis::poolmanager::v1beta1::{
    PoolRequest, PoolResponse, SpotPriceRequest,
};
use osmosis_std::types::osmosis::poolmanager::v2::SpotPriceResponse;

use crate::error::{CosmosResult, Error};
use crate::{client::CosmosClient, rpc::types::Rpc};

/// An enumeration representing different types of liquidity pools.
#[derive(Debug, Clone)]
pub enum Pool {
    Amm(Amm),
    ConcentratedAmm(ConcentratedAmm),
    CosmWasmPool(CosmWasmPool),
}

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

    /// Retrieves information about a specific pool.
    ///
    /// # Arguments
    ///
    /// * `pool_id` - The ID of the pool to get the information from.
    ///
    /// # Returns
    ///
    /// * A CosmosResult containing a PoolResponse with the pool information, or an error if the request failed.
    pub async fn pool(&self, pool_id: u64) -> CosmosResult<Pool> {
        let query = PoolRequest { pool_id };
        let resp: PoolResponse = self
            .client
            .query("/osmosis.poolmanager.v1beta1.Query/Pool", query)
            .await?;

        let pool = resp.pool.ok_or(Error::NotFoundPool)?;
        match pool.type_url.as_str() {
            "/osmosis.concentratedliquidity.v1beta1.Pool" => {
                let pool = ConcentratedAmm::decode(pool.value.as_slice())?;
                return Ok(Pool::ConcentratedAmm(pool));
            }
            "/osmosis.gamm.v1beta1.Pool" => {
                let pool = Amm::decode(pool.value.as_slice())?;
                return Ok(Pool::Amm(pool));
            }
            "/osmosis.cosmwasmpool.v1beta1.CosmWasmPool" => {
                let pool = CosmWasmPool::decode(pool.value.as_slice())?;
                return Ok(Pool::CosmWasmPool(pool));
            }
            _ => unimplemented!(),
        }
    }
}
