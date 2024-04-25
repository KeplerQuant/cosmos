//! # Cosmos RPC
//!
//! This crate provides gRPC and JSON-RPC clients for interacting with the Cosmos blockchain.
//!
//! ## Modules
//!
//! - `grpc`: Contains the gRPC client implementation.
//! - `json_rpc`: Contains the JSON-RPC client implementation.
//! - `types`: Contains types used across the RPC clients.
pub mod grpc;
pub mod json_rpc;
pub mod types;
