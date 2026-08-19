use super::error::ApiError;
use super::types::*;
use crate::config::Config;
use crate::dns::DnsRecord;
use psl::Psl;
use reqwest::Client;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct NamecheapClient {
    client: Client,
    api_url: String,
    api_user: String,
    api_key: String,
    username: String,
    client_ip: String,
    backup_dir: Option<PathBuf>,
}

/// Outcome of `add_record`. Identical type/host/value is a no-op.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddRecordResult {
    Added,
    Unchanged,
}

impl NamecheapClient {
    pub async fn new(config: &Config) -> Result<Self, ApiError> {
        let client = Client::builder()
            .user_agent(concat!("namecheap-cli/", env!("CARGO_PKG_VERSION")))
            .build()?;

        let client_ip = match &config.profile.client_ip {
            Some(ip) => ip.clone(),
            None => Self::detect_client_ip(&client).await?,
        };

        Ok(Self {
            client,
            api_url: config.api_url().to_string(),
            api_user: config.profile.api_user.clone(),
            api_key: config.profile.api_key.clone(),
            username: config.username().to_string(),
            client_ip,
            backup_dir: None,
        })
    }

    /// Build a client pointed at an arbitrary API URL (used by mocked tests).
    pub fn from_parts(
        api_url: impl Into<String>,
        api_user: impl Into<String>,
        api_key: impl Into<String>,
        username: impl Into<String>,
        client_ip: impl Into<String>,
    ) -> Result<Self, ApiError> {
        let client = Client::builder()
            .user_agent(concat!("namecheap-cli/", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self {
            client,
            api_url: api_url.into(),
            api_user: api_user.into(),
            api_key: api_key.into(),
            username: username.into(),
            client_ip: client_ip.into(),
            backup_dir: None,
        })
    }

    pub fn with_backup_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.backup_dir = Some(dir.into());
        self
    }

    async fn detect_client_ip(client: &Client) -> Result<String, ApiError> {
        let response = client
            .get("https://api.ipify.org")
            .send()
            .await?
            .text()
            .await?;
        Ok(response.trim().to_string())
    }

