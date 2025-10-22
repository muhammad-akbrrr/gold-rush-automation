use crate::types::enums::{MarketType, RoundStatus};
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RoundAccount {
    pub id: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub bet_cutoff_time: i64,
    pub vault: Pubkey,
    pub vault_bump: u8,
    pub market_type: MarketType,
    pub status: RoundStatus,
    pub start_price: Option<u64>,
    pub final_price: Option<u64>,
    pub total_pool: u64,
    pub total_bets: u64,
    pub total_fee_collected: u64,
    pub total_reward_pool: u64,
    pub winners_weight: u64,
    pub settled_bets: u64,
    pub cancelled_bets: u64,
    pub winner_group_ids: Vec<u64>,
    pub total_groups: u64,
    pub captured_start_groups: u64,
    pub captured_end_groups: u64,
    pub created_at: i64,
    pub settled_at: Option<i64>,
    pub bump: u8,
}
