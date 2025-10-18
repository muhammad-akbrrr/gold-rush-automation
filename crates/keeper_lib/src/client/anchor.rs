use anchor_lang::AnchorDeserialize;
use anyhow::{Context, Result, bail};
use hex;
use sha2::{Digest, Sha256};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};

use crate::{
    pda::{derive_config_pda, derive_round_pda},
    types::{ConfigAccount, RoundAccount},
};

/// Generate a 8-byte sighash for a global instruction
fn sighash_global(ix_name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}", "global", ix_name));
    let hash = hasher.finalize();
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(&hash[..8]);
    sighash
}

pub fn get_price_feed_account(
    shard_id: u16,
    feed_id: &str,
    push_oracle_program_id: &Pubkey,
) -> Result<Pubkey> {
    let s = feed_id.strip_prefix("0x").unwrap_or(feed_id);
    let bytes = hex::decode(s).context("invalid feed id")?;
    if bytes.len() != 32 {
        bail!("feed id muts be 32 bytes");
    }
    let mut id = [0u8; 32];
    id.copy_from_slice(&bytes);

    let shard_le = shard_id.to_le_bytes();
    let (pda, _bump) = Pubkey::find_program_address(&[&shard_le, &id], push_oracle_program_id);
    Ok(pda)
}

/// Fetch and deserialize Config account
pub fn get_config_account(client: &RpcClient, program_id: &Pubkey) -> Result<ConfigAccount> {
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

    let mut cursor: &[u8] = &data[8..];
    let cfg = ConfigAccount::deserialize(&mut cursor).context("Failed to deserialize Config")?;

    Ok(cfg)
}

pub fn get_rounds_by_ids(
    client: &RpcClient,
    program_id: &Pubkey,
    ids: &[u64],
) -> Result<Vec<RoundAccount>> {
    let pubkeys: Vec<Pubkey> = ids
        .iter()
        .map(|&id| derive_round_pda(program_id, id))
        .collect();
    let accounts = client
        .get_multiple_accounts(&pubkeys)
        .context("Failed to get multiple round accounts")?;

    let mut out = Vec::new();
    for (_, acc_opt) in pubkeys.into_iter().zip(accounts.into_iter()) {
        if let Some(acc) = acc_opt {
            if acc.owner != *program_id {
                continue;
            }

            if acc.data.len() < 8 {
                continue;
            }

            let mut cursor = &acc.data[8..];
            if let Ok(round) = RoundAccount::deserialize(&mut cursor) {
                out.push(round);
            }
        }
    }

    Ok(out)
}

pub fn start_round(
    rpc: &RpcClient,
    payer: &Keypair,
    round_pda: &Pubkey,
    gold_price_feed: &Pubkey,
    system_program_id: &Pubkey,
    program_id: &Pubkey,
) -> Result<Signature> {
    let config_pda = derive_config_pda(&program_id);
    let data = sighash_global("start_round").to_vec();

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(config_pda, false),
        AccountMeta::new(*round_pda, false),
        AccountMeta::new_readonly(*gold_price_feed, false),
        AccountMeta::new_readonly(*system_program_id, false),
    ];

    let instruction = Instruction {
        data,
        accounts,
        program_id: *program_id,
    };
    let bh = rpc
        .get_latest_blockhash()
        .context("Failed to get latest blockhash")?;
    let tx =
        Transaction::new_signed_with_payer(&[instruction], Some(&payer.pubkey()), &[payer], bh);

    let sig = rpc
        .send_and_confirm_transaction(&tx)
        .context("Failed to send and confirm transaction")?;

    Ok(sig)
}
