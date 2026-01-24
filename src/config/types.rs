use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,
    #[serde(default)]
    pub presets: HashMap<String, CustomPreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub api_user: String,
    pub api_key: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub client_ip: Option<String>,
    #[serde(default = "default_sandbox")]
    pub sandbox: bool,
}

fn default_sandbox() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPreset {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub records: Vec<PresetRecord>,
    #[serde(default)]
    pub variables: Vec<PresetVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetRecord {
    #[serde(rename = "type")]
    pub record_type: String,
    pub host: String,
    pub value: String,
    #[serde(default)]
    pub ttl: Option<u32>,
    #[serde(default)]
    pub priority: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetVariable {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub profile: Profile,
    pub profile_name: String,
    pub presets: HashMap<String, CustomPreset>,
}

impl Config {
    pub fn api_url(&self) -> &str {
        if self.profile.sandbox {
            "https://api.sandbox.namecheap.com/xml.response"
        } else {
            "https://api.namecheap.com/xml.response"
        }
    }

    pub fn username(&self) -> &str {
        self.profile
            .username
            .as_deref()
            .unwrap_or(&self.profile.api_user)
    }
}
