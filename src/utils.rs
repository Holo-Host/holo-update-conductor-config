use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

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
