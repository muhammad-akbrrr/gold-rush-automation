use anyhow::Result;
use keeper_lib::client::anchor::{get_rounds_by_ids, start_round};
use keeper_lib::pda::derive_round_pda;
use keeper_lib::types::RoundStatus;
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
        for round in rounds {
            if matches!(round.status, RoundStatus::Scheduled) && round.start_time <= now {
                let round_pda = derive_round_pda(&app.program_id, round.id);

                match start_round(
                    app.rpc.client(),
                    app.signer(),
                    &round_pda,
                    &app.gold_price_feed,
                    &app.system_program_id,
                    &app.program_id,
                ) {
                    Ok(sig) => sigs.push(sig),
                    Err(err) => {
                        eprintln!("start_round failed for {}: {:#}", round.id, err);
                    }
                }
            }
        }

        start = upper.saturating_add(1);
    }

    Ok(sigs)
}
