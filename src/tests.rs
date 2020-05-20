use super::*;

use std::env;

static OLD_CONFIG_TOML: &str =
    include_str!("../resources/test/old-conductor-config.toml");
static NEW_CONFIG_TOML: &str =
    include_str!("../resources/test/new-conductor-config.toml");
static NIX_CONFIG_TOML: &str =
    include_str!("../resources/test/nix-conductor-config.toml");
static EXPECTED_OLD_CONFIG_TOML: &str =
    include_str!("../resources/test/expected-old-conductor-config.toml");
static EXPECTED_NEW_CONFIG_TOML: &str =
    include_str!("../resources/test/expected-new-conductor-config.toml");

#[test]
fn test_deserialize() {
    let config = Configuration::from_toml(EXPECTED_OLD_CONFIG_TOML)
        .expect("deserialization failed");
    assert_eq!(
        config.persistence_dir,
        PathBuf::from("/var/lib/holochain-conductor")
    )
}

#[test]
/// This test is checking if rebuild from old-style configuration file won't corrupt resulting config file
/// Old style means config file created by holo-nixpkgs before merge of PR #439
fn update_from_old_config_smoke() {
    // existing config on HPOS in an old format (without holo_hosted dna and instance property)
    let old_config = Configuration::from_toml(OLD_CONFIG_TOML).unwrap();

    // data provided by nixOs at systemd preStart
    let mut nix_config = Configuration::from_toml(NIX_CONFIG_TOML).unwrap();

    // resulting config after merge operation
    let expected_config =
        Configuration::from_toml(EXPECTED_OLD_CONFIG_TOML).unwrap();

    // merge configs
    nix_config.persist_state_from(&old_config);

    // compare results
    assert_eq!(
        nix_config, expected_config,
        "Configuration after an update is different from expected."
    );
}

#[test]
/// This test is checking if rebuild from new-style configuration transfers holo-hosted instances correctly
/// New style means config file created by holo-nixpkgs after merge of PR #439
fn update_from_new_config_smoke() {
    // existing config on HPOS in an old format (without holo_hosted dna and instance property)
    let new_config = Configuration::from_toml(NEW_CONFIG_TOML).unwrap();

    // data provided by nixOs at systemd preStart
    let mut nix_config = Configuration::from_toml(NIX_CONFIG_TOML).unwrap();

    // resulting config after merge operation
    let expected_config =
        Configuration::from_toml(EXPECTED_NEW_CONFIG_TOML).unwrap();

    // merge configs
    nix_config.persist_state_from(&new_config);

    // compare results
    assert_eq!(
        nix_config, expected_config,
        "Configuration after an update is different from expected."
    );
}

#[test]
/// test if host_id can be retrieved from HPOS_CONFIG_PATH env var
fn retrieve_host_id() {
    let path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let hpos_config_path = format!("{}/resources/test/hpos-config.json", path);
    env::set_var("HPOS_CONFIG_PATH", &hpos_config_path);

    let host_id = crate::utils::get_host_id().unwrap();
    assert_eq!(
        host_id,
        String::from("5m5srup6m3b2iilrsqmxu6ydp8p8cr0rdbh4wamupk3s4sxqr5")
    );
}
