use anyhow::Result;
use chrono::Utc;
use keeper_lib::{
    client::anchor::{
        capture_end_price, get_rounds_by_ids, settle_group_round, settle_single_round,
    },
    pda::{derive_config_pda, derive_round_pda, derive_round_vault_pda},
    types::{MarketType, RoundAccount, RoundStatus},
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};

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
        println!("found {} rounds", rounds.len());

        for round in rounds {
            println!("round {}: {:?}", round.id, round.status);
            println!("round market type: {:?}", round.market_type);
            println!("round total bets: {}", round.total_bets);
            println!("round settled bets: {}", round.settled_bets);
            println!("round start time: {}", round.start_time);
            println!("round end time: {}", round.end_time);

            if matches!(
                round.status,
                RoundStatus::Active | RoundStatus::PendingSettlement
            ) && round.end_time <= now
            {
                let round_pda = derive_round_pda(&app.program_id, round.id);
                let round_vault_pda = derive_round_vault_pda(&app.program_id, &round_pda);

                let sig_res = match round.market_type {
                    MarketType::SingleAsset => {
                        settle_single(app, &config_pda, &round_pda, &round_vault_pda, &round)
                    }
                    MarketType::GroupBattle => {
                        settle_group(app, &config_pda, &round_pda, &round_vault_pda, &round)
                    }
                };
                match sig_res {
                    Ok(sig) => {
                        println!("settled round {}: {}", round.id, sig);
                        sigs.push(sig);
                    }
                    Err(err) => {
                        eprintln!("settle_round failed for round {}: {:#}", round.id, err);
                        continue;
                    }
                }
            }
        }

        start = upper.saturating_add(1);
    }

    Ok(sigs)
}

fn settle_single(
    app: &App,
    config_pda: &Pubkey,
    round_pda: &Pubkey,
    round_vault_pda: &Pubkey,
    round: &RoundAccount,
) -> Result<Signature> {
    println!("settling single round {}", round_pda);

    settle_single_round(
        &app.rpc,
        app.signer(),
        &config_pda,
        &round_pda,
        &round_vault_pda,
        &round,
        &app.treasury,
        &app.treasury_token_account,
        &app.gold_price_feed,
        &app.token_mint,
        &app.token_program_id,
        &app.associated_token_program_id,
        &app.system_program_id,
        &app.program_id,
    )
    .map_err(|err| anyhow::anyhow!("settle_single_round failed for {}: {:#}", round_pda, err))
}

fn settle_group(
    app: &App,
    config_pda: &Pubkey,
    round_pda: &Pubkey,
    round_vault_pda: &Pubkey,
    round: &RoundAccount,
) -> Result<Signature> {
    println!("settling group round {}", round_pda);

    // Capture end price
    capture_end_price(
        &app.rpc,
        app.signer(),
        &config_pda,
        &round_pda,
        &round,
        &app.push_oracle_program_id,
        &app.system_program_id,
        &app.program_id,
    )?;

    // Finalize end group assets

    // Finalize end groups

    // Settle group round
    settle_group_round(
        &app.rpc,
        app.signer(),
        &config_pda,
        &round_pda,
        &round_vault_pda,
        &round,
        &app.treasury,
        &app.treasury_token_account,
        &app.gold_price_feed,
        &app.token_mint,
        &app.token_program_id,
        &app.associated_token_program_id,
        &app.system_program_id,
        &app.program_id,
    )
    .map_err(|err| anyhow::anyhow!("settle_group_round failed for {}: {:#}", round_pda, err))
}
