use anyhow::Result;
use keeper_lib::client::anchor::{get_rounds_by_ids, start_round};
use keeper_lib::pda::derive_round_pda;
use keeper_lib::types::{MarketType, RoundStatus};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;

use crate::App;

pub fn run_one(app: &App) -> Result<Vec<Signature>> {
    let cfg = app.fetch_config()?;
    let now = chrono::Utc::now().timestamp();
    let mut sigs = Vec::new();

    if cfg.current_round_counter == 0 {
        return Ok(sigs);
    }

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
            println!("round start time: {}", round.start_time);
            println!("now: {}", now);
            println!("round market type: {:?}", round.market_type);
            if matches!(round.status, RoundStatus::Scheduled) && round.start_time <= now {
                let round_pda = derive_round_pda(&app.program_id, round.id);

                let sig_res = match round.market_type {
                    MarketType::SingleAsset => start_single_round(app, &round_pda),
                    MarketType::GroupBattle => start_group_round(app, &round_pda),
                };
                match sig_res {
                    Ok(sig) => {
                        println!("started round {}: {}", round.id, sig);
                        sigs.push(sig);
                    }
                    Err(err) => {
                        eprintln!("start_round failed for round {}: {:#}", round.id, err);
                        continue;
                    }
                }
            }
        }

        start = upper.saturating_add(1);
    }

    Ok(sigs)
}

fn start_single_round(app: &App, round_pda: &Pubkey) -> Result<Signature> {
    println!("starting single round {}", round_pda);
    start_round(
        app.rpc.client(),
        app.signer(),
        &round_pda,
        &app.gold_price_feed,
        &app.system_program_id,
        &app.program_id,
    )
    .map_err(|err| anyhow::anyhow!("start_round failed for {}: {:#}", round_pda, err))
}

fn start_group_round(app: &App, round_pda: &Pubkey) -> Result<Signature> {
    println!("starting group round {}", round_pda);
    return Err(anyhow::anyhow!("start_group_round not implemented"));
}

fn capture_start_price() -> Result<()> {
    todo!()
}

fn finalize_start_group_assets() -> Result<()> {
    todo!()
}

fn finalize_start_groups() -> Result<()> {
    todo!()
}
