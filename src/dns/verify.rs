use super::record::{DnsRecord, RecordType};
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::Resolver;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

type TokioResolver = Resolver<TokioConnectionProvider>;

#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub record: DnsRecord,
    pub success: bool,
    pub actual_values: Vec<String>,
    pub message: String,
}

fn create_resolver() -> TokioResolver {
    Resolver::builder_with_config(
        ResolverConfig::default(),
        TokioConnectionProvider::default(),
    )
    .with_options(ResolverOpts::default())
    .build()
}

pub async fn verify_record(
    domain: &str,
    record: &DnsRecord,
) -> Result<VerificationResult, Box<dyn std::error::Error + Send + Sync>> {
    let resolver = create_resolver();

    let fqdn = if record.host == "@" {
        domain.to_string()
    } else {
        format!("{}.{}", record.host, domain)
    };

    let (actual_values, success, message) = match record.record_type {
        RecordType::A => verify_a(&resolver, &fqdn, &record.value).await?,
        RecordType::AAAA => verify_aaaa(&resolver, &fqdn, &record.value).await?,
        RecordType::CNAME => verify_cname(&resolver, &fqdn, &record.value).await?,
        RecordType::MX => verify_mx(&resolver, &fqdn, &record.value, record.priority).await?,
        RecordType::TXT => verify_txt(&resolver, &fqdn, &record.value).await?,
        RecordType::NS => verify_ns(&resolver, &fqdn, &record.value).await?,
        _ => (
            vec![],
            false,
            format!(
                "Verification not supported for {} records",
                record.record_type
            ),
        ),
    };

    Ok(VerificationResult {
        record: record.clone(),
        success,
        actual_values,
        message,
    })
}

async fn verify_a(
    resolver: &TokioResolver,
    fqdn: &str,
    expected: &str,
) -> Result<(Vec<String>, bool, String), Box<dyn std::error::Error + Send + Sync>> {
    let expected_ip = Ipv4Addr::from_str(expected)?;

    match resolver.ipv4_lookup(fqdn).await {
        Ok(lookup) => {
            let actual: Vec<String> = lookup.iter().map(|ip| ip.0.to_string()).collect();
            let found = lookup.iter().any(|ip| ip.0 == expected_ip);
            let message = if found {
                "Record verified".to_string()
            } else {
                format!("Expected {} not found in {:?}", expected, actual)
            };
            Ok((actual, found, message))
        }
        Err(e) => Ok((vec![], false, format!("DNS lookup failed: {}", e))),
    }
}

async fn verify_aaaa(
    resolver: &TokioResolver,
    fqdn: &str,
    expected: &str,
) -> Result<(Vec<String>, bool, String), Box<dyn std::error::Error + Send + Sync>> {
    let expected_ip = Ipv6Addr::from_str(expected)?;

    match resolver.ipv6_lookup(fqdn).await {
        Ok(lookup) => {
            let actual: Vec<String> = lookup.iter().map(|ip| ip.0.to_string()).collect();
            let found = lookup.iter().any(|ip| ip.0 == expected_ip);
            let message = if found {
                "Record verified".to_string()
            } else {
                format!("Expected {} not found in {:?}", expected, actual)
            };
            Ok((actual, found, message))
        }
        Err(e) => Ok((vec![], false, format!("DNS lookup failed: {}", e))),
    }
}

async fn verify_cname(
    resolver: &TokioResolver,
    fqdn: &str,
    expected: &str,
) -> Result<(Vec<String>, bool, String), Box<dyn std::error::Error + Send + Sync>> {
    match resolver
        .lookup(fqdn, hickory_resolver::proto::rr::RecordType::CNAME)
        .await
    {
        Ok(lookup) => {
            let actual: Vec<String> = lookup
                .iter()
                .filter_map(|r| {
                    r.as_cname()
                        .map(|c| c.to_string().trim_end_matches('.').to_string())
                })
                .collect();

            let expected_normalized = expected.trim_end_matches('.');
            let found = actual
                .iter()
                .any(|v| v.eq_ignore_ascii_case(expected_normalized));

            let message = if found {
                "Record verified".to_string()
            } else {
                format!("Expected {} not found in {:?}", expected, actual)
            };
            Ok((actual, found, message))
        }
        Err(e) => Ok((vec![], false, format!("DNS lookup failed: {}", e))),
    }
}

