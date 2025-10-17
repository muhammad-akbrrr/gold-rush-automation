use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use std::env;

pub struct Rpc {
    inner: RpcClient,
}

impl Rpc {
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();
        let rpc_url = env::var("SOLANA_RPC_URL").context("SOLANA_RPC_URL must be set")?;

        Ok(Self {
            inner: RpcClient::new(rpc_url),
        })
    }

    pub fn new(rpc_url: impl AsRef<str>) -> Self {
        Self {
            inner: RpcClient::new(rpc_url.as_ref().to_string()),
        }
    }

    pub fn client(&self) -> &RpcClient {
        &self.inner
    }
}
