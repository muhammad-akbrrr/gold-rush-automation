use anyhow::Result;
use keepers::{App, config};
use std::time::Duration;
use tokio::time::{Instant, MissedTickBehavior, interval_at};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::load()?;
    keepers::logging::init_tracing(&cfg);

    let period = Duration::from_secs(cfg.settle_round_period_in_secs);
    let start = Instant::now();
    let mut ticker = interval_at(start, period);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let app = App::init_from(cfg)?;

    loop {
        ticker.tick().await;

        match keepers::keepers::settle_round::run_one(&app) {
            Ok(sigs) => {
                if !sigs.is_empty() {
                    info!(settled_rounds = sigs.len(), "settled rounds");
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "run_one error");
            }
        }
    }
}
