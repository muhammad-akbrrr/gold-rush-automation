use anchor_lang::AnchorDeserialize;
use anyhow::{Context, Result, bail};
use hex;
use sha2::{Digest, Sha256};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
};

use crate::{
    client::rpc::{Rpc, send_tx_with_retry},
    pda::{
        derive_asset_pda, derive_bet_pda, derive_config_pda, derive_group_asset_pda,
        derive_round_pda,
    },
    types::{AssetAccount, ConfigAccount, GroupAssetAccount, MarketType, RoundAccount},
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

/// Get the price feed account for a given feed ID
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

    let mut cursor = &data[8..];
    let cfg = ConfigAccount::deserialize(&mut cursor).context("Failed to deserialize Config")?;

    Ok(cfg)
}

/// Fetch and deserialize multiple Round accounts by their IDs
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

/// Fetch and deserialize GroupAsset account
pub fn get_group_asset_account(
    client: &RpcClient,
    program_id: &Pubkey,
    group_asset_pda: &Pubkey,
) -> Result<GroupAssetAccount> {
    let acc = client
        .get_account(&group_asset_pda)
        .with_context(|| format!("Failed to get group asset account {}", group_asset_pda))?;

    if acc.owner != *program_id {
        bail!(
            "Group asset owner mismatch. expected={}, got={}",
            program_id,
            acc.owner
        );
    }

    let data = acc.data;
    if data.len() < 8 {
        bail!("Group asset account data too short");
    }

    let mut cursor = &data[8..];
    let group_asset = GroupAssetAccount::deserialize(&mut cursor)
        .context("Failed to deserialize GroupAssetAccount")?;

    Ok(group_asset)
}

/// Fetch and deserialize Asset account
pub fn get_asset_account(
    client: &RpcClient,
    asset_pda: &Pubkey,
    program_id: &Pubkey,
) -> Result<AssetAccount> {
    let acc = client
        .get_account(asset_pda)
        .context("Failed to get asset account")?;

    if acc.owner != *program_id {
        bail!(
            "Asset owner mismatch. expected={}, got={}",
            program_id,
            acc.owner
        );
    }

    let data = acc.data;
    if data.len() < 8 {
        bail!("Asset account data too short");
    }

    let mut cursor = &data[8..];
    let asset =
        AssetAccount::deserialize(&mut cursor).context("Failed to deserialize AssetAccount")?;

    Ok(asset)
}

/// Start a round
pub fn start_round(
    rpc: &Rpc,
    payer: &Keypair,
    config_pda: &Pubkey,
    round_pda: &Pubkey,
    gold_price_feed: Option<&Pubkey>,
    system_program_id: &Pubkey,
    program_id: &Pubkey,
) -> Result<Signature> {
    let data = sighash_global("start_round").to_vec();

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(*config_pda, false),
        AccountMeta::new(*round_pda, false),
        AccountMeta::new_readonly(*gold_price_feed.unwrap_or(&program_id), false), // use program id if no gold price feed is provided
        AccountMeta::new_readonly(*system_program_id, false),
    ];

    let instruction = Instruction {
        data,
        accounts,
        program_id: *program_id,
    };

    let sig = send_tx_with_retry(&rpc, payer, [instruction].to_vec())?;

    Ok(sig)
}