async fn verify_mx(
    resolver: &TokioResolver,
    fqdn: &str,
    expected: &str,
    expected_priority: Option<u16>,
) -> Result<(Vec<String>, bool, String), Box<dyn std::error::Error + Send + Sync>> {
    match resolver.mx_lookup(fqdn).await {
        Ok(lookup) => {
            let actual: Vec<String> = lookup
                .iter()
                .map(|mx| {
                    format!(
                        "{} {}",
                        mx.preference(),
                        mx.exchange().to_string().trim_end_matches('.')
                    )
                })
                .collect();

            let expected_normalized = expected.trim_end_matches('.');
            let found = lookup.iter().any(|mx| {
                let exchange_match = mx
                    .exchange()
                    .to_string()
                    .trim_end_matches('.')
                    .eq_ignore_ascii_case(expected_normalized);
                let priority_match = expected_priority
                    .map(|p| mx.preference() == p)
                    .unwrap_or(true);
                exchange_match && priority_match
            });

            let message = if found {
                "Record verified".to_string()
            } else {
                format!("Expected {} not found in {:?}", expected, actual)
            };
            Ok((actual, found, message))
        }
        Err(e) => Ok((vec![], false, format!("DNS lookup failed: {}", e))),
    }
}

async fn verify_txt(
    resolver: &TokioResolver,
    fqdn: &str,
    expected: &str,
) -> Result<(Vec<String>, bool, String), Box<dyn std::error::Error + Send + Sync>> {
    match resolver.txt_lookup(fqdn).await {
        Ok(lookup) => {
            let actual: Vec<String> = lookup.iter().map(|txt| txt.to_string()).collect();
            let found = actual
                .iter()
                .any(|v| v.contains(expected) || expected.contains(v.as_str()));

            let message = if found {
                "Record verified".to_string()
            } else {
                format!("Expected {} not found in {:?}", expected, actual)
            };
            Ok((actual, found, message))
        }
        Err(e) => Ok((vec![], false, format!("DNS lookup failed: {}", e))),
    }
}

async fn verify_ns(
    resolver: &TokioResolver,
    fqdn: &str,
    expected: &str,
) -> Result<(Vec<String>, bool, String), Box<dyn std::error::Error + Send + Sync>> {
    match resolver.ns_lookup(fqdn).await {
        Ok(lookup) => {
            let actual: Vec<String> = lookup
                .iter()
                .map(|ns| ns.to_string().trim_end_matches('.').to_string())
                .collect();

            let expected_normalized = expected.trim_end_matches('.');
            let found = actual
                .iter()
                .any(|v| v.eq_ignore_ascii_case(expected_normalized));

            let message = if found {
                "Record verified".to_string()
            } else {
                format!("Expected {} not found in {:?}", expected, actual)
            };
            Ok((actual, found, message))
        }
        Err(e) => Ok((vec![], false, format!("DNS lookup failed: {}", e))),
    }
}

pub async fn verify_with_wait(
    domain: &str,
    records: &[DnsRecord],
    timeout: Duration,
    interval: Duration,
) -> Vec<VerificationResult> {
    let start = std::time::Instant::now();
    let mut results: Vec<VerificationResult> = Vec::new();

    while start.elapsed() < timeout {
        results.clear();
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
                    results.push(VerificationResult {
                        record: record.clone(),
                        success: false,
                        actual_values: vec![],
                        message: format!("Verification error: {}", e),
                    });
                }
            }
        }

        if all_success {
            return results;
        }

        sleep(interval).await;
    }

    results
}
