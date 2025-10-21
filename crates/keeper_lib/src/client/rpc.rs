use anyhow::{Context, Result};
use solana_client::{rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig};
use solana_commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use std::{thread::sleep, time::Duration};

pub struct Rpc {
    inner: RpcClient,
    commitment_cfg: CommitmentConfig,
    send_cfg: RpcSendTransactionConfig,
    max_retries: usize,
    cu_limit: u32,
    cu_price_micro_lamports: u64,
    backoff_ms: u64,
}

impl Rpc {
    pub fn new(
        rpc_url: &str,
        timeout_ms: u64,
        commitment: CommitmentLevel,
        preflight: bool,
        max_retries: usize,
        cu_limit: u32,
        cu_price_micro_lamports: u64,
        backoff_ms: u64,
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
            max_retries,
            cu_limit,
            cu_price_micro_lamports,
            backoff_ms,
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

    pub fn cu_limit(&self) -> u32 {
        self.cu_limit
    }

    pub fn cu_price_micro_lamports(&self) -> u64 {
        self.cu_price_micro_lamports
    }

    pub fn max_retries(&self) -> usize {
        self.max_retries
    }

    pub fn backoff_ms(&self) -> u64 {
        self.backoff_ms
    }
}

/// Send a transaction with retry logic
///
/// # Arguments
///
/// * `rpc` - The RPC client to use
/// * `payer` - The keypair to use as the transaction payer
/// * `ixs` - The instructions to execute
///
/// # Returns
///
/// The signature of the transaction
pub fn send_tx_with_retry(
    rpc: &Rpc,
    payer: &Keypair,
    mut ixs: Vec<Instruction>,
) -> Result<Signature> {
    let client = rpc.client();
    let commitment_cfg = rpc.commitment_cfg().clone();
    let send_cfg = rpc.send_cfg().clone();
    let cu_limit = rpc.cu_limit();
    let cu_price_micro_lamports = rpc.cu_price_micro_lamports();
    let max_retries = rpc.max_retries();
    let backoff_ms = rpc.backoff_ms();

    ixs.insert(
        0,
        ComputeBudgetInstruction::set_compute_unit_limit(cu_limit),
    );
    ixs.insert(
        0,
        ComputeBudgetInstruction::set_compute_unit_price(cu_price_micro_lamports),
    );

    let mut last_err = None;
    for attempt in 1..=max_retries {
        let bh = client
            .get_latest_blockhash()
            .context("get_latest_blockhash")?;
        let tx = Transaction::new_signed_with_payer(&ixs, Some(&payer.pubkey()), &[payer], bh);

        match client.send_and_confirm_transaction_with_spinner_and_config(
            &tx,
            commitment_cfg.clone(),
            send_cfg.clone(),
        ) {
            Ok(sig) => return Ok(sig),
            Err(e) => {
                last_err = Some(e);
                sleep(Duration::from_millis(
                    backoff_ms.saturating_mul(attempt as u64),
                ));
                continue;
            }
        }
    }

    Err(anyhow::anyhow!("send_tx exhausted retries: {:?}", last_err))
}
