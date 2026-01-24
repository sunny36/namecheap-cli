use crate::api::HostRecord;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[allow(clippy::upper_case_acronyms)]
pub enum RecordType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    NS,
    SRV,
    CAA,
    ALIAS,
    URL,
    URL301,
    FRAME,
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordType::A => write!(f, "A"),
            RecordType::AAAA => write!(f, "AAAA"),
            RecordType::CNAME => write!(f, "CNAME"),
            RecordType::MX => write!(f, "MX"),
            RecordType::TXT => write!(f, "TXT"),
            RecordType::NS => write!(f, "NS"),
            RecordType::SRV => write!(f, "SRV"),
            RecordType::CAA => write!(f, "CAA"),
            RecordType::ALIAS => write!(f, "ALIAS"),
            RecordType::URL => write!(f, "URL"),
            RecordType::URL301 => write!(f, "URL301"),
            RecordType::FRAME => write!(f, "FRAME"),
        }
    }
}

impl FromStr for RecordType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "A" => Ok(RecordType::A),
            "AAAA" => Ok(RecordType::AAAA),
            "CNAME" => Ok(RecordType::CNAME),
            "MX" => Ok(RecordType::MX),
            "TXT" => Ok(RecordType::TXT),
            "NS" => Ok(RecordType::NS),
            "SRV" => Ok(RecordType::SRV),
            "CAA" => Ok(RecordType::CAA),
            "ALIAS" => Ok(RecordType::ALIAS),
            "URL" => Ok(RecordType::URL),
            "URL301" => Ok(RecordType::URL301),
            "FRAME" => Ok(RecordType::FRAME),
            _ => Err(format!("Unknown record type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: RecordType,
    pub host: String,
    pub value: String,
    pub ttl: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
}

impl DnsRecord {
    pub fn new(
        record_type: RecordType,
        host: &str,
        value: &str,
        ttl: u32,
        priority: Option<u16>,
    ) -> Self {
        Self {
            record_type,
            host: host.to_string(),
            value: value.to_string(),
            ttl,
            priority,
        }
    }

    pub fn matches(&self, other: &DnsRecord) -> bool {
        self.record_type == other.record_type
            && self.host.eq_ignore_ascii_case(&other.host)
            && self.value.eq_ignore_ascii_case(&other.value)
    }

    pub fn matches_key(&self, record_type: &str, host: &str) -> bool {
        self.record_type
            .to_string()
            .eq_ignore_ascii_case(record_type)
            && self.host.eq_ignore_ascii_case(host)
    }
}

impl From<HostRecord> for DnsRecord {
    fn from(host: HostRecord) -> Self {
        let record_type = host.record_type.parse().unwrap_or(RecordType::A);
        let ttl = host.ttl.and_then(|t| t.parse().ok()).unwrap_or(1800);
        let priority = host.mx_pref.and_then(|p| p.parse().ok());

        DnsRecord {
            record_type,
            host: host.name,
            value: host.address,
            ttl,
            priority,
        }
    }
}

impl fmt::Display for DnsRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(priority) = self.priority {
            write!(
                f,
                "{} {} {} {} {}",
                self.record_type, self.host, self.value, self.ttl, priority
            )
        } else {
            write!(
                f,
                "{} {} {} {}",
                self.record_type, self.host, self.value, self.ttl
            )
        }
    }
}

pub fn parse_record_from_args(
    record_type: &str,
    host: &str,
    value: &str,
    ttl: Option<u32>,
    priority: Option<u16>,
) -> Result<DnsRecord, String> {
    let record_type: RecordType = record_type.parse()?;
    let ttl = ttl.unwrap_or(1800);

    let priority = match record_type {
        RecordType::MX => Some(priority.unwrap_or(10)),
        _ => priority,
    };

    Ok(DnsRecord::new(record_type, host, value, ttl, priority))
}
