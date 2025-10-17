use anyhow::{Context, Result, bail};
use serde_json::from_str;
use solana_sdk::{signature::Keypair, signer::SeedDerivable};
use std::fs::read_to_string;

/// Load keypair from file
pub fn load_keypair_from_file(path: &str) -> Result<Keypair> {
    let contents =
        read_to_string(path).with_context(|| format!("Failed to read keypair file: {}", path))?;

    let keypair_data: Vec<u8> = from_str(&contents).context("Failed to parse keypair JSON")?;

    // validate length - keypair should be 64 bytes
    if keypair_data.len() != 64 {
        bail!(
            "Invalid keypair length: expected 64 bytes, got {}",
            keypair_data.len()
        );
    }

    // extract first 32 bytes as secret key seed
    let mut secret_bytes = [0u8; 32];
    secret_bytes.copy_from_slice(&keypair_data[..32]);

    let keypair = Keypair::from_seed(&secret_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to create keypair from seed: {}", e))?;

    Ok(keypair)
}
