use super::record::DnsRecord;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum DiffAction {
    Add,
    Remove,
    Modify,
}

#[derive(Debug, Clone, Serialize)]
pub struct DnsRecordDiff {
    pub action: DiffAction,
    pub record: DnsRecord,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_record: Option<DnsRecord>,
}

impl DnsRecordDiff {
    pub fn add(record: DnsRecord) -> Self {
        Self {
            action: DiffAction::Add,
            record,
            old_record: None,
        }
    }

    pub fn remove(record: DnsRecord) -> Self {
        Self {
            action: DiffAction::Remove,
            record,
            old_record: None,
        }
    }

    pub fn modify(new_record: DnsRecord, old_record: DnsRecord) -> Self {
        Self {
            action: DiffAction::Modify,
            record: new_record,
            old_record: Some(old_record),
        }
    }
}

pub fn calculate_diff(current: &[DnsRecord], desired: &[DnsRecord]) -> Vec<DnsRecordDiff> {
    let mut diffs = Vec::new();

    // Find records to remove or modify
    for current_record in current {
        let matching_desired = desired.iter().find(|d| {
            d.record_type == current_record.record_type
                && d.host.eq_ignore_ascii_case(&current_record.host)
                && d.value.eq_ignore_ascii_case(&current_record.value)
        });

        match matching_desired {
            Some(desired_record) => {
                // Record exists, check if TTL or priority changed
                if desired_record.ttl != current_record.ttl
                    || desired_record.priority != current_record.priority
                {
                    diffs.push(DnsRecordDiff::modify(
                        desired_record.clone(),
                        current_record.clone(),
                    ));
                }
            }
            None => {
                // Record doesn't exist in desired, should be removed
                diffs.push(DnsRecordDiff::remove(current_record.clone()));
            }
        }
    }

    // Find records to add
    for desired_record in desired {
        let exists = current.iter().any(|c| {
            c.record_type == desired_record.record_type
                && c.host.eq_ignore_ascii_case(&desired_record.host)
                && c.value.eq_ignore_ascii_case(&desired_record.value)
        });

        if !exists {
            diffs.push(DnsRecordDiff::add(desired_record.clone()));
        }
    }

    diffs
}

pub fn apply_diff(current: &[DnsRecord], diffs: &[DnsRecordDiff]) -> Vec<DnsRecord> {
    let mut result: Vec<DnsRecord> = current.to_vec();

    for diff in diffs {
        match diff.action {
            DiffAction::Add => {
                result.push(diff.record.clone());
            }
            DiffAction::Remove => {
                result.retain(|r| !r.matches(&diff.record));
            }
            DiffAction::Modify => {
                if let Some(old) = &diff.old_record {
                    result.retain(|r| !r.matches(old));
                    result.push(diff.record.clone());
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dns::RecordType;

    #[test]
    fn test_calculate_diff_add() {
        let current = vec![];
        let desired = vec![DnsRecord::new(RecordType::A, "@", "1.2.3.4", 1800, None)];

        let diffs = calculate_diff(&current, &desired);
        assert_eq!(diffs.len(), 1);
        assert!(matches!(diffs[0].action, DiffAction::Add));
    }

    #[test]
    fn test_calculate_diff_remove() {
        let current = vec![DnsRecord::new(RecordType::A, "@", "1.2.3.4", 1800, None)];
        let desired = vec![];

        let diffs = calculate_diff(&current, &desired);
        assert_eq!(diffs.len(), 1);
        assert!(matches!(diffs[0].action, DiffAction::Remove));
    }

    #[test]
    fn test_calculate_diff_no_change() {
        let current = vec![DnsRecord::new(RecordType::A, "@", "1.2.3.4", 1800, None)];
        let desired = vec![DnsRecord::new(RecordType::A, "@", "1.2.3.4", 1800, None)];

        let diffs = calculate_diff(&current, &desired);
        assert!(diffs.is_empty());
    }
}
