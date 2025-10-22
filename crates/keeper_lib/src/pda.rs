use solana_sdk::pubkey::Pubkey;

pub fn derive_config_pda(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"config"], program_id).0
}

pub fn derive_round_pda(program_id: &Pubkey, round_id: u64) -> Pubkey {
    Pubkey::find_program_address(&[b"round", &round_id.to_le_bytes()], program_id).0
}

pub fn derive_round_vault_pda(program_id: &Pubkey, round_pda: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault", round_pda.as_ref()], program_id).0
}

pub fn derive_group_asset_pda(program_id: &Pubkey, round_pda: &Pubkey, group_id: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[b"group_asset", round_pda.as_ref(), &group_id.to_le_bytes()],
        program_id,
    )
    .0
}

pub fn derive_asset_pda(program_id: &Pubkey, group_asset_pda: &Pubkey, asset_id: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[b"asset", group_asset_pda.as_ref(), &asset_id.to_le_bytes()],
        program_id,
    )
    .0
}

pub fn derive_bet_pda(program_id: &Pubkey, round_pda: &Pubkey, bet_id: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[b"bet", round_pda.as_ref(), &bet_id.to_le_bytes()],
        program_id,
    )
    .0
}

pub fn derive_token_account_pda(
    token_program_id: &Pubkey,
    associated_token_program_id: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(
        &[owner.as_ref(), token_program_id.as_ref(), mint.as_ref()],
        associated_token_program_id,
    )
    .0
}
