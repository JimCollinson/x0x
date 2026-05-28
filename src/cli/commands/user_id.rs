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
pub async fn create(output: Option<PathBuf>) -> Result<PathBuf> {
    let path = match output {
        Some(path) => path,
        None => default_user_key_path()?,
    };

    let keypair = UserKeypair::generate()
        .map_err(|e| anyhow::anyhow!("failed to generate user keypair: {e}"))?;
    save_user_keypair_to(&keypair, &path)
        .await
        .with_context(|| format!("failed to write user keypair to {}", path.display()))?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::load_user_keypair_from;
    use anyhow::ensure;

    #[tokio::test]
    async fn create_writes_loadable_user_keypair() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let path = temp_dir.path().join("user.key");

        let resolved = create(Some(path.clone())).await?;

        ensure!(
            resolved == path,
            "resolved path should match requested path"
        );
        ensure!(path.is_file(), "user key file should exist");
        ensure!(
            path.metadata()?.len() > 0,
            "user key file should not be empty"
        );
        let loaded = load_user_keypair_from(&path).await?;
        let _user_id = loaded.user_id();

        Ok(())
    }
}
