use crate::api::NamecheapClient;
use crate::config::Config;
use crate::dns::{apply_diff, calculate_diff, parse_record_from_args, DnsRecord};
use crate::error::{CliError, Result};
use crate::output::{is_json, json, table};
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::Confirm;
use std::io::{self, BufRead};

use super::GlobalOpts;

#[derive(Parser)]
pub struct DnsCommand {
    #[command(subcommand)]
    pub command: DnsSubcommand,
}

#[derive(Subcommand)]
pub enum DnsSubcommand {
    /// List DNS records for a domain
    List {
        /// Domain name
        domain: String,

        /// Filter by record type
        #[arg(long, short = 't')]
        record_type: Option<String>,
    },

    /// Add a DNS record
    Add {
        /// Domain name
        domain: String,

        /// Record type (A, AAAA, CNAME, MX, TXT, NS, SRV, CAA, ALIAS, URL, URL301, FRAME)
        record_type: String,

        /// Host/subdomain (use @ for root)
        host: String,

        /// Record value
        value: String,

        /// TTL in seconds
        #[arg(long, default_value = "1800")]
        ttl: u32,

        /// Priority (required for MX records)
        #[arg(long)]
        priority: Option<u16>,

        /// Allow replacing the zone with zero records (Namecheap setHosts is a full-zone replace)
        #[arg(long)]
        allow_empty: bool,
    },

    /// Set/replace a DNS record
    Set {
        /// Domain name
        domain: String,

        /// Record type
        record_type: String,

        /// Host/subdomain (use @ for root)
        host: String,

        /// Record value
        value: String,

        /// TTL in seconds
        #[arg(long, default_value = "1800")]
        ttl: u32,

        /// Priority (required for MX records)
        #[arg(long)]
        priority: Option<u16>,

        /// Allow replacing the zone with zero records (Namecheap setHosts is a full-zone replace)
        #[arg(long)]
        allow_empty: bool,
    },

    /// Remove a DNS record
    Rm {
        /// Domain name
        domain: String,

        /// Record type
        record_type: String,

        /// Host/subdomain (use @ for root)
        host: String,

        /// Record value (optional, removes all matching type+host if not specified)
        value: Option<String>,

        /// Allow replacing the zone with zero records (Namecheap setHosts is a full-zone replace)
        #[arg(long)]
        allow_empty: bool,
    },

    /// Export DNS records
    Export {
        /// Domain name
        domain: String,

        /// Output format (json or zone)
        #[arg(long, default_value = "json")]
        format: String,
    },

    /// Sync DNS records from a file or stdin
    Sync {
        /// Domain name
        domain: String,

        /// File to sync from (use - for stdin)
        #[arg(default_value = "-")]
        file: String,

        /// Delete records not in the input
        #[arg(long)]
        delete: bool,

        /// Allow replacing the zone with zero records (Namecheap setHosts is a full-zone replace)
        #[arg(long)]
        allow_empty: bool,
    },

    /// Show diff between current and desired DNS records
    Diff {
        /// Domain name
        domain: String,

        /// File with desired records (use - for stdin)
        #[arg(default_value = "-")]
        file: String,
    },
}

pub async fn run(cmd: DnsCommand, config: &Config, global: &GlobalOpts) -> Result<()> {
    match cmd.command {
        DnsSubcommand::List {
            domain,
            record_type,
        } => list(&domain, record_type.as_deref(), config, global).await,
        DnsSubcommand::Add {
            domain,
            record_type,
            host,
            value,
            ttl,
            priority,
            allow_empty,
        } => {
            add(
                &domain,
                &record_type,
                &host,
                &value,
                ttl,
                priority,
                allow_empty,
                config,
                global,
            )
            .await
        }
        DnsSubcommand::Set {
            domain,
            record_type,
            host,
            value,
            ttl,
            priority,
            allow_empty,
        } => {
            set(
                &domain,
                &record_type,
                &host,
                &value,
                ttl,
                priority,
                allow_empty,
                config,
                global,
            )
            .await
        }
        DnsSubcommand::Rm {
            domain,
            record_type,
            host,
            value,
            allow_empty,
        } => {
            rm(
                &domain,
                &record_type,
                &host,
                value.as_deref(),
                allow_empty,
                config,
                global,
            )
            .await
        }
        DnsSubcommand::Export { domain, format } => export(&domain, &format, config, global).await,
        DnsSubcommand::Sync {
            domain,
            file,
            delete,
            allow_empty,
        } => sync(&domain, &file, delete, allow_empty, config, global).await,
        DnsSubcommand::Diff { domain, file } => diff(&domain, &file, config, global).await,
    }
}

