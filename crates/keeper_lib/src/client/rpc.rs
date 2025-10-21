use solana_client::{rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig};
use solana_commitment_config::{CommitmentConfig, CommitmentLevel};
use std::time::Duration;
pub struct Rpc {
    inner: RpcClient,
    commitment_cfg: CommitmentConfig,
    send_cfg: RpcSendTransactionConfig,
}

impl Rpc {
    pub fn new(
        rpc_url: &str,
        timeout_ms: u64,
        commitment: CommitmentLevel,
        preflight: bool,
        max_retries: usize,
    ) -> Self {
        let commitment_cfg = CommitmentConfig { commitment };
        let inner = RpcClient::new_with_timeout_and_commitment(
            rpc_url.to_string(),
            Duration::from_millis(timeout_ms),
            commitment_cfg,
        );
        let send_cfg = RpcSendTransactionConfig {
            skip_preflight: !preflight,
            max_retries: Some(max_retries),
            preflight_commitment: Some(commitment),
            ..Default::default()
        };

        Self {
            inner,
            commitment_cfg,
            send_cfg,
        }
    }

    pub fn client(&self) -> &RpcClient {
        &self.inner
    }

    pub fn commitment_cfg(&self) -> &CommitmentConfig {
        &self.commitment_cfg
    }

    pub fn send_cfg(&self) -> &RpcSendTransactionConfig {
        &self.send_cfg
    }
}
