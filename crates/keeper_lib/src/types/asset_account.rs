use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AssetAccount {
    pub id: u64,
    pub group: Pubkey,
    pub round: Pubkey,
    pub feed_id: [u8; 32],
    pub symbol: [u8; 8],
    pub start_price: Option<u64>,
    pub final_price: Option<u64>,
    pub growth_rate_bps: Option<i64>,
    pub created_at: i64,
    pub started_price_at: Option<i64>,
    pub finalized_at: Option<i64>,
    pub bump: u8,
}
