use crate::api::{DomainCheckItem, DomainGetInfoResult, DomainItem, UserGetBalancesResult};
use crate::dns::{DiffAction, DnsRecord, DnsRecordDiff, VerificationResult};
use crate::preset::Preset;
use console::{style, Style};
use tabled::settings::Style as TabledStyle;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct DomainRow {
    #[tabled(rename = "Domain")]
    name: String,
    #[tabled(rename = "Expires")]
    expires: String,
    #[tabled(rename = "Auto-Renew")]
    auto_renew: String,
    #[tabled(rename = "DNS")]
    dns: String,
}

pub fn print_domains(domains: &[DomainItem]) {
    let rows: Vec<DomainRow> = domains
        .iter()
        .map(|d| DomainRow {
            name: d.name.clone(),
            expires: d.expires.clone().unwrap_or_default(),
            auto_renew: if d.auto_renew.as_deref() == Some("true") {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
            dns: if d.is_our_dns.as_deref() == Some("true") {
                "Namecheap".to_string()
            } else {
                "Custom".to_string()
            },
        })
        .collect();

    let table = Table::new(rows).with(TabledStyle::rounded()).to_string();

    println!("{}", table);
}

pub fn print_domain_info(info: &DomainGetInfoResult) {
    println!("{}", style("Domain Information").bold());
    println!();

    if let Some(name) = &info.domain_name {
        println!("  {} {}", style("Domain:").dim(), name);
    }
    if let Some(status) = &info.status {
        println!("  {} {}", style("Status:").dim(), status);
    }

    if let Some(details) = &info.domain_details {
        if let Some(created) = &details.created_date {
            println!("  {} {}", style("Created:").dim(), created);
        }
        if let Some(expires) = &details.expired_date {
            println!("  {} {}", style("Expires:").dim(), expires);
        }
    }

    if let Some(dns) = &info.dns_details {
        println!();
        println!("{}", style("DNS Configuration").bold());
        println!(
            "  {} {}",
            style("Provider:").dim(),
            dns.provider_type.as_deref().unwrap_or("Unknown")
        );
        println!(
            "  {} {}",
            style("Using Namecheap DNS:").dim(),
            if dns.is_using_our_dns.as_deref() == Some("true") {
                "Yes"
            } else {
                "No"
            }
        );
        if !dns.nameservers.is_empty() {
            println!("  {}", style("Nameservers:").dim());
            for ns in &dns.nameservers {
                println!("    - {}", ns);
            }
        }
    }

    if let Some(whois) = &info.whois_guard {
        println!();
        println!("{}", style("WhoisGuard").bold());
        println!(
            "  {} {}",
            style("Enabled:").dim(),
            if whois.enabled.as_deref() == Some("True") {
                "Yes"
            } else {
                "No"
            }
        );
        if let Some(expires) = &whois.expired_date {
            println!("  {} {}", style("Expires:").dim(), expires);
        }
    }
}

#[derive(Tabled)]
struct DnsRecordRow {
    #[tabled(rename = "Type")]
    record_type: String,
    #[tabled(rename = "Host")]
    host: String,
    #[tabled(rename = "Value")]
    value: String,
    #[tabled(rename = "TTL")]
    ttl: String,
    #[tabled(rename = "Priority")]
    priority: String,
}

pub fn print_dns_records(records: &[DnsRecord]) {
    let rows: Vec<DnsRecordRow> = records
        .iter()
        .map(|r| DnsRecordRow {
            record_type: r.record_type.to_string(),
            host: r.host.clone(),
            value: truncate_value(&r.value, 50),
            ttl: r.ttl.to_string(),
            priority: r.priority.map(|p| p.to_string()).unwrap_or_default(),
        })
        .collect();

    let table = Table::new(rows).with(TabledStyle::rounded()).to_string();

    println!("{}", table);
}

fn truncate_value(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

pub fn print_dns_diff(diffs: &[DnsRecordDiff]) {
    if diffs.is_empty() {
        println!("{}", style("No changes detected").dim());
        return;
    }

    println!("{}", style("Changes to apply:").bold());
    println!();

    for diff in diffs {
        let (prefix, color) = match diff.action {
            DiffAction::Add => ("+", Style::new().green()),
            DiffAction::Remove => ("-", Style::new().red()),
            DiffAction::Modify => ("~", Style::new().yellow()),
        };

        println!(
            "  {} {} {} {} (TTL: {}{})",
            color.apply_to(prefix),
            diff.record.record_type,
            diff.record.host,
            diff.record.value,
            diff.record.ttl,
            diff.record
                .priority
                .map(|p| format!(", Priority: {}", p))
                .unwrap_or_default()
        );

        if let Some(old) = &diff.old_record {
            println!(
                "    {} was: {} (TTL: {}{})",
                style("↳").dim(),
                old.value,
                old.ttl,
                old.priority
                    .map(|p| format!(", Priority: {}", p))
                    .unwrap_or_default()
            );
        }
    }
}

pub fn print_verification_results(results: &[VerificationResult]) {
    for result in results {
        let status = if result.success {
            style("✓").green()
        } else {
            style("✗").red()
        };

        println!(
            "{} {} {} {} - {}",
            status,
            result.record.record_type,
            result.record.host,
            result.record.value,
            result.message
        );

        if !result.actual_values.is_empty() && !result.success {
            println!("    Actual: {:?}", result.actual_values);
        }
    }
}

pub fn print_nameservers(is_using_our_dns: bool, nameservers: &[String]) {
    println!(
        "{} {}",
        style("Using Namecheap DNS:").dim(),
        if is_using_our_dns { "Yes" } else { "No" }
    );
    println!();
    println!("{}", style("Nameservers:").bold());
    for ns in nameservers {
        println!("  - {}", ns);
    }
}

#[derive(Tabled)]
struct DomainCheckRow {
    #[tabled(rename = "Domain")]
    domain: String,
    #[tabled(rename = "Available")]
    available: String,
    #[tabled(rename = "Premium")]
    premium: String,
    #[tabled(rename = "Price")]
    price: String,
}

pub fn print_domain_check(results: &[DomainCheckItem]) {
    let rows: Vec<DomainCheckRow> = results
        .iter()
        .map(|r| DomainCheckRow {
            domain: r.domain.clone(),
            available: if r.available == "true" {
                style("Yes").green().to_string()
            } else {
                style("No").red().to_string()
            },
            premium: if r.is_premium_name.as_deref() == Some("true") {
                "Yes".to_string()
            } else {
                "No".to_string()
            },
            price: r.premium_registration_price.clone().unwrap_or_default(),
        })
        .collect();

    let table = Table::new(rows).with(TabledStyle::rounded()).to_string();

    println!("{}", table);
}

pub fn print_balances(balances: &UserGetBalancesResult) {
    println!("{}", style("Account Balances").bold());
    println!();
    println!(
        "  {} {} {}",
        style("Available Balance:").dim(),
        balances.available_balance.as_deref().unwrap_or("N/A"),
        balances.currency.as_deref().unwrap_or("")
    );
    println!(
        "  {} {} {}",
        style("Account Balance:").dim(),
        balances.account_balance.as_deref().unwrap_or("N/A"),
        balances.currency.as_deref().unwrap_or("")
    );
    if let Some(earned) = &balances.earned_amount {
        println!(
            "  {} {} {}",
            style("Earned Amount:").dim(),
            earned,
            balances.currency.as_deref().unwrap_or("")
        );
    }
}

#[derive(Tabled)]
struct PresetRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Records")]
    records: String,
}

pub fn print_presets(presets: &[Preset]) {
    let rows: Vec<PresetRow> = presets
        .iter()
        .map(|p| PresetRow {
            name: p.name.clone(),
            description: p.description.clone().unwrap_or_default(),
            records: p.records.len().to_string(),
        })
        .collect();

    let table = Table::new(rows).with(TabledStyle::rounded()).to_string();

    println!("{}", table);
}

pub fn print_preset_detail(preset: &Preset) {
    println!("{}", style(&preset.name).bold());
    if let Some(desc) = &preset.description {
        println!("{}", style(desc).dim());
    }
    println!();

    if !preset.variables.is_empty() {
        println!("{}", style("Variables:").bold());
        for var in &preset.variables {
            print!("  {} {}", style(&var.name).cyan(), style(":").dim());
            if let Some(desc) = &var.description {
                print!(" {}", desc);
            }
            if var.required {
                print!(" {}", style("(required)").red());
            }
            if let Some(default) = &var.default {
                print!(" {}", style(format!("[default: {}]", default)).dim());
            }
            println!();
        }
        println!();
    }

    println!("{}", style("Records:").bold());
    for record in &preset.records {
        print!(
            "  {} {} {}",
            style(&record.record_type).green(),
            record.host,
            record.value
        );
        if let Some(ttl) = record.ttl {
            print!(" {}", style(format!("(TTL: {})", ttl)).dim());
        }
        if let Some(priority) = record.priority {
            print!(" {}", style(format!("(Priority: {})", priority)).dim());
        }
        println!();
    }
}