    fn base_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("ApiUser".to_string(), self.api_user.clone());
        params.insert("ApiKey".to_string(), self.api_key.clone());
        params.insert("UserName".to_string(), self.username.clone());
        params.insert("ClientIp".to_string(), self.client_ip.clone());
        params
    }

    async fn execute(
        &self,
        command: &str,
        extra_params: HashMap<String, String>,
    ) -> Result<ApiResponse, ApiError> {
        let mut params = self.base_params();
        params.insert("Command".to_string(), command.to_string());
        for (k, v) in extra_params {
            params.insert(k, v);
        }

        let response = self.client.post(&self.api_url).form(&params).send().await?;

        let text = response.text().await?;
        let api_response: ApiResponse = quick_xml::de::from_str(&text)?;

        if api_response.status != "OK" {
            if let Some(errors) = &api_response.errors {
                let error_list: Vec<(String, String)> = errors
                    .errors
                    .iter()
                    .map(|e| (e.number.clone(), e.message.clone()))
                    .collect();
                return Err(ApiError::from_api_response(error_list));
            }
            return Err(ApiError::InvalidResponse("Unknown error".to_string()));
        }

        Ok(api_response)
    }

    pub async fn get_balances(&self) -> Result<UserGetBalancesResult, ApiError> {
        let response = self
            .execute("namecheap.users.getBalances", HashMap::new())
            .await?;
        response
            .command_response
            .and_then(|cr| cr.user_balances)
            .ok_or_else(|| ApiError::InvalidResponse("Missing balances in response".to_string()))
    }

    pub async fn list_domains(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<DomainItem>, ApiError> {
        let mut params = HashMap::new();
        params.insert("Page".to_string(), page.to_string());
        params.insert("PageSize".to_string(), page_size.to_string());

        let response = self.execute("namecheap.domains.getList", params).await?;
        Ok(response
            .command_response
            .and_then(|cr| cr.domain_list)
            .map(|dl| dl.domains)
            .unwrap_or_default())
    }

    pub async fn list_all_domains(&self) -> Result<Vec<DomainItem>, ApiError> {
        let mut all_domains = Vec::new();
        let mut page = 1;
        let page_size = 100;

        loop {
            let domains = self.list_domains(page, page_size).await?;
            let count = domains.len();
            all_domains.extend(domains);

            if count < page_size as usize {
                break;
            }
            page += 1;
        }

        Ok(all_domains)
    }

    pub async fn get_domain_info(&self, domain: &str) -> Result<DomainGetInfoResult, ApiError> {
        let mut params = HashMap::new();
        params.insert("DomainName".to_string(), domain.to_string());

        let response = self.execute("namecheap.domains.getInfo", params).await?;
        response
            .command_response
            .and_then(|cr| cr.domain_info)
            .ok_or_else(|| ApiError::InvalidResponse("Missing domain info in response".to_string()))
    }

    pub async fn check_domains(&self, domains: &[&str]) -> Result<Vec<DomainCheckItem>, ApiError> {
        let mut params = HashMap::new();
        params.insert("DomainList".to_string(), domains.join(","));

        let response = self.execute("namecheap.domains.check", params).await?;
        Ok(response
            .command_response
            .and_then(|cr| cr.domain_check)
            .unwrap_or_default())
    }

    pub async fn get_hosts(&self, domain: &str) -> Result<Vec<DnsRecord>, ApiError> {
        let (sld, tld) = split_domain(domain);
        let mut params = HashMap::new();
        params.insert("SLD".to_string(), sld);
        params.insert("TLD".to_string(), tld);

        let response = self
            .execute("namecheap.domains.dns.getHosts", params)
            .await?;
        let hosts = response
            .command_response
            .and_then(|cr| cr.dns_hosts)
            .map(|dh| dh.hosts)
            .unwrap_or_default();

        Ok(hosts.into_iter().map(DnsRecord::from).collect())
    }

    /// Replace the full zone. Namecheap `setHosts` is a complete replace.
    ///
    /// Refuses a zero-record payload unless `allow_empty` is set, writes a
    /// timestamped JSON backup of the current zone first, and never leaks
    /// formatted form keys.
    pub async fn set_hosts(
        &self,
        domain: &str,
        records: &[DnsRecord],
        allow_empty: bool,
    ) -> Result<(), ApiError> {
        if records.is_empty() && !allow_empty {
            return Err(ApiError::EmptyZone(format!(
                "refusing to replace {} with zero records (Namecheap setHosts is a full-zone replace); pass --allow-empty to override",
                domain
            )));
        }

        self.backup_zone(domain).await?;

        let (sld, tld) = split_domain(domain);
        let mut params = HashMap::new();
        params.insert("SLD".to_string(), sld);
        params.insert("TLD".to_string(), tld);

        for (i, record) in records.iter().enumerate() {
            let idx = i + 1;
            params.insert(format!("HostName{}", idx), record.host.clone());
            params.insert(format!("RecordType{}", idx), record.record_type.to_string());
            params.insert(format!("Address{}", idx), record.value.clone());
            params.insert(format!("TTL{}", idx), record.ttl.to_string());
            if let Some(priority) = record.priority {
                params.insert(format!("MXPref{}", idx), priority.to_string());
            }
        }

        let response = self
            .execute("namecheap.domains.dns.setHosts", params)
            .await?;

        if let Some(cr) = response.command_response {
            if let Some(result) = cr.dns_set_hosts {
                if result.is_success.as_deref() == Some("true") {
                    return Ok(());
                }
            }
        }

        Err(ApiError::InvalidResponse(
            "Failed to set DNS hosts".to_string(),
        ))
    }

    pub async fn add_record(
        &self,
        domain: &str,
        record: DnsRecord,
        allow_empty: bool,
    ) -> Result<AddRecordResult, ApiError> {
        let mut records = self.get_hosts(domain).await?;

        if records.iter().any(|r| {
            r.record_type == record.record_type
                && r.host.eq_ignore_ascii_case(&record.host)
                && r.value.eq_ignore_ascii_case(&record.value)
        }) {
            return Ok(AddRecordResult::Unchanged);
        }

        if let Some(existing) = records.iter().find(|r| {
            r.record_type == record.record_type && r.host.eq_ignore_ascii_case(&record.host)
        }) {
            return Err(ApiError::RecordConflict(format!(
                "{} record for host '{}' already exists with value '{}'. Use `dns set` to replace it.",
                existing.record_type, existing.host, existing.value
            )));
        }

        records.push(record);
        self.set_hosts(domain, &records, allow_empty).await?;
        Ok(AddRecordResult::Added)
    }

    pub async fn set_record(
        &self,
        domain: &str,
        record: DnsRecord,
        allow_empty: bool,
    ) -> Result<(), ApiError> {
        let record_type = record.record_type.to_string();
        let host = record.host.clone();
        let mut records = self.get_hosts(domain).await?;
        records.retain(|r| !r.matches_key(&record_type, &host));
        records.push(record);
        self.set_hosts(domain, &records, allow_empty).await
    }

    pub async fn remove_record(
        &self,
        domain: &str,
        record_type: &str,
        host: &str,
        value: Option<&str>,
        allow_empty: bool,
    ) -> Result<bool, ApiError> {
        let mut records = self.get_hosts(domain).await?;
        let original_len = records.len();

        records.retain(|r| {
            let type_match = r.record_type.to_string().eq_ignore_ascii_case(record_type);
            let host_match = r.host.eq_ignore_ascii_case(host);
            let value_match = value
                .map(|v| r.value.eq_ignore_ascii_case(v))
                .unwrap_or(true);
            !(type_match && host_match && value_match)
        });

        if records.len() == original_len {
            return Ok(false);
        }

        self.set_hosts(domain, &records, allow_empty).await?;
        Ok(true)
    }

    async fn backup_zone(&self, domain: &str) -> Result<PathBuf, ApiError> {
        let current = self.get_hosts(domain).await?;
        let dir = match &self.backup_dir {
            Some(dir) => dir.clone(),
            None => crate::config::backup_dir().ok_or_else(|| {
                ApiError::Backup("could not determine cache directory for zone backup".to_string())
            })?,
        };
        write_zone_backup(&dir, domain, &current)
    }

    pub async fn get_nameservers(&self, domain: &str) -> Result<(bool, Vec<String>), ApiError> {
        let (sld, tld) = split_domain(domain);
        let mut params = HashMap::new();
        params.insert("SLD".to_string(), sld);
        params.insert("TLD".to_string(), tld);

        let response = self
            .execute("namecheap.domains.dns.getList", params)
            .await?;
        let result = response
            .command_response
            .and_then(|cr| cr.dns_list)
            .ok_or_else(|| ApiError::InvalidResponse("Missing DNS list in response".to_string()))?;

        let is_using_our_dns = result.is_using_our_dns.as_deref() == Some("true");
        Ok((is_using_our_dns, result.nameservers))
    }

    pub async fn set_nameservers(
        &self,
        domain: &str,
        nameservers: &[&str],
    ) -> Result<(), ApiError> {
        let (sld, tld) = split_domain(domain);
        let mut params = HashMap::new();
        params.insert("SLD".to_string(), sld);
        params.insert("TLD".to_string(), tld);
        params.insert("Nameservers".to_string(), nameservers.join(","));

        self.execute("namecheap.domains.dns.setCustom", params)
            .await?;
        Ok(())
    }

    pub async fn reset_nameservers(&self, domain: &str) -> Result<(), ApiError> {
        let (sld, tld) = split_domain(domain);
        let mut params = HashMap::new();
        params.insert("SLD".to_string(), sld);
        params.insert("TLD".to_string(), tld);

        self.execute("namecheap.domains.dns.setDefault", params)
            .await?;
        Ok(())
    }
}

