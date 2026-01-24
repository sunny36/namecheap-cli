use crate::api::{DomainCheckItem, DomainGetInfoResult, DomainItem, UserGetBalancesResult};
use crate::dns::{DnsRecord, DnsRecordDiff, VerificationResult};
use crate::preset::Preset;
use serde::Serialize;

pub fn print_json<T: Serialize>(data: &T) {
    match serde_json::to_string_pretty(data) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing to JSON: {}", e),
    }
}

pub fn print_domains_json(domains: &[DomainItem]) {
    #[derive(Serialize)]
    struct Output<'a> {
        domains: &'a [DomainItem],
    }
    print_json(&Output { domains });
}

pub fn print_domain_info_json(info: &DomainGetInfoResult) {
    print_json(info);
}

pub fn print_dns_records_json(records: &[DnsRecord]) {
    #[derive(Serialize)]
    struct Output<'a> {
        records: &'a [DnsRecord],
    }
    print_json(&Output { records });
}

pub fn print_dns_diff_json(diffs: &[DnsRecordDiff]) {
    #[derive(Serialize)]
    struct Output<'a> {
        changes: &'a [DnsRecordDiff],
    }
    print_json(&Output { changes: diffs });
}

pub fn print_verification_results_json(results: &[VerificationResult]) {
    #[derive(Serialize)]
    struct Output {
        results: Vec<VerificationOutput>,
    }

    #[derive(Serialize)]
    struct VerificationOutput {
        record_type: String,
        host: String,
        value: String,
        success: bool,
        actual_values: Vec<String>,
        message: String,
    }

    let results: Vec<VerificationOutput> = results
        .iter()
        .map(|r| VerificationOutput {
            record_type: r.record.record_type.to_string(),
            host: r.record.host.clone(),
            value: r.record.value.clone(),
            success: r.success,
            actual_values: r.actual_values.clone(),
            message: r.message.clone(),
        })
        .collect();

    print_json(&Output { results });
}

pub fn print_nameservers_json(is_using_our_dns: bool, nameservers: &[String]) {
    #[derive(Serialize)]
    struct Output<'a> {
        is_using_namecheap_dns: bool,
        nameservers: &'a [String],
    }
    print_json(&Output {
        is_using_namecheap_dns: is_using_our_dns,
        nameservers,
    });
}

pub fn print_domain_check_json(results: &[DomainCheckItem]) {
    #[derive(Serialize)]
    struct Output<'a> {
        results: &'a [DomainCheckItem],
    }
    print_json(&Output { results });
}

#[allow(dead_code)]
pub fn print_balances_json(balances: &UserGetBalancesResult) {
    print_json(balances);
}

pub fn print_presets_json(presets: &[Preset]) {
    #[derive(Serialize)]
    struct Output<'a> {
        presets: &'a [Preset],
    }
    print_json(&Output { presets });
}

pub fn print_preset_json(preset: &Preset) {
    print_json(preset);
}

pub fn print_success_json(message: &str) {
    #[derive(Serialize)]
    struct Output<'a> {
        success: bool,
        message: &'a str,
    }
    print_json(&Output {
        success: true,
        message,
    });
}
