use async_trait::async_trait;
use cosmrs::{
    proto::{cosmos::tx::v1beta1::SimulateResponse, prost::Message},
    rpc::endpoint::broadcast::{tx_async, tx_sync},
};

use crate::error::CosmosResult;

/// Type alias for the synchronous response of a broadcasted transaction.
pub type TxSyncResponse = tx_sync::Response;

/// Type alias for the asynchronous response of a broadcasted transaction.
pub type TxAsyncResponse = tx_async::Response;

/// Trait for interacting with Cosmos RPC methods.
#[async_trait]
pub trait Rpc {
    /// Asynchronously queries the blockchain at a given path with a specified message.
    /// Returns the result as a CosmosResult.
    async fn query<M, R>(&self, path: &str, msg: M) -> CosmosResult<R>
    where
        Self: Sized,
        M: Message + Default + 'static,
        R: Message + Default + 'static;

    /// Asynchronously simulates a transaction using the provided payload.
    /// Returns the simulation response as a CosmosResult.
    async fn simulate_tx(&self, payload: Vec<u8>) -> CosmosResult<SimulateResponse>;

    /// Asynchronously broadcasts a transaction and waits for it to be included in a block.
    /// Returns the sync response as a CosmosResult.
    async fn broadcast_tx_sync(&self, payload: Vec<u8>) -> CosmosResult<TxSyncResponse>;

    /// Asynchronously broadcasts a transaction without waiting for it to be included in a block.
    /// Returns the async response as a CosmosResult.
    async fn broadcast_tx_async(&self, payload: Vec<u8>) -> CosmosResult<TxAsyncResponse>;
}
