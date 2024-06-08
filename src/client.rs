use std::ops::{DivAssign, MulAssign};

use cosmrs::proto::cosmos::auth::v1beta1::{
    BaseAccount, QueryAccountRequest, QueryAccountResponse,
};
use cosmrs::proto::cosmos::tx::v1beta1::SimulateResponse;
use cosmrs::proto::cosmos::vesting::v1beta1::ContinuousVestingAccount;
use cosmrs::proto::prost::Message;
use cosmrs::tx::Body;

use crate::error::{CosmosResult, Error};
use crate::rpc::types::{Rpc, TxAsyncResponse, TxSyncResponse};
use crate::rpc::{grpc::Grpc, json_rpc::JsonRpc};
use crate::signer::Signer;

/// Represents a Cosmos client that can interact with the blockchain using different RPC protocols.
#[derive(Debug, Clone)]
pub struct CosmosClient<T: Rpc + Clone + Send + Sync> {
    /// The chain ID for the Cosmos blockchain.
    chain_id: String,
    /// The underlying RPC implementation used by the client.
    rpc: T,
    /// The signer used for transaction signing.
    signer: Option<Signer>,
}

impl CosmosClient<JsonRpc> {
    /// Creates a new Cosmos client with JSON-RPC protocol.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint URL for the JSON-RPC server.
    /// * `chain_id` - The chain ID for the Cosmos blockchain.
    ///
    /// # Returns
    ///
    /// A `CosmosResult` containing the initialized `CosmosClient` if successful, or an error if
    /// the initialization fails.
    pub fn with_json_rpc(endpoint: &str, chain_id: &str) -> CosmosResult<CosmosClient<JsonRpc>> {
        let rpc = JsonRpc::new(endpoint)?;
        Ok(Self {
            rpc,
            chain_id: chain_id.to_owned(),
            signer: None,
        })
    }
}

impl CosmosClient<Grpc> {
    /// Creates a new Cosmos client with gRPC protocol.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The endpoint URL for the gRPC server.
    /// * `chain_id` - The chain ID for the Cosmos blockchain.
    ///
    /// # Returns
    ///
    /// A `CosmosClient` initialized with the specified gRPC endpoint.
    pub async fn with_grpc(endpoint: &str, chain_id: &str) -> CosmosResult<CosmosClient<Grpc>> {
        let rpc = Grpc::new(endpoint).await?;
        Ok(Self {
            rpc,
            chain_id: chain_id.to_owned(),
            signer: None,
        })
    }
}

impl<T: Rpc + Clone + Send + Sync> CosmosClient<T> {
    /// This method associates a signer with the client, providing the necessary information for
    /// transaction signing.
    pub async fn attach_signer(&mut self, signer: Signer) {
        self.signer = Some(signer);
    }

    /// Retrieves the currently associated signer.
    pub fn signer(&self) -> Option<&Signer> {
        self.signer.as_ref()
    }

    /// Asynchronously queries the blockchain at a given path with a specified message.
    /// Returns the result as a CosmosResult.
    pub async fn query<M, R>(&self, path: &str, msg: M) -> CosmosResult<R>
    where
        Self: Sized,
        M: Message + Default + 'static,
        R: Message + Default + 'static,
    {
        self.rpc.query(path, msg).await
    }

    /// Asynchronously simulates a transaction using the provided payload.
    /// Returns the simulation response as a CosmosResult.
    pub async fn simulate_tx(&self, body: Body) -> CosmosResult<SimulateResponse> {
        let mut signer = self.signer.clone().ok_or(Error::NoSignerAttached)?;
        let (account_number, sequence_id) = self.account_sequence_id().await?;
        let tx = signer
            .sign(&self.chain_id, account_number, sequence_id, 100u64, body)
            .await?;

        self.rpc.simulate_tx(tx).await
    }

    /// Asynchronously broadcasts a transaction without waiting for it to be included in a block.
    /// Returns the async response as a CosmosResult.
    pub async fn broadcast_tx_async(&self, body: Body) -> CosmosResult<TxAsyncResponse> {
        let payload = self.sign_tx(body).await?;
        self.rpc.broadcast_tx_async(payload).await
    }

    /// Asynchronously broadcasts a transaction and waits for it to be included in a block.
    /// Returns the sync response as a CosmosResult.
    pub async fn broadcast_tx_sync(&self, body: Body) -> CosmosResult<TxSyncResponse> {
        let payload = self.sign_tx(body).await?;
        self.rpc.broadcast_tx_sync(payload).await
    }

    /// Asynchronously signs a transaction using the provided `Body`.
    async fn sign_tx(&self, body: Body) -> CosmosResult<Vec<u8>> {
        let simulate_response = self.simulate_tx(body.clone()).await?;
        if simulate_response.gas_info.is_none() {
            return Err(Error::CannotSimulateTxGas);
        }

        let mut signer = self.signer.clone().ok_or(Error::NoSignerAttached)?;
        let mut gas_info = simulate_response.gas_info.unwrap_or_default().gas_used;

        gas_info.mul_assign(100u64 + u64::from(signer.gas_adjustment_percent));
        gas_info.div_assign(100);

        let (account_number, sequence_id) = self.account_sequence_id().await?;
        signer
            .sign(&self.chain_id, account_number, sequence_id, gas_info, body)
            .await
    }

    /// Asynchronously updates the client's sequence ID and account ID from the blockchain.
    ///
    /// This method queries the blockchain to obtain the latest account information, including
    /// the account's sequence ID and account ID. It updates the client's internal state with
    /// the obtained information.
    ///
    /// # Returns
    ///
    /// A `CosmosResult` indicating the success of the operation or an error if any.
    async fn account_sequence_id(&self) -> CosmosResult<(u64, u64)> {
        let signer = self.signer.clone().ok_or(Error::NoSignerAttached)?;

        let query = QueryAccountRequest {
            address: signer.public_address.to_string(),
        };

        let response: QueryAccountResponse = self
            .query("/cosmos.auth.v1beta1.Query/Account", query)
            .await?;

        let account = response.account.ok_or(Error::AccountDoesNotExist {
            address: signer.public_address.to_string(),
        })?;

        match account.type_url.as_str() {
            "/cosmos.auth.v1beta1.BaseAccount" => {
                let account = BaseAccount::decode(account.value.as_slice())?;
                return Ok((account.account_number, account.sequence));
            }
            "/cosmos.vesting.v1beta1.ContinuousVestingAccount" => {
                let account = ContinuousVestingAccount::decode(account.value.as_slice())?;
                let account = account
                    .base_vesting_account
                    .ok_or(Error::NoVestingBaseAccount)?
                    .base_account
                    .ok_or(Error::NoVestingBaseAccount)?;
                return Ok((account.account_number, account.sequence));
            }
            _ => {}
        }

        Err(Error::AccountDoesNotExist {
            address: signer.public_address.to_string(),
        })
    }
}
