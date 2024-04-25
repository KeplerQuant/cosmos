use async_trait::async_trait;
use cosmrs::proto::cosmos::tx::v1beta1::{SimulateRequest, SimulateResponse};
use cosmrs::rpc::HttpClient;
use cosmrs::tendermint::abci::Code;
use cosmrs::{proto::prost::Message, rpc::Client};

use crate::error::{CosmosResult, Error};
use crate::rpc::types::{Rpc, TxAsyncResponse, TxSyncResponse};

/// Struct representing a JSON-RPC client for interacting with Cosmos blockchain.
#[derive(Clone, Debug)]
pub struct JsonRpc {
    client: HttpClient,
}

impl JsonRpc {
    /// Creates a new `JsonRpc` instance with the specified RPC endpoint.
    pub fn new(rpc_endpoint: &str) -> CosmosResult<Self> {
        Ok(Self {
            client: HttpClient::new(rpc_endpoint)?,
        })
    }
}

#[async_trait]
impl Rpc for JsonRpc {
    /// Asynchronously queries the blockchain at a given path with a specified message.
    /// Returns the result as a CosmosResult.
    async fn query<M, R>(&self, path: &str, msg: M) -> CosmosResult<R>
    where
        Self: Sized,
        M: Message + Default + 'static,
        R: Message + Default + 'static,
    {
        let data = msg.encode_to_vec();
        let res = self
            .client
            .abci_query(Some(path.to_string()), data, None, false)
            .await?;

        if res.code != Code::Ok {
            return Err(Error::RpcError(res.log));
        }

        let proto_res = R::decode(res.value.as_slice())?;

        Ok(proto_res)
    }

    /// Asynchronously simulates a transaction using the provided payload.
    /// Returns the simulation response as a CosmosResult.
    async fn simulate_tx(&self, payload: Vec<u8>) -> CosmosResult<SimulateResponse> {
        #[allow(deprecated)]
        let query = SimulateRequest {
            tx: None,
            tx_bytes: payload,
        };

        self.query("/cosmos.tx.v1beta1.Service/Simulate", query)
            .await
    }

    /// Asynchronously broadcasts a transaction without waiting for it to be included in a block.
    /// Returns the async response as a CosmosResult.
    async fn broadcast_tx_async(&self, payload: Vec<u8>) -> CosmosResult<TxAsyncResponse> {
        let res = self.client.broadcast_tx_async(payload).await?;
        Ok(res)
    }

    /// Asynchronously broadcasts a transaction and waits for it to be included in a block.
    /// Returns the sync response as a CosmosResult.
    async fn broadcast_tx_sync(&self, payload: Vec<u8>) -> CosmosResult<TxSyncResponse> {
        let res = self.client.broadcast_tx_sync(payload).await?;
        Ok(res)
    }
}
