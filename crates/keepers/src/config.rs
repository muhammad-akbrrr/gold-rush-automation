use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use std::{env, str::FromStr};

#[derive(Clone)]
pub struct RuntimeConfig {
    pub solana_rpc_url: String,
    pub keeper_keypair_path: String,
    pub gold_price_feed_id: String,

    pub push_oracle_program_id: Pubkey,
    pub system_program_id: Pubkey,
    pub program_id: Pubkey,

    pub start_round_period_in_secs: u64,
    pub settle_round_period_in_secs: u64,
}

pub fn load() -> Result<RuntimeConfig> {
    let _ = dotenvy::dotenv();

    let solana_rpc_url = env::var("SOLANA_RPC_URL").context("SOLANA_RPC_URL must be set")?;
    let keeper_keypair_path =
        env::var("KEEPER_KEYPAIR_PATH").context("KEEPER_KEYPAIR_PATH must be set")?;
    let gold_price_feed_id =
        env::var("GOLD_PRICE_FEED_ID").context("GOLD_PRICE_FEED_ID must be set")?;
    let push_oracle_program_id = Pubkey::from_str(
        &env::var("PUSH_ORACLE_PROGRAM_ID").context("PUSH_ORACLE_PROGRAM_ID must be set")?,
    )?;
    let system_program_id =
        Pubkey::from_str(&env::var("SYSTEM_PROGRAM_ID").context("SYSTEM_PROGRAM_ID must be set")?)?;
    let program_id = Pubkey::from_str(&env::var("PROGRAM_ID").context("PROGRAM_ID must be set")?)?;

    let start_round_period_in_secs = env::var("START_ROUND_PERIOD_IN_SECS")
        .context("START_ROUND_PERIOD_IN_SECS must be set")?
        .parse::<u64>()
        .context("START_ROUND_PERIOD_IN_SECS must be a valid number")?;
    let settle_round_period_in_secs = env::var("SETTLE_ROUND_PERIOD_IN_SECS")
        .context("SETTLE_ROUND_PERIOD_IN_SECS must be set")?
        .parse::<u64>()
        .context("SETTLE_ROUND_PERIOD_IN_SECS must be a valid number")?;

    Ok(RuntimeConfig {
        solana_rpc_url,
        keeper_keypair_path,
        gold_price_feed_id,
        push_oracle_program_id,
        system_program_id,
        program_id,
        start_round_period_in_secs,
        settle_round_period_in_secs,
    })
}
