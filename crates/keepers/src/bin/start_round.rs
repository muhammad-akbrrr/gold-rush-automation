use anyhow::Result;
use keeper_lib::client::anchor::{get_config_account, program_id_from_env};
use keeper_lib::client::rpc::Rpc;

fn main() -> Result<()> {
    let rpc = Rpc::from_env()?;
    let program_id = program_id_from_env()?;
    let cfg = get_config_account(rpc.client(), &program_id)?;

    println!(
        "Config loaded: admin={}, current_round_counter={}, version={}",
        cfg.admin, cfg.current_round_counter, cfg.version
    );

    Ok(())
}
