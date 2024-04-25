use std::thread::sleep;
use std::time::Duration;

use crate::error::{CosmosResult, Error};
use crate::rpc::types::{TxAsyncResponse, TxSyncResponse};
use crate::{client::CosmosClient, rpc::types::Rpc};
use cosmrs::proto::cosmos::tx::v1beta1::{GetTxRequest, GetTxResponse, SimulateResponse};
use cosmrs::rpc::endpoint::broadcast::{tx_async, tx_sync};
use cosmrs::tx::Body;

/// Enum representing different responses for broadcast transactions.
#[derive(Clone, Debug)]
pub enum Response {
    /// Asynchronous broadcast response.
    Async(tx_async::Response),
    /// Synchronous broadcast response.
    Sync(tx_sync::Response),
}

/// Main struct providing access to Tx module functions.
#[derive(Debug, Clone)]
pub struct Tx<T: Rpc + Clone + Send + Sync> {
    client: CosmosClient<T>,
}

impl<T: Rpc + Clone + Send + Sync> Tx<T> {
    /// Creates a new `Tx` instance with the provided Cosmos client.
    pub fn new(client: CosmosClient<T>) -> Self {
        Self { client }
    }

    /// Simulates the execution of a transaction.
    pub async fn simulate(&self, payload: Body) -> CosmosResult<SimulateResponse> {
        self.client.simulate_tx(payload).await
    }

    /// Broadcasts a transaction synchronously.
    pub async fn broadcast_tx_sync(&self, body: Body) -> CosmosResult<TxSyncResponse> {
        self.client.broadcast_tx_sync(body).await
    }

    /// Broadcasts a transaction asynchronously.
    pub async fn broadcast_tx_async(&self, body: Body) -> CosmosResult<TxAsyncResponse> {
        self.client.broadcast_tx_async(body).await
    }

    /// Retrieves information about a specific transaction using its hash.
    pub async fn get_tx(&self, hash: &str) -> CosmosResult<GetTxResponse> {
        let query = GetTxRequest {
            hash: hash.to_string(),
        };

        self.client
            .query("/cosmos.tx.v1beta1.Service/GetTx", query)
            .await
    }

    /// Polls for a transaction until it is found or a timeout is reached.
    ///
    /// This function repeatedly calls `get_tx` to check the status of a transaction identified by its hash.
    /// It will continue polling for up to 60 iterations, with a 3-second sleep between each attempt.
    pub async fn poll_for_tx(&self, hash: &str) -> CosmosResult<GetTxResponse> {
        for _ in 0..60 {
            let tx = self.get_tx(hash).await;

            if tx.is_ok() {
                return tx;
            }

            sleep(Duration::from_secs(3));
        }

        Err(Error::TXPollingTimeout)
    }
}