/// Capture the start price for a group
pub fn capture_start_price(
    rpc: &Rpc,
    payer: &Keypair,
    config_pda: &Pubkey,
    round_pda: &Pubkey,
    round: &RoundAccount,
    push_oracle_program_id: &Pubkey,
    system_program_id: &Pubkey,
    program_id: &Pubkey,
) -> Result<Vec<Signature>> {
    if !matches!(round.market_type, MarketType::GroupBattle) {
        bail!("capture_start_price only supported for group battle rounds");
    }

    if round.total_groups < 1 {
        bail!("capture_start_price requires at least one group");
    }

    let mut sigs: Vec<Signature> = Vec::new();

    let data = sighash_global("capture_start_price").to_vec();

    for group_id in 1..=round.total_groups {
        let mut remaining_accounts: Vec<AccountMeta> = Vec::new();

        let group_asset_pda = derive_group_asset_pda(program_id, round_pda, group_id);
        let group_asset = get_group_asset_account(rpc.client(), program_id, &group_asset_pda)?;

        if group_asset.total_assets < 1 {
            println!("group {} has no assets", group_id);
            continue;
        }

        if group_asset.captured_start_price_assets == group_asset.total_assets {
            println!("group {} already captured start price", group_id);
            continue;
        }

        for asset_id in 1..=group_asset.total_assets {
            let asset_pda = derive_asset_pda(program_id, &group_asset_pda, asset_id);
            remaining_accounts.push(AccountMeta {
                pubkey: asset_pda,
                is_signer: false,
                is_writable: true,
            });

            let asset = get_asset_account(rpc.client(), &asset_pda, program_id)?;
            let price_feed_account =
                get_price_feed_account(0, &hex::encode(asset.feed_id), push_oracle_program_id)?;
            remaining_accounts.push(AccountMeta {
                pubkey: price_feed_account,
                is_signer: false,
                is_writable: false,
            });
        }

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(*config_pda, false),
            AccountMeta::new(*round_pda, false),
            AccountMeta::new(group_asset_pda, false),
            AccountMeta::new_readonly(*system_program_id, false),
        ]
        .into_iter()
        .chain(remaining_accounts.into_iter())
        .collect();

        let instruction = Instruction {
            data: data.clone(),
            accounts,
            program_id: *program_id,
        };

        let sig = send_tx_with_retry(&rpc, payer, [instruction].to_vec())?;

        println!("capture_start_price for group {}: {}", group_id, sig);

        sigs.push(sig);
    }

    Ok(sigs)
}

/// Finalize the start price for a group
pub fn finalize_start_group_assets(
    rpc: &Rpc,
    payer: &Keypair,
    config_pda: &Pubkey,
    round_pda: &Pubkey,
    round: &RoundAccount,
    system_program_id: &Pubkey,
    program_id: &Pubkey,
) -> Result<Vec<Signature>> {
    if !matches!(round.market_type, MarketType::GroupBattle) {
        bail!("finalize_start_group_assets only supported for group battle rounds");
    }

    if round.total_groups < 1 {
        bail!("finalize_start_group_assets requires at least one group");
    }

    let mut sigs: Vec<Signature> = Vec::new();

    let data = sighash_global("finalize_start_group_asset").to_vec();

    for group_id in 1..=round.total_groups {
        let mut remaining_accounts: Vec<AccountMeta> = Vec::new();

        let group_asset_pda = derive_group_asset_pda(program_id, round_pda, group_id);
        let group_asset = get_group_asset_account(rpc.client(), program_id, &group_asset_pda)?;

        if group_asset.total_assets < 1 {
            println!("group {} has no assets", group_id);
            continue;
        }

        if group_asset.captured_start_price_assets != group_asset.total_assets {
            println!("group {} has not captured start price", group_id);
            continue;
        }

        if group_asset.finalized_start_price_assets == group_asset.total_assets {
            println!("group {} already finalized start price", group_id);
            continue;
        }

        for asset_id in 1..=group_asset.total_assets {
            let asset_pda = derive_asset_pda(program_id, &group_asset_pda, asset_id);
            remaining_accounts.push(AccountMeta {
                pubkey: asset_pda,
                is_signer: false,
                is_writable: true,
            });
        }

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(*config_pda, false),
            AccountMeta::new(*round_pda, false),
            AccountMeta::new(group_asset_pda, false),
            AccountMeta::new_readonly(*system_program_id, false),
        ]
        .into_iter()
        .chain(remaining_accounts.into_iter())
        .collect();

        let instruction = Instruction {
            data: data.clone(),
            accounts,
            program_id: *program_id,
        };

        let sig = send_tx_with_retry(&rpc, payer, [instruction].to_vec())?;

        println!(
            "finalize_start_group_assets for group {}: {}",
            group_id, sig
        );

        sigs.push(sig);
    }

    Ok(sigs)
}

