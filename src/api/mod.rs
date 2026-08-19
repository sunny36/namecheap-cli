mod client;
mod error;
mod types;

pub use client::{split_domain, AddRecordResult, NamecheapClient};
pub use error::ApiError;
pub use types::*;
