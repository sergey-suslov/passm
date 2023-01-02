use std::path::PathBuf;

use anyhow::{anyhow, Result};
use shared::password::Password;
use tokio::fs;

pub async fn save_to_file(content: &[u8], path: &PathBuf) -> Result<()> {
    fs::write(path, content).await?;
    Ok(())
}

pub async fn read_password_bytes(path: &PathBuf) -> Result<Vec<u8>> {
    let content = fs::read(path).await?;
    Ok(content)
}

pub async fn read_passwords_from_path(path: &PathBuf) -> Result<Vec<Password>> {
    match fs::read_dir(path).await {
        Ok(mut dir) => {
            let mut entries: Vec<Password> = vec![];
            while let Some(entry) = dir.next_entry().await.unwrap() {
                if entry.file_type().await.unwrap().is_file() {
                    entries.push(Password {
                        name: entry.file_name().to_str().unwrap().to_string(),
                    })
                }
            }
            Ok(entries)
        }
        Err(_err) => return Err(anyhow!("Error reading passwords directory")),
    }
}
