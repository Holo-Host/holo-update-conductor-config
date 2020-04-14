mod types;

use anyhow::{Context, Result};
use std::io::{self, Read};
use types::Configuration;

fn main() -> Result<()> {
    // Holochain settings are read from stdin into a struct new-config
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    let mut new_config = Configuration::from_toml(&input)
        .context("stdin is not a valid conductor config")?;

    let config_path = new_config.persistence_dir.join("conductor-config.toml");

    // existing conductor-config.toml is loaded into struct old-config
    let old_config =
        std::fs::read_to_string(&config_path).with_context(|| {
            format!(
                "failed to read old config file at {}",
                &config_path.display()
            )
        })?;
    let old_config = Configuration::from_toml(&old_config)
        .context("failed to parse old_config")?;

    // all the DNAs in new-config are copied from derivations to conductor's working directory
    // dnas.file in new-config is updated to new location of DNAs
    new_config.copy_dnas_to_persistence_dir().with_context(|| {
        format!(
            "failed to copy DNAs to persistence_dir ({})",
            new_config.persistence_dir.display()
        )
    })?;

    // new-config gets updated with selected values from old-config
    new_config.update_with(&old_config);

    // new-config is written to stdout
    let new_config_toml = new_config
        .to_toml()
        .context("failed to serialize new_config to TOML")?;
    println!("{}", new_config_toml);

    // (in alpha) KV store HAPP2HOST is updated with values of all holo-hosted hApps

    // TODO: enable when resolver is up
    // new_config
    //     .update_happ2host()
    //     .context("failed to update HAPP2HOST")?;

    Ok(())
}
