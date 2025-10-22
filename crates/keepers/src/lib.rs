use crate::config::RuntimeConfig;
use anyhow::Result;
use keeper_lib::{
    client::{
        anchor::{get_config_account, get_price_feed_account},
        rpc::Rpc,
    },
    pda::derive_token_account_pda,
    storage::sqlite::{SQLiteLogConfig, init_global_logger},
    types::config_account::ConfigAccount,
    wallet::load_keypair_from_file,
};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

pub mod config;
pub mod keepers;
pub mod logging;

pub struct App {
    rpc: Rpc,
    signer: Arc<Keypair>,
    treasury: Pubkey,
    treasury_token_account: Pubkey,
    gold_price_feed: Pubkey,
    token_mint: Pubkey,
    token_program_id: Pubkey,
    associated_token_program_id: Pubkey,
    push_oracle_program_id: Pubkey,
    system_program_id: Pubkey,
    program_id: Pubkey,
}

impl App {
    pub fn init_from(cfg: RuntimeConfig) -> Result<Self> {
        let rpc = Rpc::new(
            &cfg.solana_rpc_url,
            cfg.rpc_timeout_ms,
            cfg.commitment,
            cfg.preflight,
            cfg.tx_max_retries,
            cfg.cu_limit,
            cfg.cu_price_micro_lamports,
            cfg.backoff_ms,
            cfg.max_remaining_accounts,
        );
        let signer = Arc::new(load_keypair_from_file(&cfg.keeper_keypair_path)?);
        let gold_price_feed =
            get_price_feed_account(0, &cfg.gold_price_feed_id, &cfg.push_oracle_program_id)?;
        let treasury_token_account = derive_token_account_pda(
            &cfg.token_program_id,
            &cfg.associated_token_program_id,
            &cfg.treasury,
            &cfg.token_mint,
        );

        let keeper_instance_id = cfg
            .keeper_instance_id
            .clone()
            .unwrap_or_else(|| format!("pid:{}", std::process::id()));

        if cfg.persist_logs {
            let log_cfg = SQLiteLogConfig {
                path: cfg.log_db_path.clone(),
                batch_max: cfg.log_batch_max,
                batch_ms: cfg.log_batch_ms,
                queue_cap: cfg.log_queue_cap,
                retention_days: cfg.log_retention_days,
                keeper_instance_id: keeper_instance_id.clone(),
            };
            init_global_logger(log_cfg.clone());
            // Store default instance id for entries that don't set it
            keeper_lib::storage::sqlite::set_default_instance_id(keeper_instance_id.clone());
        }

        Ok(Self {
            rpc,
            signer,
            treasury: cfg.treasury,
            treasury_token_account,
            gold_price_feed: gold_price_feed,
            token_mint: cfg.token_mint,
            token_program_id: cfg.token_program_id,
            associated_token_program_id: cfg.associated_token_program_id,
            push_oracle_program_id: cfg.push_oracle_program_id,
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
