use crate::api::NamecheapClient;
use crate::config::{self, Config, Profile};
use crate::error::{CliError, Result};
use crate::output::{is_json, json, table};
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{Input, Password};

use super::GlobalOpts;

#[derive(Parser)]
pub struct AuthCommand {
    #[command(subcommand)]
    pub command: AuthSubcommand,
}

#[derive(Subcommand)]
pub enum AuthSubcommand {
    /// Configure API credentials
    Login {
        /// API user
        #[arg(long)]
        api_user: Option<String>,

        /// API key
        #[arg(long)]
        api_key: Option<String>,

        /// Username (if different from API user)
        #[arg(long)]
        username: Option<String>,

        /// Client IP (auto-detected if not specified)
        #[arg(long)]
        client_ip: Option<String>,

        /// Use sandbox API
        #[arg(long)]
        sandbox: bool,

        /// Profile name to save as
        #[arg(long, default_value = "default")]
        profile: String,
    },

    /// Show authentication status
    Status,

    /// Show current user info and account balance
    Whoami,
}

pub async fn run(cmd: AuthCommand, config: &Config, global: &GlobalOpts) -> Result<()> {
    match cmd.command {
        AuthSubcommand::Login {
            api_user,
            api_key,
            username,
            client_ip,
            sandbox,
            profile,
        } => {
            login(
                api_user, api_key, username, client_ip, sandbox, profile, global,
            )
            .await
        }
        AuthSubcommand::Status => status(config, global).await,
        AuthSubcommand::Whoami => whoami(config, global).await,
    }
}

async fn login(
    api_user: Option<String>,
    api_key: Option<String>,
    username: Option<String>,
    client_ip: Option<String>,
    sandbox: bool,
    profile_name: String,
    global: &GlobalOpts,
) -> Result<()> {
    let api_user = match api_user {
        Some(u) => u,
        None => Input::new()
            .with_prompt("API User")
            .interact_text()
            .map_err(|e| CliError::Other(e.to_string()))?,
    };

    let api_key = match api_key {
        Some(k) => k,
        None => Password::new()
            .with_prompt("API Key")
            .interact()
            .map_err(|e| CliError::Other(e.to_string()))?,
    };

    let profile = Profile {
        api_user,
        api_key,
        username,
        client_ip,
        sandbox,
    };

    // Verify credentials work
    let temp_config = Config {
        profile: profile.clone(),
        profile_name: profile_name.clone(),
        presets: std::collections::HashMap::new(),
    };

    if !global.quiet {
        println!("Verifying credentials...");
    }

    let client = NamecheapClient::new(&temp_config)
        .await
        .map_err(|e| CliError::Auth(e.to_string()))?;

    client
        .get_balances()
        .await
        .map_err(|e| CliError::Auth(e.to_string()))?;

    // Save to config file
    let mut config_file = config::load_config_file()?;
    config_file.profiles.insert(profile_name.clone(), profile);

    if config_file.default_profile.is_none() {
        config_file.default_profile = Some(profile_name.clone());
    }

    config::save_config(&config_file)?;

    if is_json(global) {
        json::print_success_json(&format!("Profile '{}' saved successfully", profile_name));
    } else if !global.quiet {
        println!(
            "{} Profile '{}' saved successfully",
            style("✓").green(),
            profile_name
        );
    }

    Ok(())
}

async fn status(config: &Config, global: &GlobalOpts) -> Result<()> {
    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Auth(e.to_string()))?;

    match client.get_balances().await {
        Ok(_) => {
            if is_json(global) {
                json::print_json(&serde_json::json!({
                    "authenticated": true,
                    "profile": config.profile_name,
                    "api_user": config.profile.api_user,
                    "sandbox": config.profile.sandbox,
                }));
            } else {
                println!("{} Authenticated", style("✓").green());
                println!("  Profile: {}", config.profile_name);
                println!("  API User: {}", config.profile.api_user);
                if config.profile.sandbox {
                    println!("  Mode: {}", style("Sandbox").yellow());
                }
            }
            Ok(())
        }
        Err(e) => {
            if is_json(global) {
                json::print_json(&serde_json::json!({
                    "authenticated": false,
                    "error": e.to_string(),
                }));
            } else {
                println!("{} Not authenticated: {}", style("✗").red(), e);
            }
            Err(CliError::Auth(e.to_string()))
        }
    }
}

async fn whoami(config: &Config, global: &GlobalOpts) -> Result<()> {
    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Auth(e.to_string()))?;

    let balances = client
        .get_balances()
        .await
        .map_err(|e| CliError::Auth(e.to_string()))?;

    if is_json(global) {
        json::print_json(&serde_json::json!({
            "profile": config.profile_name,
            "api_user": config.profile.api_user,
            "username": config.username(),
            "sandbox": config.profile.sandbox,
            "balances": balances,
        }));
    } else {
        println!("{}", style("User Information").bold());
        println!();
        println!("  {} {}", style("Profile:").dim(), config.profile_name);
        println!("  {} {}", style("API User:").dim(), config.profile.api_user);
        println!("  {} {}", style("Username:").dim(), config.username());
        if config.profile.sandbox {
            println!("  {} {}", style("Mode:").dim(), style("Sandbox").yellow());
        }
        println!();
        table::print_balances(&balances);
    }

    Ok(())
}
