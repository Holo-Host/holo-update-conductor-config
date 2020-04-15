//! Most of the structs are copied from
//! `holochain_core_types` and `holochain_conductor_lib`.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone, Default, Debug)]
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

    /// Copy all DNA files to `persistence_dir` and update their `file` values.
    pub fn copy_dnas_to_persistence_dir(&mut self) -> Result<()> {
        let dnas_dir = self.persistence_dir.join("dnas");
        fs::create_dir(&dnas_dir).with_context(|| {
            format!("failed to create dnas dir ({})", &dnas_dir.display())
        })?;

        for dna in self.dnas.iter_mut() {
            let filename = &dna.file.file_name().with_context(|| {
                format!(
                    "dna {} with path {} has no filename",
                    &dna.id,
                    &dna.file.display()
                )
            })?;
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
    pub fn update_with(&mut self, other: &Self) {
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
            } else {
                self.attach(&instance.id, "admin-interface");
            }
        }
    }

    fn has_dna(&self, id: &str) -> bool {
        self.dna_by_id(&id).is_some()
    }

    fn attach(&mut self, instance_id: &str, interface_id: &str) {
        self.interfaces
            .iter_mut()
            .find(|interface| interface.id == interface_id)
            .map(|interface| {
                interface.instances.push(InstanceReferenceConfiguration {
                    id: instance_id.to_string(),
                    ..InstanceReferenceConfiguration::default()
                })
            });
    }

    /// POST Holo-hosted hApp URLs to resolver
    pub fn update_happ2host(&self) -> Result<()> {
        let happ_urls = self
            .dnas
            .iter()
            .cloned()
            .filter(|dna| dna.holo_hosted)
            .filter_map(|dna| dna.happ_url)
            .collect::<Vec<String>>();
        let response =
            ureq::post("https://resolver.holohost.net/update/addHost")
                .send_json(serde_json::to_value(&happ_urls)?);
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
        self.dnas.iter().find(|dc| &dc.id == id).cloned()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct DnaConfiguration {
    id: String,
    file: PathBuf,
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

    // TODO: more examples
    static CONFIG_TOML: &str = include_str!("../tests/conductor-config.toml");

    #[test]
    fn test_deserialize() {
        let config = Configuration::from_toml(CONFIG_TOML)
            .expect("deserialization failed");
        assert_eq!(
            config.persistence_dir,
            PathBuf::from("/var/lib/holochain-conductor")
        )
    }

    // TODO: more tests
}
