use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ProgramStatus {
    Active,
    Paused,
    EmergencyPaused,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum MarketType {
    SingleAsset,
    GroupBattle,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum RoundStatus {
    Scheduled,
    Active,
    Cancelling,
    PendingSettlement,
    Ended,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum BetDirection {
    Up,
    Down,
    PercentageChangeBps(i16),
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum BetStatus {
    Pending,
    Won,
    Lost,
    Draw,
}
