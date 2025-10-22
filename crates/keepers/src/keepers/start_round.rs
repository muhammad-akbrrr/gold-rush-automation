use anyhow::Result;
use chrono::Utc;
use keeper_lib::{
    client::anchor::{
        capture_start_price, finalize_start_group_assets, finalize_start_groups, get_rounds_by_ids,
        start_round,
    },
    pda::{derive_config_pda, derive_round_pda},
    types::{enums::MarketType, enums::RoundStatus, round_account::RoundAccount},
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use tracing::{debug, info, warn};

use crate::App;

pub fn run_one(app: &App) -> Result<Vec<Signature>> {
    let mut sigs: Vec<Signature> = Vec::new();

    let cfg = app.fetch_config()?;
    if cfg.current_round_counter == 0 {
        return Ok(sigs);
    }

    let config_pda = derive_config_pda(&app.program_id);

    let now = Utc::now().timestamp();
    let mut start = 1u64;
    let end = cfg.current_round_counter;
    let batch_size = 100;

    while start <= end {
        let upper = start.saturating_add(batch_size - 1).min(end);
        let ids: Vec<u64> = (start..=upper).collect();
        let rounds = get_rounds_by_ids(app.rpc.client(), &app.program_id, &ids)?;
        debug!(batch_start = start, batch_end = upper, total = rounds.len(), "fetched rounds batch");

        for round in rounds {
            debug!(round_id = round.id, status = ?round.status, start_time = round.start_time, now, market_type = ?round.market_type, "round fetched");

            if matches!(round.status, RoundStatus::Scheduled) && round.start_time <= now {
                let round_pda = derive_round_pda(&app.program_id, round.id);

                let sig_res = match round.market_type {
                    MarketType::SingleAsset => start_single_round(app, &config_pda, &round_pda),
                    MarketType::GroupBattle => {
                        start_group_round(app, &config_pda, &round_pda, &round)
                    }
                };
                match sig_res {
                    Ok(sig) => {
                        info!(round_id = round.id, tx_sig = %sig, "round started");
                        sigs.push(sig);
                    }
                    Err(err) => {
                        warn!(round_id = round.id, error = %err, "start_round failed");
                        continue;
                    }
                }
            }
        }

        start = upper.saturating_add(1);
    }

    Ok(sigs)
}

fn start_single_round(app: &App, config_pda: &Pubkey, round_pda: &Pubkey) -> Result<Signature> {
    info!(round_pda = %round_pda, "starting single round");
    start_round(
        &app.rpc,
        app.signer(),
        &config_pda,
        &round_pda,
        Some(&app.gold_price_feed),
        &app.system_program_id,
        &app.program_id,
    )
    .map_err(|err| anyhow::anyhow!("start_round failed for {}: {:#}", round_pda, err))
}

fn start_group_round(
    app: &App,
    config_pda: &Pubkey,
    round_pda: &Pubkey,
    round: &RoundAccount,
) -> Result<Signature> {
    info!(round_pda = %round_pda, "starting group round");

    // if round has not captured start groups
    if round.captured_start_groups < round.total_groups {
        // capture start price
        capture_start_price(
            &app.rpc,
            app.signer(),
            &config_pda,
            &round_pda,
            round,
            &app.push_oracle_program_id,
            &app.system_program_id,
            &app.program_id,
        )?;

        // finalize start group assets
        finalize_start_group_assets(
            &app.rpc,
            app.signer(),
            &config_pda,
            &round_pda,
            round,
            &app.system_program_id,
            &app.program_id,
        )?;

        // finalize start groups
        finalize_start_groups(
            &app.rpc,
            app.signer(),
            &config_pda,
            &round_pda,
            round,
            &app.system_program_id,
            &app.program_id,
        )?;
    }

    // start round
    start_round(
        &app.rpc,
        app.signer(),
        &config_pda,
        &round_pda,
        None,
        &app.system_program_id,
        &app.program_id,
    )
    .map_err(|err| anyhow::anyhow!("start_round failed for {}: {:#}", round_pda, err))
}
