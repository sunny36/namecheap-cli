mod diff;
mod record;
mod verify;

pub use diff::{apply_diff, calculate_diff, DiffAction, DnsRecordDiff};
pub use record::{parse_record_from_args, DnsRecord, RecordType};
pub use verify::{verify_record, verify_with_wait, VerificationResult};