/// Finalize the start groups
pub fn finalize_start_groups(
    rpc: &Rpc,
    payer: &Keypair,
    config_pda: &Pubkey,
    round_pda: &Pubkey,
    round: &RoundAccount,
    system_program_id: &Pubkey,
    program_id: &Pubkey,
) -> Result<Signature> {
    if !matches!(round.market_type, MarketType::GroupBattle) {
        bail!("finalize_start_groups only supported for group battle rounds");
    }

    if round.total_groups < 1 {
        bail!("finalize_start_groups requires at least one group");
    }

    if round.captured_start_groups == round.total_groups {
        bail!("finalize_start_groups already finalized start groups");
    }

    let data = sighash_global("finalize_start_groups").to_vec();

    let mut remaining_accounts: Vec<AccountMeta> = Vec::new();

    for group_id in 1..=round.total_groups {
        let group_asset_pda = derive_group_asset_pda(program_id, round_pda, group_id);
        remaining_accounts.push(AccountMeta {
            pubkey: group_asset_pda,
            is_signer: false,
            is_writable: false,
        });
    }

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(*config_pda, false),
        AccountMeta::new(*round_pda, false),
        AccountMeta::new_readonly(*system_program_id, false),
    ]
    .into_iter()
    .chain(remaining_accounts.into_iter())
    .collect();

    let instruction = Instruction {
        data: data.clone(),
        accounts,
        program_id: *program_id,
    };

    let sig = send_tx_with_retry(&rpc, payer, [instruction].to_vec())?;

    Ok(sig)
}

pub fn settle_single_round(
    rpc: &Rpc,
    payer: &Keypair,
    config_pda: &Pubkey,
    round_pda: &Pubkey,
    round_vault: &Pubkey,
    round: &RoundAccount,
    treasury: &Pubkey,
    treasury_token_account: &Pubkey,
    gold_price_feed: &Pubkey,
    token_mint: &Pubkey,
    token_program_id: &Pubkey,
    associated_token_program_id: &Pubkey,
    system_program_id: &Pubkey,
    program_id: &Pubkey,
) -> Result<Signature> {
    let max_remaining_accounts = rpc.max_remaining_accounts();

    let data = sighash_global("settle_single_round").to_vec();

    let base_accounts = || -> Vec<AccountMeta> {
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(*config_pda, false),
            AccountMeta::new(*round_pda, false),
            AccountMeta::new(*round_vault, false),
            AccountMeta::new_readonly(*gold_price_feed, false),
            AccountMeta::new_readonly(*treasury, false),
            AccountMeta::new(*treasury_token_account, false),
            AccountMeta::new_readonly(*token_mint, false),
            AccountMeta::new_readonly(*token_program_id, false),
            AccountMeta::new_readonly(*associated_token_program_id, false),
            AccountMeta::new_readonly(*system_program_id, false),
        ]
    };

    // If there are no bets, settle the round immediately
    if round.total_bets == 0 {
        let instruction = Instruction {
            data: data.clone(),
            accounts: base_accounts(),
            program_id: *program_id,
        };
        let sig = send_tx_with_retry(&rpc, payer, [instruction].to_vec())?;
        return Ok(sig);
    }

    let total = round.total_bets as usize;
    let mut last_sig: Option<Signature> = None;

    let mut start = 1usize;
    while start <= total {
        let end = (start + max_remaining_accounts - 1).min(total);
        println!("settling bets from {} to {}", start, end);

        let mut remaining_accounts: Vec<AccountMeta> = Vec::with_capacity(end - start + 1);
        for bet_id in start..=end {
            let bet_pda = derive_bet_pda(program_id, round_pda, bet_id as u64);
            remaining_accounts.push(AccountMeta {
                pubkey: bet_pda,
                is_signer: false,
                is_writable: true,
            });
        }

        let accounts = base_accounts()
            .into_iter()
            .chain(remaining_accounts.into_iter())
            .collect();

        let instruction = Instruction {
            data: data.clone(),
            accounts,
            program_id: *program_id,
        };

        let sig = send_tx_with_retry(&rpc, payer, [instruction].to_vec())?;
        last_sig = Some(sig);
        println!("settled bets from {} to {} with sig {}", start, end, sig);

        start = end.saturating_add(1);
    }

    Ok(last_sig.expect("no signatures returned"))
}

