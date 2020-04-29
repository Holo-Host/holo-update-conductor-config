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
    pub fn copy_dnas_to_persistence_dir(&mut self) -> Result<()> {
        let dnas_dir = self.persistence_dir.join("dnas");
        if !dnas_dir.is_dir() {
            fs::create_dir(&dnas_dir).with_context(|| {
                format!("failed to create dnas dir ({})", &dnas_dir.display())
            })?;
        }

        for dna in self.dnas.iter_mut() {
            if !&dna.holo_hosted {
                continue;
            }
<<<<<<< 895acc78c9f97e08b58f44e05ee84baf317fa2a2
=======

>>>>>>> add config consistency tests
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
        use std::{thread, time::Duration};

        let mut retries = 1;
        let delay = Duration::from_millis(1000);
        let url = "https://resolver.holohost.net/update/addHost";

        let happ_urls = self
            .dnas
            .iter()
            .cloned()
            .filter(|dna| dna.holo_hosted)
            .filter_map(|dna| dna.happ_url)
            .collect::<Vec<String>>();
        let happ_urls = serde_json::to_value(&happ_urls)?;

        let response = loop {
            let response = ureq::post(url).send_json(happ_urls.clone());
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
