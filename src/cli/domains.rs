use crate::api::NamecheapClient;
use crate::config::Config;
use crate::error::{CliError, Result};
use crate::output::{is_json, json, table};
use clap::{Parser, Subcommand};

use super::GlobalOpts;

#[derive(Parser)]
pub struct DomainsCommand {
    #[command(subcommand)]
    pub command: DomainsSubcommand,
}

#[derive(Subcommand)]
pub enum DomainsSubcommand {
    /// List all domains in your account
    List,

    /// Get detailed information about a domain
    Info {
        /// Domain name
        domain: String,
    },

    /// Check domain availability
    Check {
        /// Domain names to check
        #[arg(required = true)]
        domains: Vec<String>,
    },
}

pub async fn run(cmd: DomainsCommand, config: &Config, global: &GlobalOpts) -> Result<()> {
    match cmd.command {
        DomainsSubcommand::List => list(config, global).await,
        DomainsSubcommand::Info { domain } => info(&domain, config, global).await,
        DomainsSubcommand::Check { domains } => {
            let domains: Vec<&str> = domains.iter().map(|s| s.as_str()).collect();
            check(&domains, config, global).await
        }
    }
}

async fn list(config: &Config, global: &GlobalOpts) -> Result<()> {
    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let domains = client
        .list_all_domains()
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    if domains.is_empty() {
        if is_json(global) {
            json::print_domains_json(&domains);
        } else if !global.quiet {
            println!("No domains found in your account.");
        }
        return Ok(());
    }

    if is_json(global) {
        json::print_domains_json(&domains);
    } else {
        table::print_domains(&domains);
    }

    Ok(())
}

async fn info(domain: &str, config: &Config, global: &GlobalOpts) -> Result<()> {
    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let info = client.get_domain_info(domain).await.map_err(|e| match e {
        crate::api::ApiError::DomainNotFound(_) => CliError::DomainNotFound(domain.to_string()),
        _ => CliError::Api(e.to_string()),
    })?;

    if is_json(global) {
        json::print_domain_info_json(&info);
    } else {
        table::print_domain_info(&info);
    }

    Ok(())
}

async fn check(domains: &[&str], config: &Config, global: &GlobalOpts) -> Result<()> {
    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let results = client
        .check_domains(domains)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    if is_json(global) {
        json::print_domain_check_json(&results);
    } else {
        table::print_domain_check(&results);
    }

    Ok(())
}