async fn list(
    domain: &str,
    record_type: Option<&str>,
    config: &Config,
    global: &GlobalOpts,
) -> Result<()> {
    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let mut records = client
        .get_hosts(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    if let Some(rt) = record_type {
        records.retain(|r| r.record_type.to_string().eq_ignore_ascii_case(rt));
    }

    if is_json(global) {
        json::print_dns_records_json(&records);
    } else if records.is_empty() {
        println!("No DNS records found.");
    } else {
        table::print_dns_records(&records);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn add(
    domain: &str,
    record_type: &str,
    host: &str,
    value: &str,
    ttl: u32,
    priority: Option<u16>,
    allow_empty: bool,
    config: &Config,
    global: &GlobalOpts,
) -> Result<()> {
    let record = parse_record_from_args(record_type, host, value, Some(ttl), priority)
        .map_err(CliError::Validation)?;

    if global.dry_run {
        if is_json(global) {
            json::print_json(&serde_json::json!({
                "dry_run": true,
                "action": "add",
                "record": record,
            }));
        } else {
            println!("{} Would add: {}", style("[dry-run]").yellow(), record);
        }
        return Ok(());
    }

    let client = NamecheapClient::new(config).await?;

    let outcome = client
        .add_record(domain, record.clone(), allow_empty)
        .await?;

    match outcome {
        crate::api::AddRecordResult::Unchanged => {
            if is_json(global) {
                json::print_success_json("Record already exists (no change)");
            } else if !global.quiet {
                println!(
                    "{} Record already exists (no change): {}",
                    style("✓").green(),
                    record
                );
            }
        }
        crate::api::AddRecordResult::Added => {
            if is_json(global) {
                json::print_success_json("Record added successfully");
            } else if !global.quiet {
                println!("{} Record added: {}", style("✓").green(), record);
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn set(
    domain: &str,
    record_type: &str,
    host: &str,
    value: &str,
    ttl: u32,
    priority: Option<u16>,
    allow_empty: bool,
    config: &Config,
    global: &GlobalOpts,
) -> Result<()> {
    let record = parse_record_from_args(record_type, host, value, Some(ttl), priority)
        .map_err(CliError::Validation)?;

    if global.dry_run {
        if is_json(global) {
            json::print_json(&serde_json::json!({
                "dry_run": true,
                "action": "set",
                "record": record,
            }));
        } else {
            println!("{} Would set: {}", style("[dry-run]").yellow(), record);
        }
        return Ok(());
    }

    let client = NamecheapClient::new(config).await?;
    client
        .set_record(domain, record.clone(), allow_empty)
        .await?;

    if is_json(global) {
        json::print_success_json("Record set successfully");
    } else if !global.quiet {
        println!("{} Record set: {}", style("✓").green(), record);
    }

    Ok(())
}

async fn rm(
    domain: &str,
    record_type: &str,
    host: &str,
    value: Option<&str>,
    allow_empty: bool,
    config: &Config,
    global: &GlobalOpts,
) -> Result<()> {
    if global.dry_run {
        if is_json(global) {
            json::print_json(&serde_json::json!({
                "dry_run": true,
                "action": "remove",
                "record_type": record_type,
                "host": host,
                "value": value,
            }));
        } else {
            println!(
                "{} Would remove: {} {} {}",
                style("[dry-run]").yellow(),
                record_type,
                host,
                value.unwrap_or("*")
            );
        }
        return Ok(());
    }

    let client = NamecheapClient::new(config).await?;

    let removed = client
        .remove_record(domain, record_type, host, value, allow_empty)
        .await?;

    if !removed {
        return Err(CliError::RecordNotFound(format!(
            "{} {} {}",
            record_type,
            host,
            value.unwrap_or("")
        )));
    }

    if is_json(global) {
        json::print_success_json("Record removed successfully");
    } else if !global.quiet {
        println!("{} Record removed", style("✓").green());
    }

    Ok(())
}

async fn export(domain: &str, format: &str, config: &Config, _global: &GlobalOpts) -> Result<()> {
    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let records = client
        .get_hosts(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    match format {
        "json" => {
            json::print_dns_records_json(&records);
        }
        "zone" => {
            for record in &records {
                let host = if record.host == "@" {
                    domain.to_string()
                } else {
                    format!("{}.{}", record.host, domain)
                };
                println!(
                    "{}\t{}\tIN\t{}\t{}",
                    host, record.ttl, record.record_type, record.value
                );
            }
        }
        _ => {
            return Err(CliError::Validation(format!(
                "Unknown format: {}. Use 'json' or 'zone'.",
                format
            )));
        }
    }

    Ok(())
}

async fn sync(
    domain: &str,
    file: &str,
    delete: bool,
    allow_empty: bool,
    config: &Config,
    global: &GlobalOpts,
) -> Result<()> {
    let desired = read_records_from_file(file)?;

    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let current = client
        .get_hosts(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let mut diffs = calculate_diff(&current, &desired);

    // If not deleting, filter out remove actions
    if !delete {
        diffs.retain(|d| !matches!(d.action, crate::dns::DiffAction::Remove));
    }

    if diffs.is_empty() {
        if is_json(global) {
            json::print_success_json("No changes needed");
        } else if !global.quiet {
            println!("No changes needed.");
        }
        return Ok(());
    }

    if is_json(global) {
        json::print_dns_diff_json(&diffs);
    } else {
        table::print_dns_diff(&diffs);
    }

    if global.dry_run {
        return Ok(());
    }

    if !global.yes {
        let confirm = Confirm::new()
            .with_prompt("Apply these changes?")
            .default(false)
            .interact()
            .map_err(|e| CliError::Other(e.to_string()))?;

        if !confirm {
            return Ok(());
        }
    }

    let new_records = apply_diff(&current, &diffs);
    client.set_hosts(domain, &new_records, allow_empty).await?;

    if is_json(global) {
        json::print_success_json("Changes applied successfully");
    } else if !global.quiet {
        println!("{} Changes applied successfully", style("✓").green());
    }

    Ok(())
}

async fn diff(domain: &str, file: &str, config: &Config, global: &GlobalOpts) -> Result<()> {
    let desired = read_records_from_file(file)?;

    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let current = client
        .get_hosts(domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    let diffs = calculate_diff(&current, &desired);

    if is_json(global) {
        json::print_dns_diff_json(&diffs);
    } else {
        table::print_dns_diff(&diffs);
    }

    Ok(())
}

fn read_records_from_file(file: &str) -> Result<Vec<DnsRecord>> {
    let content = if file == "-" {
        let stdin = io::stdin();
        let mut content = String::new();
        for line in stdin.lock().lines() {
            content.push_str(&line.map_err(CliError::Io)?);
            content.push('\n');
        }
        content
    } else {
        std::fs::read_to_string(file)?
    };

    // Try parsing as JSON first
    if let Ok(wrapper) = serde_json::from_str::<RecordsWrapper>(&content) {
        return Ok(wrapper.records);
    }

    // Try parsing as array of records
    if let Ok(records) = serde_json::from_str::<Vec<DnsRecord>>(&content) {
        return Ok(records);
    }

    Err(CliError::Validation(
        "Could not parse records file. Expected JSON format.".to_string(),
    ))
}

#[derive(serde::Deserialize)]
struct RecordsWrapper {
    records: Vec<DnsRecord>,
}
