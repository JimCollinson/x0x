//! User identity CLI commands.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::identity::UserKeypair;
use crate::storage::save_user_keypair_to;

/// Default user identity key path (`~/.x0x/user.key`).
pub fn default_user_key_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".x0x").join("user.key"))
}

/// `x0x user-id create [PATH]` — create a user identity keypair on disk.
pub async fn create(output: Option<PathBuf>) -> Result<()> {
    let path = match output {
        Some(path) => path,
        None => default_user_key_path()?,
    };

    let keypair = UserKeypair::generate()
        .map_err(|e| anyhow::anyhow!("failed to generate user keypair: {e}"))?;
    save_user_keypair_to(&keypair, &path)
        .await
        .with_context(|| format!("failed to write user keypair to {}", path.display()))?;

    println!("Created user identity keypair at {}", path.display());
    Ok(())
}
