use anchor_lang::AnchorDeserialize;
use anyhow::{Context, Result, bail};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::{env, str::FromStr};

use crate::{pda::derive_config_pda, types::Config};

/// Read PROGRAM_ID from env and parse into Pubkey
pub fn program_id_from_env() -> Result<Pubkey> {
    let _ = dotenvy::dotenv();
    let s = env::var("PROGRAM_ID").context("PROGRAM_ID must be set")?;
    Pubkey::from_str(&s).context("Invalid PROGRAM_ID")
}

/// Fetch and deserialize the on-chain Config account
pub fn get_config_account(client: &RpcClient, program_id: &Pubkey) -> Result<Config> {
    let config_pda = derive_config_pda(program_id);

    let acc = client
        .get_account(&config_pda)
        .with_context(|| format!("Failed to fetch config account {}", config_pda))?;

    if acc.owner != *program_id {
        bail!(
            "Config owner mismatch. expected={}, got={}",
            program_id,
            acc.owner
        );
    }

    let data = acc.data;
    if data.len() < 8 {
        bail!("Config account data too short");
    }

    // Skip 8-byte Anchor discriminator
    let mut cursor: &[u8] = &data[8..];
    let cfg = Config::deserialize(&mut cursor).context("Failed to deserialize Config")?;

    Ok(cfg)
}
