use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "ApiResponse")]
pub struct ApiResponse {
    #[serde(rename = "@Status")]
    pub status: String,
    #[serde(rename = "Errors", default)]
    pub errors: Option<Errors>,
    #[serde(rename = "CommandResponse")]
    pub command_response: Option<CommandResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Errors {
    #[serde(rename = "Error", default)]
    pub errors: Vec<ApiErrorItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorItem {
    #[serde(rename = "@Number")]
    pub number: String,
    #[serde(rename = "$text")]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    #[serde(rename = "@Type")]
    pub response_type: Option<String>,
    #[serde(rename = "DomainGetListResult")]
    pub domain_list: Option<DomainGetListResult>,
    #[serde(rename = "DomainDNSGetHostsResult")]
    pub dns_hosts: Option<DnsGetHostsResult>,
    #[serde(rename = "DomainDNSSetHostsResult")]
    pub dns_set_hosts: Option<DnsSetHostsResult>,
    #[serde(rename = "DomainDNSGetListResult")]
    pub dns_list: Option<DnsGetListResult>,
    #[serde(rename = "DomainDNSSetCustomResult")]
    pub dns_set_custom: Option<DnsSetCustomResult>,
    #[serde(rename = "DomainDNSSetDefaultResult")]
    pub dns_set_default: Option<DnsSetDefaultResult>,
    #[serde(rename = "DomainGetInfoResult")]
    pub domain_info: Option<DomainGetInfoResult>,
    #[serde(rename = "DomainCheckResult")]
    pub domain_check: Option<Vec<DomainCheckItem>>,
    #[serde(rename = "UserGetBalancesResult")]
    pub user_balances: Option<UserGetBalancesResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainGetListResult {
    #[serde(rename = "Domain", default)]
    pub domains: Vec<DomainItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainItem {
    #[serde(rename = "@ID")]
    pub id: String,
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@User")]
    pub user: Option<String>,
    #[serde(rename = "@Created")]
    pub created: Option<String>,
    #[serde(rename = "@Expires")]
    pub expires: Option<String>,
    #[serde(rename = "@IsExpired")]
    pub is_expired: Option<String>,
    #[serde(rename = "@IsLocked")]
    pub is_locked: Option<String>,
    #[serde(rename = "@AutoRenew")]
    pub auto_renew: Option<String>,
    #[serde(rename = "@WhoisGuard")]
    pub whois_guard: Option<String>,
    #[serde(rename = "@IsPremium")]
    pub is_premium: Option<String>,
    #[serde(rename = "@IsOurDNS")]
    pub is_our_dns: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsGetHostsResult {
    #[serde(rename = "@Domain")]
    pub domain: Option<String>,
    #[serde(rename = "@IsUsingOurDNS")]
    pub is_using_our_dns: Option<String>,
    #[serde(rename = "host", default)]
    pub hosts: Vec<HostRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostRecord {
    #[serde(rename = "@HostId")]
    pub host_id: Option<String>,
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@Type")]
    pub record_type: String,
    #[serde(rename = "@Address")]
    pub address: String,
    #[serde(rename = "@MXPref")]
    pub mx_pref: Option<String>,
    #[serde(rename = "@TTL")]
    pub ttl: Option<String>,
    #[serde(rename = "@AssociatedAppTitle")]
    pub associated_app_title: Option<String>,
    #[serde(rename = "@FriendlyName")]
    pub friendly_name: Option<String>,
    #[serde(rename = "@IsActive")]
    pub is_active: Option<String>,
    #[serde(rename = "@IsDDNSEnabled")]
    pub is_ddns_enabled: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSetHostsResult {
    #[serde(rename = "@Domain")]
    pub domain: Option<String>,
    #[serde(rename = "@IsSuccess")]
    pub is_success: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsGetListResult {
    #[serde(rename = "@Domain")]
    pub domain: Option<String>,
    #[serde(rename = "@IsUsingOurDNS")]
    pub is_using_our_dns: Option<String>,
    #[serde(rename = "Nameserver", default)]
    pub nameservers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSetCustomResult {
    #[serde(rename = "@Domain")]
    pub domain: Option<String>,
    #[serde(rename = "@Updated")]
    pub updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSetDefaultResult {
    #[serde(rename = "@Domain")]
    pub domain: Option<String>,
    #[serde(rename = "@Updated")]
    pub updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainGetInfoResult {
    #[serde(rename = "@Status")]
    pub status: Option<String>,
    #[serde(rename = "@ID")]
    pub id: Option<String>,
    #[serde(rename = "@DomainName")]
    pub domain_name: Option<String>,
    #[serde(rename = "@OwnerName")]
    pub owner_name: Option<String>,
    #[serde(rename = "@IsOwner")]
    pub is_owner: Option<String>,
    #[serde(rename = "@IsPremium")]
    pub is_premium: Option<String>,
    #[serde(rename = "DomainDetails")]
    pub domain_details: Option<DomainDetails>,
    #[serde(rename = "Whoisguard")]
    pub whois_guard: Option<WhoisGuard>,
    #[serde(rename = "DnsDetails")]
    pub dns_details: Option<DnsDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainDetails {
    #[serde(rename = "CreatedDate")]
    pub created_date: Option<String>,
    #[serde(rename = "ExpiredDate")]
    pub expired_date: Option<String>,
    #[serde(rename = "NumYears")]
    pub num_years: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoisGuard {
    #[serde(rename = "@Enabled")]
    pub enabled: Option<String>,
    #[serde(rename = "@ID")]
    pub id: Option<String>,
    #[serde(rename = "ExpiredDate")]
    pub expired_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsDetails {
    #[serde(rename = "@ProviderType")]
    pub provider_type: Option<String>,
    #[serde(rename = "@IsUsingOurDNS")]
    pub is_using_our_dns: Option<String>,
    #[serde(rename = "Nameserver", default)]
    pub nameservers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCheckItem {
    #[serde(rename = "@Domain")]
    pub domain: String,
    #[serde(rename = "@Available")]
    pub available: String,
    #[serde(rename = "@ErrorNo")]
    pub error_no: Option<String>,
    #[serde(rename = "@Description")]
    pub description: Option<String>,
    #[serde(rename = "@IsPremiumName")]
    pub is_premium_name: Option<String>,
    #[serde(rename = "@PremiumRegistrationPrice")]
    pub premium_registration_price: Option<String>,
    #[serde(rename = "@PremiumRenewalPrice")]
    pub premium_renewal_price: Option<String>,
    #[serde(rename = "@PremiumRestorePrice")]
    pub premium_restore_price: Option<String>,
    #[serde(rename = "@PremiumTransferPrice")]
    pub premium_transfer_price: Option<String>,
    #[serde(rename = "@IcannFee")]
    pub icann_fee: Option<String>,
    #[serde(rename = "@EapFee")]
    pub eap_fee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGetBalancesResult {
    #[serde(rename = "@Currency")]
    pub currency: Option<String>,
    #[serde(rename = "@AvailableBalance")]
    pub available_balance: Option<String>,
    #[serde(rename = "@AccountBalance")]
    pub account_balance: Option<String>,
    #[serde(rename = "@EarnedAmount")]
    pub earned_amount: Option<String>,
    #[serde(rename = "@WithdrawableAmount")]
    pub withdrawable_amount: Option<String>,
    #[serde(rename = "@FundsRequiredForAutoRenew")]
    pub funds_required_for_auto_renew: Option<String>,
}
