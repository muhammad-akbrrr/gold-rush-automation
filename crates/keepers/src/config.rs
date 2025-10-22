use anyhow::{Context, Result};
use solana_commitment_config::CommitmentLevel;
use solana_sdk::pubkey::Pubkey;
use std::{env, str::FromStr};

#[derive(Clone)]
pub struct RuntimeConfig {
    pub solana_rpc_url: String,
    pub commitment: CommitmentLevel,
    pub rpc_timeout_ms: u64,
    pub preflight: bool,
    pub tx_max_retries: usize,
    pub cu_limit: u32,
    pub cu_price_micro_lamports: u64,
    pub backoff_ms: u64,

    pub keeper_keypair_path: String,
    pub treasury: Pubkey,

    pub gold_price_feed_id: String,
    pub token_mint: Pubkey,

    pub start_round_period_in_secs: u64,
    pub settle_round_period_in_secs: u64,
    pub max_remaining_accounts: usize,

    pub token_program_id: Pubkey,
    pub associated_token_program_id: Pubkey,
    pub push_oracle_program_id: Pubkey,
    pub system_program_id: Pubkey,
    pub program_id: Pubkey,
}

pub fn load() -> Result<RuntimeConfig> {
    let _ = dotenvy::dotenv();

    let solana_rpc_url = env_str("SOLANA_RPC_URL", None).context("SOLANA_RPC_URL must be set")?;
    let commitment = env_commitment("COMMITMENT", None).context("COMMITMENT must be set")?;
    let rpc_timeout_ms = env_u64("RPC_TIMEOUT_MS", None).context("RPC_TIMEOUT_MS must be set")?;
    let tx_max_retries = env_usize("TX_MAX_RETRIES", None).context("TX_MAX_RETRIES must be set")?;
    let preflight = env_bool("PREFLIGHT", None).context("PREFLIGHT must be set")?;
    let cu_limit = env_u32("COMPUTE_UNIT_LIMIT", None).context("COMPUTE_UNIT_LIMIT must be set")?;
    let cu_price_micro_lamports = env_u64("PRIORITY_FEE_MICROLAMPORTS", None)
        .context("PRIORITY_FEE_MICROLAMPORTS must be set")?;
    let backoff_ms = env_u64("BACKOFF_MS", None).context("BACKOFF_MS must be set")?;

    let keeper_keypair_path =
        env_str("KEEPER_KEYPAIR_PATH", None).context("KEEPER_KEYPAIR_PATH must be set")?;
    let treasury = env_pubkey("TREASURY", None).context("TREASURY must be set")?;

    let gold_price_feed_id =
        env_str("GOLD_PRICE_FEED_ID", None).context("GOLD_PRICE_FEED_ID must be set")?;
    let token_mint = env_pubkey("TOKEN_MINT", None).context("TOKEN_MINT must be set")?;

    let start_round_period_in_secs = env_u64("START_ROUND_PERIOD_IN_SECS", None)
        .context("START_ROUND_PERIOD_IN_SECS must be set")?;
    let settle_round_period_in_secs = env_u64("SETTLE_ROUND_PERIOD_IN_SECS", None)
        .context("SETTLE_ROUND_PERIOD_IN_SECS must be set")?;
    let max_remaining_accounts =
        env_usize("MAX_REMAINING_ACCOUNTS", None).context("MAX_REMAINING_ACCOUNTS must be set")?;

    let token_program_id =
        env_pubkey("TOKEN_PROGRAM_ID", None).context("TOKEN_PROGRAM_ID must be set")?;
    let associated_token_program_id = env_pubkey("ASSOCIATED_TOKEN_PROGRAM_ID", None)
        .context("ASSOCIATED_TOKEN_PROGRAM_ID must be set")?;
    let push_oracle_program_id =
        env_pubkey("PUSH_ORACLE_PROGRAM_ID", None).context("PUSH_ORACLE_PROGRAM_ID must be set")?;
    let system_program_id =
        env_pubkey("SYSTEM_PROGRAM_ID", None).context("SYSTEM_PROGRAM_ID must be set")?;
    let program_id = env_pubkey("PROGRAM_ID", None).context("PROGRAM_ID must be set")?;

    Ok(RuntimeConfig {
        solana_rpc_url,
        commitment,
        rpc_timeout_ms,
        preflight,
        tx_max_retries,
        cu_limit,
        cu_price_micro_lamports,
        backoff_ms,
        keeper_keypair_path,
        treasury,
        gold_price_feed_id,
        token_mint,
        start_round_period_in_secs,
        settle_round_period_in_secs,
        max_remaining_accounts,
        token_program_id,
        associated_token_program_id,
        push_oracle_program_id,
        system_program_id,
        program_id,
    })
}

fn env_str(key: &str, default: Option<String>) -> Option<String> {
    env::var(key).ok().or(default)
}

fn env_bool(key: &str, default: Option<bool>) -> Option<bool> {
    env::var(key).ok().and_then(|v| v.parse().ok()).or(default)
}

fn env_u64(key: &str, default: Option<u64>) -> Option<u64> {
    env::var(key).ok().and_then(|v| v.parse().ok()).or(default)
}

fn env_u32(key: &str, default: Option<u32>) -> Option<u32> {
    env::var(key).ok().and_then(|v| v.parse().ok()).or(default)
}

fn env_usize(key: &str, default: Option<usize>) -> Option<usize> {
    env::var(key).ok().and_then(|v| v.parse().ok()).or(default)
}

fn env_pubkey(key: &str, default: Option<Pubkey>) -> Option<Pubkey> {
    env::var(key)
        .ok()
        .and_then(|v| Pubkey::from_str(&v).ok())
        .or(default)
}

fn env_commitment(key: &str, default: Option<CommitmentLevel>) -> Option<CommitmentLevel> {
    match env::var(key).unwrap_or_default().to_lowercase().as_str() {
        "finalized" => Some(CommitmentLevel::Finalized),
        "confirmed" => Some(CommitmentLevel::Confirmed),
        "processed" => Some(CommitmentLevel::Processed),
        _ => None,
    }
    .or(default)
}