pub fn split_domain(domain: &str) -> (String, String) {
    let domain_lower = domain.to_lowercase();
    let list = psl::List;

    if let Some(suffix) = list.suffix(domain_lower.as_bytes()) {
        let suffix_str = std::str::from_utf8(suffix.as_bytes()).unwrap_or("");
        if let Some(sld) = domain_lower
            .strip_suffix(suffix_str)
            .and_then(|s| s.strip_suffix('.'))
        {
            return (sld.to_string(), suffix_str.to_string());
        }
    }

    // Fallback for simple TLDs
    let parts: Vec<&str> = domain_lower.splitn(2, '.').collect();
    (
        parts[0].to_string(),
        parts.get(1).unwrap_or(&"").to_string(),
    )
}

fn sanitize_domain_for_filename(domain: &str) -> String {
    domain
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn write_zone_backup(dir: &Path, domain: &str, records: &[DnsRecord]) -> Result<PathBuf, ApiError> {
    std::fs::create_dir_all(dir).map_err(|e| ApiError::Backup(e.to_string()))?;

    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let filename = format!("{}-{}.json", sanitize_domain_for_filename(domain), millis);
    let path = dir.join(filename);

    let payload = serde_json::json!({
        "domain": domain,
        "backed_up_at_unix_ms": millis,
        "records": records,
    });
    let body =
        serde_json::to_string_pretty(&payload).map_err(|e| ApiError::Backup(e.to_string()))?;
    std::fs::write(&path, body).map_err(|e| ApiError::Backup(e.to_string()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_domain_simple() {
        let (sld, tld) = split_domain("example.com");
        assert_eq!(sld, "example");
        assert_eq!(tld, "com");
    }

    #[test]
    fn test_split_domain_co_uk() {
        let (sld, tld) = split_domain("example.co.uk");
        assert_eq!(sld, "example");
        assert_eq!(tld, "co.uk");
    }

    #[test]
    fn test_split_domain_readyms_xyz() {
        let (sld, tld) = split_domain("readyms.xyz");
        assert_eq!(sld, "readyms");
        assert_eq!(tld, "xyz");
    }
}
