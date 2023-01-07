use std::io::Cursor;

use anyhow::{anyhow, Result};
use ipfs_api::{IpfsApi, IpfsClient};

#[derive(Default)]
pub struct IPFSStorage {
    client: IpfsClient,
}

impl IPFSStorage {
    pub async fn save_file(&self, content: Vec<u8>) -> Result<String> {
        let data = Cursor::new(content);
        self.client
            .add(data)
            .await
            .map(|r| r.hash)
            .map_err(|ipfs_err| anyhow!("Error writing to IPFS: {}", ipfs_err.to_string()))
    }
}
