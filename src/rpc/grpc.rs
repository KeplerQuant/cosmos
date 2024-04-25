use std::str::FromStr;

use async_trait::async_trait;
use bytes::Bytes;
use cosmrs::proto::cosmos::tx::v1beta1::service_client::ServiceClient;
use cosmrs::proto::cosmos::tx::v1beta1::{BroadcastMode, BroadcastTxRequest};
use cosmrs::proto::cosmos::tx::v1beta1::{SimulateRequest, SimulateResponse};
use cosmrs::proto::prost::Message;

use cosmrs::tendermint::abci::Code;
use cosmrs::tendermint::Hash;
use tonic::codec::ProstCodec;
use tonic::transport::Channel;

use crate::error::{CosmosResult, Error};
use crate::rpc::types::{Rpc, TxAsyncResponse, TxSyncResponse};

/// Struct representing a gRPC client for interacting with Cosmos blockchain.
#[derive(Clone, Debug)]
pub struct Grpc {
    grpc_endpoint: String,
    client: tonic::client::Grpc<Channel>,
}

impl Grpc {
    /// Creates a new `Grpc` instance with the specified gRPC endpoint.
    pub async fn new(grpc_endpoint: &str) -> CosmosResult<Self> {
        let conn = tonic::transport::Endpoint::new(grpc_endpoint.to_string())?
            .connect()
            .await?;
        let client = tonic::client::Grpc::new(conn);

        Ok(Self {
            client,
            grpc_endpoint: grpc_endpoint.to_string(),
        })
    }
}

#[async_trait]
impl Rpc for Grpc {
    /// Asynchronously queries the blockchain at a given path with a specified message.
    /// Returns the result as a CosmosResult.
    async fn query<M, R>(&self, path: &str, msg: M) -> CosmosResult<R>
    where
        Self: Sized,
        M: Message + Default + 'static,
        R: Message + Default + 'static,
    {
        let codec: ProstCodec<M, R> = tonic::codec::ProstCodec::default();
        let mut client_clone = self.client.clone();
        client_clone.ready().await?;

        let res = client_clone
            .unary(
                tonic::Request::new(msg),
                path.parse()
                    .map_err(|_| Error::QueryPath(path.to_string()))?,
                codec,
            )
            .await?;

        Ok(res.into_inner())
    }

    /// Asynchronously simulates a transaction using the provided payload.
    /// Returns the simulation response as a CosmosResult.
    async fn simulate_tx(&self, payload: Vec<u8>) -> CosmosResult<SimulateResponse> {
        let mut client = ServiceClient::connect(self.grpc_endpoint.clone()).await?;

        #[allow(deprecated)]
        let request = SimulateRequest {
            tx: None,
            tx_bytes: payload,
        };

        let res = client.simulate(request).await?;

        Ok(res.into_inner())
    }

    /// Asynchronously broadcasts a transaction without waiting for it to be included in a block.
    /// Returns the async response as a CosmosResult.
    async fn broadcast_tx_async(&self, payload: Vec<u8>) -> CosmosResult<TxAsyncResponse> {
        let mut client = ServiceClient::connect(self.grpc_endpoint.clone()).await?;

        let request = BroadcastTxRequest {
            tx_bytes: payload,
            mode: BroadcastMode::Async.into(),
        };

        let res = client.broadcast_tx(request).await?;
        let tx = res.into_inner().tx_response.ok_or(Error::NoneTxResponse)?;

        Ok(TxAsyncResponse {
            code: Code::from(tx.code),
            data: Bytes::from(tx.data.encode_to_vec()),
            log: tx.raw_log,
            hash: Hash::from_str(&tx.txhash).unwrap(),
        })
    }

    /// Asynchronously broadcasts a transaction and waits for it to be included in a block.
    /// Returns the sync response as a CosmosResult.
    async fn broadcast_tx_sync(&self, payload: Vec<u8>) -> CosmosResult<TxSyncResponse> {
        let mut client = ServiceClient::connect(self.grpc_endpoint.clone()).await?;

        let request = BroadcastTxRequest {
            tx_bytes: payload,
            mode: BroadcastMode::Sync.into(),
        };

        let res = client.broadcast_tx(request).await?;
        let tx = res.into_inner().tx_response.ok_or(Error::NoneTxResponse)?;

        Ok(TxSyncResponse {
            code: Code::from(tx.code),
            data: Bytes::from(tx.data.encode_to_vec()),
            log: tx.raw_log,
            hash: Hash::from_str(&tx.txhash).unwrap(),
        })
    }
}
