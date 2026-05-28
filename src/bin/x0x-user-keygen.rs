//! Deprecated compatibility shim for `x0x user-id create`.
//!
//! This binary remains buildable from source for scripts that invoke it
//! directly. New code should use `x0x user-id create [PATH]`.

use anyhow::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let output = std::env::args().nth(1).map(PathBuf::from);
    x0x::cli::commands::user_id::create(output).await
}
