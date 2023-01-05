use anyhow::{Ok, Result};
use crypto::{exporter::export_private_key_bytes, signer::Signer};
use log::{debug, info};
use std::path::PathBuf;

use crate::files::save_to_file;

pub async fn export_private_key(
    signer: &Signer,
    master_password: String,
    export_file_name: PathBuf,
) -> Result<()> {
    let armored = signer.export_private_key()?;
    let exported = export_private_key_bytes(armored, master_password.to_string())?;
    debug!(
        "saving to: {}",
        export_file_name
            .clone()
            .to_str()
            .expect("expect to get path")
    );
    save_to_file(&exported, &export_file_name).await?;
    Ok(())
}
