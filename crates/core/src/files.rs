use std::path::PathBuf;

use anyhow::{Ok, Result};
use tokio::fs;

pub async fn save_to_file(content: &[u8], path: &PathBuf) -> Result<()> {
    fs::write(path, content).await?;
    Ok(())
}
