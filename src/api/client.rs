use super::error::ApiError;
use super::types::*;
use crate::config::Config;
use crate::dns::DnsRecord;
use psl::Psl;
use reqwest::Client;
use std::collections::HashMap;

pub struct NamecheapClient {
    client: Client,
    api_url: String,
    api_user: String,
    api_key: String,
    username: String,
    client_ip: String,
}

impl NamecheapClient {
    pub async fn new(config: &Config) -> Result<Self, ApiError> {
        let client = Client::builder()
            .user_agent("namecheap-cli/0.1.0")
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
        })
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

    fn base_params(&self) -> HashMap<&str, String> {
        let mut params = HashMap::new();
        params.insert("ApiUser", self.api_user.clone());
        params.insert("ApiKey", self.api_key.clone());
        params.insert("UserName", self.username.clone());
        params.insert("ClientIp", self.client_ip.clone());
        params
    }

    async fn execute(
        &self,
        command: &str,
        extra_params: HashMap<&str, String>,
    ) -> Result<ApiResponse, ApiError> {
        let mut params = self.base_params();
        params.insert("Command", command.to_string());
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
        params.insert("Page", page.to_string());
        params.insert("PageSize", page_size.to_string());

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
        params.insert("DomainName", domain.to_string());

        let response = self.execute("namecheap.domains.getInfo", params).await?;
        response
            .command_response
            .and_then(|cr| cr.domain_info)
            .ok_or_else(|| ApiError::InvalidResponse("Missing domain info in response".to_string()))
    }

    pub async fn check_domains(&self, domains: &[&str]) -> Result<Vec<DomainCheckItem>, ApiError> {
        let mut params = HashMap::new();
        params.insert("DomainList", domains.join(","));

        let response = self.execute("namecheap.domains.check", params).await?;
        Ok(response
            .command_response
            .and_then(|cr| cr.domain_check)
            .unwrap_or_default())
    }

    pub async fn get_hosts(&self, domain: &str) -> Result<Vec<DnsRecord>, ApiError> {
        let (sld, tld) = split_domain(domain);
        let mut params = HashMap::new();
        params.insert("SLD", sld);
        params.insert("TLD", tld);

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

    pub async fn set_hosts(&self, domain: &str, records: &[DnsRecord]) -> Result<(), ApiError> {
        let (sld, tld) = split_domain(domain);
        let mut params = HashMap::new();
        params.insert("SLD", sld);
        params.insert("TLD", tld);

        for (i, record) in records.iter().enumerate() {
            let idx = i + 1;
            params.insert(
                Box::leak(format!("HostName{}", idx).into_boxed_str()),
                record.host.clone(),
            );
            params.insert(
                Box::leak(format!("RecordType{}", idx).into_boxed_str()),
                record.record_type.to_string(),
            );
            params.insert(
                Box::leak(format!("Address{}", idx).into_boxed_str()),
                record.value.clone(),
            );
            params.insert(
                Box::leak(format!("TTL{}", idx).into_boxed_str()),
                record.ttl.to_string(),
            );
            if let Some(priority) = record.priority {
                params.insert(
                    Box::leak(format!("MXPref{}", idx).into_boxed_str()),
                    priority.to_string(),
                );
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

    pub async fn add_record(&self, domain: &str, record: DnsRecord) -> Result<(), ApiError> {
        let mut records = self.get_hosts(domain).await?;
        records.push(record);
        self.set_hosts(domain, &records).await
    }

    pub async fn remove_record(
        &self,
        domain: &str,
        record_type: &str,
        host: &str,
        value: Option<&str>,
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

        self.set_hosts(domain, &records).await?;
        Ok(true)
    }

    pub async fn get_nameservers(&self, domain: &str) -> Result<(bool, Vec<String>), ApiError> {
        let (sld, tld) = split_domain(domain);
        let mut params = HashMap::new();
        params.insert("SLD", sld);
        params.insert("TLD", tld);

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
        params.insert("SLD", sld);
        params.insert("TLD", tld);
        params.insert("Nameservers", nameservers.join(","));

        self.execute("namecheap.domains.dns.setCustom", params)
            .await?;
        Ok(())
    }

    pub async fn reset_nameservers(&self, domain: &str) -> Result<(), ApiError> {
        let (sld, tld) = split_domain(domain);
        let mut params = HashMap::new();
        params.insert("SLD", sld);
        params.insert("TLD", tld);

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
}
