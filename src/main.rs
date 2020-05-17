mod types;
mod utils;

use anyhow::{Context, Result};
use std::io::Read;
use std::path::PathBuf;
use std::{fs, io};
use structopt::StructOpt;
use types::Configuration;

#[derive(StructOpt, Debug)]
struct Args {
    /// Path to existing conductor-config.toml.
    /// Will be created if not exists.
    config_path: PathBuf,
}

#[paw::main]
fn main(args: Args) -> Result<()> {
    // Holochain settings are read from stdin into a struct new-config
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    let mut new_config = Configuration::from_toml(&input)
        .context("stdin is not a valid conductor config")?;

    if args.config_path.exists() {
        // existing conductor-config.toml is loaded into struct old-config
        let old_config =
            fs::read_to_string(&args.config_path).with_context(|| {
                format!(
                    "failed to read old config file at {}",
                    &args.config_path.display()
                )
            })?;
        let old_config = Configuration::from_toml(&old_config)
            .context("failed to parse old_config")?;

        // new-config gets updated with selected values from old-config
        new_config.persist_state_from(&old_config);
    }

    // Holo-hosted DNAs in new-config are copied from derivations to conductor's working directory and renamed
    // dnas.file in new-config is updated to new location of DNAs
    new_config
        .copy_dnas_to_persistence_dir(None)
        .with_context(|| {
            format!(
                "failed to copy DNAs to persistence_dir ({})",
                new_config.persistence_dir.display()
            )
        })?;

    // new-config is written into conductor-config.toml file
    let new_config_toml = new_config
        .to_toml()
        .context("failed to serialize new_config to TOML")?;
    std::fs::write(&args.config_path, &new_config_toml).with_context(
        || {
            format!(
                "failed to write new_config to config_path ({})",
                &args.config_path.display()
            )
        },
    )?;
    utils::set_write_permissions(&args.config_path)?;

    // (in alpha) KV store HAPP2HOST is updated with values of all holo-hosted hApps

    // TODO: enable when resolver is up
    new_config
        .update_happ2host()
        .context("failed to update HAPP2HOST")?;

    Ok(())
}

#[cfg(test)]
mod tests;
