mod builtin;
mod types;

pub use types::{Preset, PresetRecord, PresetVariable};

use crate::config::Config;
use crate::error::{CliError, Result};
use builtin::BuiltinPresets;

pub fn list_presets(config: &Config) -> Vec<Preset> {
    let mut presets = Vec::new();

    // Add builtin presets
    for file in BuiltinPresets::iter() {
        if file.ends_with(".toml") {
            if let Some(content) = BuiltinPresets::get(&file) {
                if let Ok(content_str) = std::str::from_utf8(content.data.as_ref()) {
                    if let Ok(preset) = toml::from_str::<Preset>(content_str) {
                        presets.push(preset);
                    }
                }
            }
        }
    }

    // Add custom presets from config
    for (name, custom) in &config.presets {
        presets.push(Preset {
            name: name.clone(),
            description: custom.description.clone(),
            records: custom
                .records
                .iter()
                .map(|r| PresetRecord {
                    record_type: r.record_type.clone(),
                    host: r.host.clone(),
                    value: r.value.clone(),
                    ttl: r.ttl,
                    priority: r.priority,
                })
                .collect(),
            variables: custom
                .variables
                .iter()
                .map(|v| PresetVariable {
                    name: v.name.clone(),
                    description: v.description.clone(),
                    default: v.default.clone(),
                    required: v.required,
                })
                .collect(),
        });
    }

    // Sort by name
    presets.sort_by(|a, b| a.name.cmp(&b.name));

    presets
}

pub fn get_preset(name: &str, config: &Config) -> Result<Preset> {
    // Check custom presets first
    if let Some(custom) = config.presets.get(name) {
        return Ok(Preset {
            name: name.to_string(),
            description: custom.description.clone(),
            records: custom
                .records
                .iter()
                .map(|r| PresetRecord {
                    record_type: r.record_type.clone(),
                    host: r.host.clone(),
                    value: r.value.clone(),
                    ttl: r.ttl,
                    priority: r.priority,
                })
                .collect(),
            variables: custom
                .variables
                .iter()
                .map(|v| PresetVariable {
                    name: v.name.clone(),
                    description: v.description.clone(),
                    default: v.default.clone(),
                    required: v.required,
                })
                .collect(),
        });
    }

    // Check builtin presets
    let filename = format!("{}.toml", name);
    if let Some(content) = BuiltinPresets::get(&filename) {
        if let Ok(content_str) = std::str::from_utf8(content.data.as_ref()) {
            if let Ok(preset) = toml::from_str::<Preset>(content_str) {
                return Ok(preset);
            }
        }
    }

    Err(CliError::PresetNotFound(name.to_string()))
}