pub fn capture_end_price(
    rpc: &Rpc,
    payer: &Keypair,
    config_pda: &Pubkey,
    round_pda: &Pubkey,
    round: &RoundAccount,
    push_oracle_program_id: &Pubkey,
    system_program_id: &Pubkey,
    program_id: &Pubkey,
) -> Result<Vec<Signature>> {
    if !matches!(round.market_type, MarketType::GroupBattle) {
        bail!("capture_end_price only supported for group battle rounds");
    }

    if round.total_groups < 1 {
        bail!("capture_end_price requires at least one group");
    }

    let max_remaining_accounts = rpc.max_remaining_accounts();

    let mut sigs: Vec<Signature> = Vec::new();

    let data = sighash_global("capture_end_price").to_vec();

    for group_id in 1..=round.total_groups {
        let group_asset_pda = derive_group_asset_pda(program_id, round_pda, group_id);
        println!("capturing end price for group {}", group_asset_pda);
        let group_asset = get_group_asset_account(rpc.client(), program_id, &group_asset_pda)?;
        if group_asset.total_assets < 1 {
            println!("group {} has no assets", group_id);
            continue;
        }
        if group_asset.captured_end_price_assets == group_asset.total_assets {
            println!("group {} already captured end price", group_id);
            continue;
        }

        let base_accounts: Vec<AccountMeta> = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(*config_pda, false),
            AccountMeta::new(*round_pda, false),
            AccountMeta::new(group_asset_pda, false),
            AccountMeta::new_readonly(*system_program_id, false),
        ];

        let per_asset_accounts = 2;
        let max_assets_per_batch = (max_remaining_accounts / per_asset_accounts).max(1);
        let total_assets = group_asset.total_assets as usize;
        let mut start_asset = 1usize;
        while start_asset <= total_assets {
            let end_asset = (start_asset + max_assets_per_batch - 1).min(total_assets);
            println!(
                "capturing end price for assets from {} to {} in batch of {}",
                start_asset, end_asset, max_assets_per_batch
            );

            let mut remaining_accounts: Vec<AccountMeta> =
                Vec::with_capacity((end_asset - start_asset + 1) * per_asset_accounts);
            for asset_id in start_asset..=end_asset {
                let asset_pda = derive_asset_pda(program_id, &group_asset_pda, asset_id as u64);
                remaining_accounts.push(AccountMeta {
                    pubkey: asset_pda,
                    is_signer: false,
                    is_writable: true,
                });

                let asset = get_asset_account(rpc.client(), &asset_pda, program_id)?;
                let price_feed_account =
                    get_price_feed_account(0, &hex::encode(asset.feed_id), push_oracle_program_id)?;
                remaining_accounts.push(AccountMeta {
                    pubkey: price_feed_account,
                    is_signer: false,
                    is_writable: false,
                });
            }

            let mut accounts = base_accounts.clone();
            accounts.reserve(remaining_accounts.len());
            accounts.extend(remaining_accounts.into_iter());

            let instruction = Instruction {
                data: data.clone(),
                accounts,
                program_id: *program_id,
            };

            let sig = send_tx_with_retry(&rpc, payer, [instruction].to_vec())?;
            println!(
                "captured end price for assets from {} to {}: {}",
                start_asset, end_asset, sig
            );
            sigs.push(sig);

            start_asset = end_asset.saturating_add(1);
        }
    }

    Ok(sigs)
}

pub fn settle_group_round(
    rpc: &Rpc,
    payer: &Keypair,
    config_pda: &Pubkey,
    round_pda: &Pubkey,
    round_vault: &Pubkey,
    round: &RoundAccount,
    gold_price_feed: &Pubkey,
    treasury: &Pubkey,
    treasury_token_account: &Pubkey,
    token_mint: &Pubkey,
    token_program_id: &Pubkey,
    associated_token_program_id: &Pubkey,
    system_program_id: &Pubkey,
    program_id: &Pubkey,
) -> Result<Signature> {
    return Err(anyhow::anyhow!("settle_group_round not implemented"));
}
