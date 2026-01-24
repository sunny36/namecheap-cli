mod types;

pub use types::*;

use crate::error::{CliError, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

pub fn config_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "namecheap-cli", "namecheap-cli")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

pub fn default_config_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("config.toml"))
}

pub fn load_config(config_path: Option<&str>, profile_name: Option<&str>) -> Result<Config> {
    let path = config_path.map(PathBuf::from).or_else(default_config_path);

    let config_file = match path {
        Some(p) if p.exists() => {
            let contents = std::fs::read_to_string(&p)?;
            let mut file: ConfigFile = toml::from_str(&contents)?;
            expand_env_vars(&mut file);
            file
        }
        _ => ConfigFile::default(),
    };

    let profile_name = profile_name
        .map(String::from)
        .or(config_file.default_profile.clone())
        .unwrap_or_else(|| "default".to_string());

    let profile = config_file
        .profiles
        .get(&profile_name)
        .cloned()
        .or_else(profile_from_env)
        .ok_or_else(|| {
            CliError::Config(format!(
                "Profile '{}' not found. Run 'namecheap auth login' to configure.",
                profile_name
            ))
        })?;

    Ok(Config {
        profile,
        profile_name,
        presets: config_file.presets,
    })
}

fn profile_from_env() -> Option<Profile> {
    let api_user = std::env::var("NAMECHEAP_API_USER").ok()?;
    let api_key = std::env::var("NAMECHEAP_API_KEY").ok()?;
    let username = std::env::var("NAMECHEAP_USERNAME").ok();
    let client_ip = std::env::var("NAMECHEAP_CLIENT_IP").ok();
    let sandbox = std::env::var("NAMECHEAP_SANDBOX")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false);

    Some(Profile {
        api_user,
        api_key,
        username,
        client_ip,
        sandbox,
    })
}

fn expand_env_vars(config: &mut ConfigFile) {
    for profile in config.profiles.values_mut() {
        profile.api_user = expand_string(&profile.api_user);
        profile.api_key = expand_string(&profile.api_key);
        if let Some(ref mut username) = profile.username {
            *username = expand_string(username);
        }
        if let Some(ref mut client_ip) = profile.client_ip {
            *client_ip = expand_string(client_ip);
        }
    }
}

fn expand_string(s: &str) -> String {
    shellexpand::env(s)
        .map(|expanded| expanded.into_owned())
        .unwrap_or_else(|_| s.to_string())
}

pub fn save_config(config_file: &ConfigFile) -> Result<()> {
    let path = default_config_path()
        .ok_or_else(|| CliError::Config("Could not determine config directory".to_string()))?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let contents = toml::to_string_pretty(config_file)
        .map_err(|e| CliError::Config(format!("Failed to serialize config: {}", e)))?;

    std::fs::write(&path, contents)?;
    Ok(())
}

pub fn load_config_file() -> Result<ConfigFile> {
    let path = default_config_path();

    match path {
        Some(p) if p.exists() => {
            let contents = std::fs::read_to_string(&p)?;
            let file: ConfigFile = toml::from_str(&contents)?;
            Ok(file)
        }
        _ => Ok(ConfigFile::default()),
    }
}
