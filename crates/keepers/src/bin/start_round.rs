use anyhow::Result;
use keepers::{App, config};
use std::time::Duration;
use tokio::time::{Instant, MissedTickBehavior, interval_at};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::load()?;

    let period = Duration::from_secs(cfg.start_round_period_in_secs);
    let start = Instant::now();
    let mut ticker = interval_at(start, period);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let app = App::init_from(cfg)?;

    loop {
        ticker.tick().await;

        match keepers::keepers::start_round::run_one(&app) {
            Ok(sigs) => {
                if !sigs.is_empty() {
                    println!("started {} rounds", sigs.len());
                }
            }
            Err(e) => {
                eprintln!("run_one error: {:#}", e)
            }
        }
    }
}
