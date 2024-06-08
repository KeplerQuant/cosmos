use thiserror::Error;

pub type CosmosResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    TendermintRpcError(#[from] cosmrs::rpc::Error),
    #[error(transparent)]
    Bip32Error(#[from] cosmrs::bip32::Error),
    #[error(transparent)]
    ErrorReport(#[from] cosmrs::ErrorReport),
    #[error(transparent)]
    FromHexError(#[from] hex::FromHexError),
    #[error(transparent)]
    DecodeError(#[from] cosmrs::proto::prost::DecodeError),
    #[error(transparent)]
    EncodeError(#[from] cosmrs::proto::prost::EncodeError),
    #[error(transparent)]
    TonicError(#[from] tonic::transport::Error),
    #[error(transparent)]
    TonicStatus(#[from] tonic::Status),
    #[error(transparent)]
    TendermintError(#[from] cosmrs::tendermint::Error),

    #[error("Unknown cosmos-sdk Msg")]
    UnknownCosmosMsg,
    #[error("No signer attached")]
    NoSignerAttached,
    #[error("No subscription")]
    NoSubscription,
    #[error("Cannot simulate TX Gas Fee")]
    CannotSimulateTxGasFee,
    #[error("Out of gas")]
    OutOfGas,
    #[error("Account does not exist {address:?}")]
    AccountDoesNotExist { address: String },
    #[error("Rpc errors : {0}")]
    RpcError(String),
    #[error("QueryPath errors : {0}")]
    QueryPath(String),
    #[error("NoneTxResponse")]
    NoneTxResponse,
    #[error("TXPollingTimeout")]
    TXPollingTimeout,
    #[error("No base account for vesting wallet")]
    NoVestingBaseAccount,
    #[error("{0}")]
    Custom(String),

    #[cfg(feature = "osmosis")]
    #[error("Not found pool")]
    NotFoundPool,
}
