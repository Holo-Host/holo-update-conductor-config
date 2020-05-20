//! Most of the structs are copied from
//! `holochain_core_types` and `holochain_conductor_lib`.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone, Default, PartialEq, Debug)]
pub struct Configuration {
    #[serde(default)]
    dnas: Vec<DnaConfiguration>,
    #[serde(default)]
    instances: Vec<InstanceConfiguration>,
    #[serde(default)]
    interfaces: Vec<InterfaceConfiguration>,
    pub persistence_dir: PathBuf,
    #[serde(flatten)]
    extra: HashMap<String, toml::Value>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Payload {
    host_id: String,
    happ_ids: Vec<String>,
}

impl Configuration {
    // --- ADDED ---

    pub fn from_toml(toml: &str) -> Result<Self> {
        Ok(toml::from_str(toml)?)
    }

    pub fn to_toml(&self) -> Result<String> {
        let value = toml::Value::try_from(&self)?;
        let string = toml::to_string_pretty(&value)?;
        Ok(string)
    }

    /// Copy only holo-hosted DNA files to `persistence_dir` and update their `file` values.
    pub fn copy_dnas_to_persistence_dir(
        &mut self,
        persistence_dir: Option<PathBuf>,
    ) -> Result<()> {
        let dnas_dir = persistence_dir
            .unwrap_or_else(|| self.persistence_dir.join("dnas"));
        if !dnas_dir.is_dir() {
            fs::create_dir(&dnas_dir).with_context(|| {
                format!("failed to create dnas dir ({})", &dnas_dir.display())
            })?;
        }

        for dna in self.dnas.iter_mut() {
            if !&dna.holo_hosted {
                continue;
            }

            let filename = dna.hash.to_owned() + ".dna.json";
            let to_path = dnas_dir.join(&filename);

            fs::copy(&dna.file, &to_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    &dna.file.display(),
                    &to_path.display()
                )
            })?;

            crate::utils::set_write_permissions(&to_path)?;
            dna.file = to_path;
        }
        Ok(())
    }

    /// Update `self` with selected values from `other`.
    pub fn persist_state_from(&mut self, other: &Self) {
        for instance in other.instances.iter() {
            if instance.holo_hosted {
                self.instances.push(instance.clone());
            }
        }
        self.attach_all_instances();
    }

    fn attach_all_instances(&mut self) {
        // HACK: borrowck, please...
        let instances = self.instances.clone();
        for instance in instances.iter() {
            if instance.holo_hosted && self.has_dna(&instance.dna) {
                self.attach(&instance.id, "hosted-interface");
            }
        }
    }

    fn has_dna(&self, id: &str) -> bool {
        self.dna_by_id(&id).is_some()
    }

    fn attach(&mut self, instance_id: &str, interface_id: &str) {
        if let Some(interface) = self
            .interfaces
            .iter_mut()
            .find(|interface| interface.id == interface_id)
        {
            interface.instances.push(InstanceReferenceConfiguration {
                id: instance_id.to_string(),
                ..InstanceReferenceConfiguration::default()
            })
        }
    }

    /// POST Holo-hosted hApp URLs to resolver
    pub fn update_happ2host(&self) -> Result<()> {
        use std::{thread, time::Duration};

        let mut retries = 1;
        let delay = Duration::from_millis(1000);
        let url = "https://resolver.holohost.net/update/addHost";

        let host_id = crate::utils::get_host_id()?;

        let happ_ids = self
            .dnas
            .iter()
            .cloned()
            .filter(|dna| dna.holo_hosted)
            .map(|dna| dna.id)
            .collect::<Vec<String>>();

        let payload = Payload { host_id, happ_ids };
        let payload = serde_json::to_value(&payload)?;

        let response = loop {
            let response = ureq::post(url).send_json(payload.clone());
            retries -= 1;
            if retries <= 0 || response.ok() {
                break response;
            }
            thread::sleep(delay);
        };

        if response.error() {
            return Err(anyhow!(
                "request to resolver failed: {}",
                response.status_line()
            ));
        }
        Ok(())
    }

    // --- COPIED ---

    /// Returns the DNA configuration with the given ID if present
    fn dna_by_id(&self, id: &str) -> Option<DnaConfiguration> {
        self.dnas.iter().find(|dc| dc.id == id).cloned()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct DnaConfiguration {
    id: String,
    file: PathBuf,
    hash: String,
    #[serde(default)]
    holo_hosted: bool,
    /// ALPHA: URL of Holo-hosted hApp.
    #[serde(default)]
    happ_url: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, toml::Value>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct InstanceConfiguration {
    id: String,
    dna: String,
    #[serde(default)]
    holo_hosted: bool,
    #[serde(flatten)]
    extra: HashMap<String, toml::Value>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
struct InterfaceConfiguration {
    id: String,
    #[serde(default)]
    instances: Vec<InstanceReferenceConfiguration>,
    #[serde(flatten)]
    extra: HashMap<String, toml::Value>,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
struct InstanceReferenceConfiguration {
    id: String,
    #[serde(flatten)]
    extra: HashMap<String, toml::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    static NIX_CONFIG_TOML: &str =
        include_str!("../resources/test/nix-conductor-config.toml");

    #[test]
    /// Test copy_dnas_to_persistence_dir
    fn copy_dnas_and_update_config() {
        let mut config = Configuration::from_toml(NIX_CONFIG_TOML).unwrap();

        let tmp_dir = TempDir::new().unwrap();
        let tmp_path = tmp_dir.path().to_owned();

        // Construct expected paths
        let expected_hc_dna_path = config.dna_by_id("holofuel").unwrap().file;
        let mut expected_holo_dna_path = tmp_dir.path().to_owned();

        let expected_holo_dna_file = config
            .dna_by_id("QmTyogN3tbvBwb1mkeN2zgST2NnNoEU5DWupph214b32EP")
            .unwrap()
            .hash
            + ".dna.json";
        expected_holo_dna_path.push(&expected_holo_dna_file);

        config.copy_dnas_to_persistence_dir(Some(tmp_path)).unwrap();

        // assert paths
        assert_eq!(
            config.dna_by_id("holofuel").unwrap().file,
            expected_hc_dna_path
        );
        assert_eq!(
            config
                .dna_by_id("QmTyogN3tbvBwb1mkeN2zgST2NnNoEU5DWupph214b32EP")
                .unwrap()
                .file,
            expected_holo_dna_path
        );

        let file_num =
            fs::read_dir(tmp_dir.path().to_owned()).unwrap().count();

        // assert num files
        assert_eq!(file_num, 1);
    }
}
