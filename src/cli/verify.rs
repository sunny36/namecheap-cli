use crate::api::NamecheapClient;
use crate::config::Config;
use crate::dns::{verify_record, verify_with_wait, DnsRecord, RecordType};
use crate::error::{CliError, Result};
use crate::output::{is_json, json, table};
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use super::GlobalOpts;

#[derive(Parser)]
pub struct VerifyCommand {
    /// Domain name
    pub domain: String,

    /// Record type (optional, verifies all if not specified)
    #[arg(long, short = 't')]
    pub record_type: Option<String>,

    /// Host/subdomain (optional)
    #[arg(long, short = 'H')]
    pub host: Option<String>,

    /// Wait for DNS propagation
    #[arg(long)]
    pub wait: bool,

    /// Timeout in seconds when waiting
    #[arg(long, default_value = "300")]
    pub timeout: u64,

    /// Polling interval in seconds when waiting
    #[arg(long, default_value = "10")]
    pub interval: u64,
}

pub async fn run(cmd: VerifyCommand, config: &Config, global: &GlobalOpts) -> Result<()> {
    let client = NamecheapClient::new(config)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    // Get records from Namecheap
    let mut records = client
        .get_hosts(&cmd.domain)
        .await
        .map_err(|e| CliError::Api(e.to_string()))?;

    // Filter by type if specified
    if let Some(ref rt) = cmd.record_type {
        records.retain(|r| r.record_type.to_string().eq_ignore_ascii_case(rt));
    }

    // Filter by host if specified
    if let Some(ref host) = cmd.host {
        records.retain(|r| r.host.eq_ignore_ascii_case(host));
    }

    if records.is_empty() {
        if is_json(global) {
            json::print_json(&serde_json::json!({
                "verified": false,
                "message": "No records found to verify",
            }));
        } else {
            println!("No records found to verify.");
        }
        return Ok(());
    }

    // Only verify record types that support verification
    let verifiable_records: Vec<DnsRecord> = records
        .into_iter()
        .filter(|r| is_verifiable(&r.record_type))
        .collect();

    if verifiable_records.is_empty() {
        if is_json(global) {
            json::print_json(&serde_json::json!({
                "verified": false,
                "message": "No verifiable records found (only A, AAAA, CNAME, MX, TXT, NS are supported)",
            }));
        } else {
            println!(
                "No verifiable records found (only A, AAAA, CNAME, MX, TXT, NS are supported)."
            );
        }
        return Ok(());
    }

    if cmd.wait {
        verify_with_waiting(
            &cmd.domain,
            &verifiable_records,
            cmd.timeout,
            cmd.interval,
            global,
        )
        .await
    } else {
        verify_once(&cmd.domain, &verifiable_records, global).await
    }
}

fn is_verifiable(record_type: &RecordType) -> bool {
    matches!(
        record_type,
        RecordType::A
            | RecordType::AAAA
            | RecordType::CNAME
            | RecordType::MX
            | RecordType::TXT
            | RecordType::NS
    )
}

async fn verify_once(domain: &str, records: &[DnsRecord], global: &GlobalOpts) -> Result<()> {
    let mut results = Vec::new();
    let mut all_success = true;

    for record in records {
        match verify_record(domain, record).await {
            Ok(result) => {
                if !result.success {
                    all_success = false;
                }
                results.push(result);
            }
            Err(e) => {
                all_success = false;
                results.push(crate::dns::VerificationResult {
                    record: record.clone(),
                    success: false,
                    actual_values: vec![],
                    message: format!("Verification error: {}", e),
                });
            }
        }
    }

    if is_json(global) {
        json::print_verification_results_json(&results);
    } else {
        table::print_verification_results(&results);
    }

    if all_success {
        Ok(())
    } else {
        Err(CliError::VerificationFailed(
            "Some records failed verification".to_string(),
        ))
    }
}

async fn verify_with_waiting(
    domain: &str,
    records: &[DnsRecord],
    timeout_secs: u64,
    interval_secs: u64,
    global: &GlobalOpts,
) -> Result<()> {
    let timeout = Duration::from_secs(timeout_secs);
    let interval = Duration::from_secs(interval_secs);

    let pb = if !global.quiet && !is_json(global) {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message("Waiting for DNS propagation...");
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let results = verify_with_wait(domain, records, timeout, interval).await;

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    let all_success = results.iter().all(|r| r.success);

    if is_json(global) {
        json::print_verification_results_json(&results);
    } else {
        table::print_verification_results(&results);
    }

    if all_success {
        if !global.quiet && !is_json(global) {
            println!(
                "\n{} All records verified successfully!",
                style("✓").green()
            );
        }
        Ok(())
    } else {
        Err(CliError::VerificationFailed(
            "Some records failed verification".to_string(),
        ))
    }
}
