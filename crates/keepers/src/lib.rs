use crate::config::RuntimeConfig;
use anyhow::Result;
use keeper_lib::{
    client::{
        anchor::{get_config_account, get_price_feed_account},
        rpc::Rpc,
    },
    types::ConfigAccount,
    wallet::load_keypair_from_file,
};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

pub mod config;
pub mod keepers;

pub struct App {
    rpc: Rpc,
    signer: Arc<Keypair>,
    gold_price_feed: Pubkey,
    system_program_id: Pubkey,
    program_id: Pubkey,
}

impl App {
    pub fn init_from(cfg: RuntimeConfig) -> Result<Self> {
        let rpc = Rpc::new(&cfg.solana_rpc_url);
        let signer = Arc::new(load_keypair_from_file(&cfg.keeper_keypair_path)?);
        println!("gold_price_feed_id: {:?}", &cfg.gold_price_feed_id);
        println!("push_oracle_program_id: {:?}", &cfg.push_oracle_program_id);
        let gold_price_feed =
            get_price_feed_account(0, &cfg.gold_price_feed_id, &cfg.push_oracle_program_id)?;
        println!("Gold price feed: {:?}", gold_price_feed);

        Ok(Self {
            rpc,
            signer,
            gold_price_feed: gold_price_feed,
            system_program_id: cfg.system_program_id,
            program_id: cfg.program_id,
        })
    }

    pub fn signer(&self) -> &Keypair {
        &self.signer
    }

    pub fn fetch_config(&self) -> Result<ConfigAccount> {
        let cfg = get_config_account(self.rpc.client(), &self.program_id)?;
        Ok(cfg)
    }
}
