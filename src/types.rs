use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main conductor configuration struct
/// This is the root of the configuration tree / aggregates
/// all other configuration aspects.
///
/// References between structs (instance configs pointing to
/// the agent and DNA to be instantiated) are implemented
/// via string IDs.
#[derive(Deserialize, Serialize, Clone, Default, Debug)]
pub struct Configuration {
    /// List of DNAs, for each a path to the DNA file. Optional.
    #[serde(default)]
    pub dnas: Vec<DnaConfiguration>,
    /// List of instances, includes references to an agent and a DNA. Optional.
    #[serde(default)]
    pub instances: Vec<InstanceConfiguration>,
    /// List of interfaces any UI can use to access zome functions. Optional.
    #[serde(default)]
    pub interfaces: Vec<InterfaceConfiguration>,
    /// where to persist the config file and DNAs.
    pub persistence_dir: PathBuf,
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
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
        for dna in self.dnas.iter_mut() {
            let filename = &dna.file.file_name().ok_or(anyhow!(
                "dna {} with path {} has no filename",
                &dna.id,
                &dna.file.display()
            ))?;
            let to_path = self.persistence_dir.join(&filename);

            fs::copy(&dna.file, &to_path)?;
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

    // POST Holo-hosted hApp URLs to resolver
    pub fn update_happ2host(&self) -> Result<()> {
        let happ_urls = self
            .dnas
            .clone()
            .into_iter()
            .filter(|dna| dna.holo_hosted)
            .filter_map(|dna| dna.happ_url)
            .collect::<Vec<String>>();
        let response =
            ureq::post("https://resolver.holohost.net/update/addHost")
                .send_string(&serde_json::to_string(&happ_urls)?);
        if response.error() {
            return Err(anyhow!("request to resolver failed"));
        }
        Ok(())
    }

    // --- COPIED ---

    /// Returns the DNA configuration with the given ID if present
    pub fn dna_by_id(&self, id: &str) -> Option<DnaConfiguration> {
        self.dnas.iter().find(|dc| &dc.id == id).cloned()
    }
}

/// A DNA is represented by a DNA file.
/// A hash can optionally be provided, which could be used to validate that the DNA being installed
/// is the DNA that was intended to be installed.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DnaConfiguration {
    pub id: String,
    pub file: PathBuf,
    pub hash: String,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub holo_hosted: bool,
    /// ALPHA: URL of Holo-hosted hApp.
    #[serde(default)]
    pub happ_url: Option<String>,
}

/// An instance combines a DNA with an agent.
/// Each instance has its own storage configuration.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct InstanceConfiguration {
    pub id: String,
    pub dna: String,
    pub agent: String,
    pub storage: StorageConfiguration,
    /// `false` for self-hosted DNAs, `true` for Holo-hosted
    #[serde(default)]
    pub holo_hosted: bool,
}

/// This configures the Content Addressable Storage (CAS) that
/// the instance uses to store source chain and DHT shard in.
/// There are two storage implementations in cas_implementations so far:
/// * memory
/// * file
///
/// Projected are various DB adapters.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StorageConfiguration {
    Memory,
    File {
        path: String,
    },
    Pickle {
        path: String,
    },
    Lmdb {
        path: String,
        initial_mmap_bytes: Option<usize>,
    },
}

/// Here, interfaces are user facing and make available zome functions to
/// GUIs, browser based web UIs, local native UIs, other local applications and scripts.
/// We currently have:
/// * websockets
/// * HTTP
///
/// We will also soon develop
/// * Unix domain sockets
///
/// The instances (referenced by ID) that are to be made available via that interface should be listed.
/// An admin flag will enable conductor functions for programatically changing the configuration
/// (e.g. installing apps)
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct InterfaceConfiguration {
    pub id: String,
    pub driver: InterfaceDriver,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub instances: Vec<InstanceReferenceConfiguration>,
    /// Experimental!
    /// If this flag is set the conductor might change the port the interface binds to if the
    /// given port is occupied. This might cause problems if the context that runs the conductor
    /// is not aware of this logic and is not tracking the new port (which gets printed on stdout).
    /// Use at your own risk...
    pub choose_free_port: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InterfaceDriver {
    Websocket { port: u16 },
    Http { port: u16 },
    DomainSocket { file: String },
    Custom(toml::value::Value),
}

/// An instance reference makes an instance available in the scope
/// of an interface.
/// Since UIs usually hard-code the name with which they reference an instance,
/// we need to decouple that name used by the UI from the internal ID of
/// the instance. That is what the optional `alias` field provides.
/// Given that there is 1-to-1 relationship between UIs and interfaces,
/// by setting an alias for available instances in the UI's interface
/// each UI can have its own unique handle for shared instances.
#[derive(Deserialize, Serialize, Default, Clone, Debug, PartialEq)]
pub struct InstanceReferenceConfiguration {
    /// ID of the instance that is made available in the interface
    pub id: String,

    /// A local name under which the instance gets mounted in the
    /// interface's scope
    pub alias: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
