use crate::config::RuntimeConfig;
use anyhow::Result;
use keeper_lib::{
    client::{
        anchor::{get_config_account, get_price_feed_account},
        rpc::Rpc,
    },
    pda::derive_token_account_pda,
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
