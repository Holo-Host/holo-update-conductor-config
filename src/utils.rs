use anyhow::{Context, Result};
use ed25519_dalek::{PublicKey, SecretKey};
use hpos_config_core::*;
use std::path::Path;
use std::{env, fs};

pub fn set_write_permissions(path: &Path) -> Result<()> {
    let metadata = fs::metadata(&path).with_context(|| {
        format!("failed to read metadata for {}", &path.display())
    })?;
    let mut perms = metadata.permissions();
    perms.set_readonly(false);

    fs::set_permissions(&path, perms).with_context(|| {
        format!("failed to set write permissions on {}", &path.display())
    })?;
    Ok(())
}

pub fn get_host_id() -> Result<String> {
    let hpos_config_path = env::var("HPOS_CONFIG_PATH")?;
    let contents = fs::read(&hpos_config_path)?;
    let Config::V1 { seed, .. } = serde_json::from_slice(&contents)?;

    let secret_key = SecretKey::from_bytes(&seed)?;
    let public_key = PublicKey::from(&secret_key);

    Ok(public_key::to_base36_id(&public_key))
}
