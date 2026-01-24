use crate::api::NamecheapClient;
use crate::config::Config;
use crate::dns::{DnsRecord, RecordType};
use crate::error::{CliError, Result};
use crate::output::{is_json, json, table};
use clap::{Parser, Subcommand};
use console::style;

use super::GlobalOpts;

#[derive(Parser)]
pub struct RedirectCommand {
    #[command(subcommand)]
    pub command: RedirectSubcommand,
}

#[derive(Subcommand)]
pub enum RedirectSubcommand {
    /// List URL redirects for a domain
    List {
        /// Domain name
        domain: String,
    },

    /// Add a URL redirect
    Add {
        /// Domain name
        domain: String,

        /// Host/subdomain (use @ for root)
        host: String,

        /// Destination URL
        url: String,

        /// Use 301 permanent redirect
        #[arg(long)]
        permanent: bool,

        /// Use frame/masking redirect
        #[arg(long)]
        frame: bool,
    },

    /// Remove a URL redirect
    Rm {
        /// Domain name
        domain: String,

        /// Host/subdomain (use @ for root)
        host: String,
    },
}

pub async fn run(cmd: RedirectCommand, config: &Config, global: &GlobalOpts) -> Result<()> {
    match cmd.command {
        RedirectSubcommand::List { domain } => list(&domain, config, global).await,
        RedirectSubcommand::Add {
            domain,
            host,
            url,
            permanent,
            frame,
        } => add(&domain, &host, &url, permanent, frame, config, global).await,
        RedirectSubcommand::Rm { domain, host } => rm(&domain, &host, config, global).await,
    }
}

async fn list(domain: &str, config: &Config, global: &GlobalOpts) -> Result<()> {
    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let records = client
        .get_hosts(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let redirects: Vec<DnsRecord> = records
        .into_iter()
        .filter(|r| {
            matches!(
                r.record_type,
                RecordType::URL | RecordType::URL301 | RecordType::FRAME
            )
        })
        .collect();

    if is_json(global) {
        json::print_dns_records_json(&redirects);
    } else if redirects.is_empty() {
        println!("No URL redirects found.");
    } else {
        table::print_dns_records(&redirects);
    }

    Ok(())
}

async fn add(
    domain: &str,
    host: &str,
    url: &str,
    permanent: bool,
    frame: bool,
    config: &Config,
    global: &GlobalOpts,
) -> Result<()> {
    let record_type = if frame {
        RecordType::FRAME
    } else if permanent {
        RecordType::URL301
    } else {
        RecordType::URL
    };

    let record = DnsRecord::new(record_type, host, url, 1800, None);

    if global.dry_run {
        if is_json(global) {
            json::print_json(&serde_json::json!({
                "dry_run": true,
                "action": "add_redirect",
                "record": record,
            }));
        } else {
            println!(
                "{} Would add redirect: {} -> {}",
                style("[dry-run]").yellow(),
                host,
                url
            );
        }
        return Ok(());
    }

    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    // Get existing records and remove any existing redirect for this host
    let mut records = client
        .get_hosts(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    records.retain(|r| {
        !(r.host.eq_ignore_ascii_case(host)
            && matches!(
                r.record_type,
                RecordType::URL | RecordType::URL301 | RecordType::FRAME
            ))
    });

    records.push(record);

    client
        .set_hosts(domain, &records)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    if is_json(global) {
        json::print_success_json("Redirect added successfully");
    } else if !global.quiet {
        println!("{} Redirect added: {} -> {}", style("✓").green(), host, url);
    }

    Ok(())
}

async fn rm(domain: &str, host: &str, config: &Config, global: &GlobalOpts) -> Result<()> {
    if global.dry_run {
        if is_json(global) {
            json::print_json(&serde_json::json!({
                "dry_run": true,
                "action": "remove_redirect",
                "host": host,
            }));
        } else {
            println!(
                "{} Would remove redirect for: {}",
                style("[dry-run]").yellow(),
                host
            );
        }
        return Ok(());
    }

    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let mut records = client
        .get_hosts(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let original_len = records.len();

    records.retain(|r| {
        !(r.host.eq_ignore_ascii_case(host)
            && matches!(
                r.record_type,
                RecordType::URL | RecordType::URL301 | RecordType::FRAME
            ))
    });

    if records.len() == original_len {
        return Err(CliError::RecordNotFound(format!(
            "No redirect found for host: {}",
            host
        )));
    }

    client
        .set_hosts(domain, &records)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    if is_json(global) {
        json::print_success_json("Redirect removed successfully");
    } else if !global.quiet {
        println!("{} Redirect removed for: {}", style("✓").green(), host);
    }

    Ok(())
}
