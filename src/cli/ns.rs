use crate::api::NamecheapClient;
use crate::config::Config;
use crate::error::{CliError, Result};
use crate::output::{is_json, json, table};
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::Confirm;

use super::GlobalOpts;

#[derive(Parser)]
pub struct NsCommand {
    #[command(subcommand)]
    pub command: NsSubcommand,
}

#[derive(Subcommand)]
pub enum NsSubcommand {
    /// List nameservers for a domain
    List {
        /// Domain name
        domain: String,
    },

    /// Set custom nameservers
    Set {
        /// Domain name
        domain: String,

        /// Nameservers (provide multiple times or comma-separated)
        #[arg(required = true)]
        nameservers: Vec<String>,
    },

    /// Reset to Namecheap default nameservers
    Reset {
        /// Domain name
        domain: String,
    },
}

pub async fn run(cmd: NsCommand, config: &Config, global: &GlobalOpts) -> Result<()> {
    match cmd.command {
        NsSubcommand::List { domain } => list(&domain, config, global).await,
        NsSubcommand::Set {
            domain,
            nameservers,
        } => set(&domain, &nameservers, config, global).await,
        NsSubcommand::Reset { domain } => reset(&domain, config, global).await,
    }
}

async fn list(domain: &str, config: &Config, global: &GlobalOpts) -> Result<()> {
    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let (is_using_our_dns, nameservers) = client
        .get_nameservers(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    if is_json(global) {
        json::print_nameservers_json(is_using_our_dns, &nameservers);
    } else {
        table::print_nameservers(is_using_our_dns, &nameservers);
    }

    Ok(())
}

async fn set(
    domain: &str,
    nameservers: &[String],
    config: &Config,
    global: &GlobalOpts,
) -> Result<()> {
    // Parse nameservers - handle comma-separated values
    let ns_list: Vec<String> = nameservers
        .iter()
        .flat_map(|ns| ns.split(','))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if ns_list.len() < 2 {
        return Err(CliError::Validation(
            "At least 2 nameservers are required".to_string(),
        ));
    }

    if global.dry_run {
        if is_json(global) {
            json::print_json(&serde_json::json!({
                "dry_run": true,
                "action": "set_nameservers",
                "domain": domain,
                "nameservers": ns_list,
            }));
        } else {
            println!(
                "{} Would set nameservers for {}:",
                style("[dry-run]").yellow(),
                domain
            );
            for ns in &ns_list {
                println!("  - {}", ns);
            }
        }
        return Ok(());
    }

    if !global.yes && !is_json(global) {
        println!("This will change nameservers for {} to:", domain);
        for ns in &ns_list {
            println!("  - {}", ns);
        }
        println!();
        println!(
            "{}",
            style("Warning: DNS will stop working with Namecheap after this change.").yellow()
        );

        let confirm = Confirm::new()
            .with_prompt("Continue?")
            .default(false)
            .interact()
            .map_err(|e| CliError::Other(e.to_string()))?;

        if !confirm {
            return Ok(());
        }
    }

    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let ns_refs: Vec<&str> = ns_list.iter().map(|s| s.as_str()).collect();
    client
        .set_nameservers(domain, &ns_refs)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    if is_json(global) {
        json::print_success_json("Nameservers updated successfully");
    } else if !global.quiet {
        println!("{} Nameservers updated for {}", style("✓").green(), domain);
    }

    Ok(())
}

async fn reset(domain: &str, config: &Config, global: &GlobalOpts) -> Result<()> {
    if global.dry_run {
        if is_json(global) {
            json::print_json(&serde_json::json!({
                "dry_run": true,
                "action": "reset_nameservers",
                "domain": domain,
            }));
        } else {
            println!(
                "{} Would reset {} to Namecheap default nameservers",
                style("[dry-run]").yellow(),
                domain
            );
        }
        return Ok(());
    }

    if !global.yes && !is_json(global) {
        println!(
            "This will reset {} to use Namecheap default nameservers.",
            domain
        );

        let confirm = Confirm::new()
            .with_prompt("Continue?")
            .default(false)
            .interact()
            .map_err(|e| CliError::Other(e.to_string()))?;

        if !confirm {
            return Ok(());
        }
    }

    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    client
        .reset_nameservers(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    if is_json(global) {
        json::print_success_json("Nameservers reset to Namecheap defaults");
    } else if !global.quiet {
        println!(
            "{} Nameservers reset to Namecheap defaults for {}",
            style("✓").green(),
            domain
        );
    }

    Ok(())
}
