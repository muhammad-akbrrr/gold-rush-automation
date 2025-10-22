use crate::types::enums::ProgramStatus;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ConfigAccount {
    pub admin: Pubkey,
    pub keeper_authorities: Vec<Pubkey>,
    pub token_mint: Pubkey,
    pub treasury: Pubkey,
    pub single_asset_feed_id: [u8; 32],
    pub max_price_update_age_secs: u64,
    pub fee_single_asset_bps: u16,
    pub fee_group_battle_bps: u16,
    pub min_bet_amount: u64,
    pub bet_cutoff_window_secs: i64,
    pub min_time_factor_bps: u16,
    pub max_time_factor_bps: u16,
    pub default_direction_factor_bps: u16,
    pub status: ProgramStatus,
    pub current_round_counter: u64,
    pub version: u8,
    pub bump: u8,
}
