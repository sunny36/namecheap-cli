use thiserror::Error;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_GENERAL_ERROR: i32 = 1;
pub const EXIT_AUTH_ERROR: i32 = 2;
pub const EXIT_DOMAIN_NOT_FOUND: i32 = 3;
pub const EXIT_RECORD_NOT_FOUND: i32 = 4;
pub const EXIT_VALIDATION_ERROR: i32 = 5;
pub const EXIT_NETWORK_ERROR: i32 = 6;
pub const EXIT_VERIFICATION_FAILED: i32 = 7;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    #[error("Record not found: {0}")]
    RecordNotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("API error: {0}")]
    Api(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("DNS verification failed: {0}")]
    VerificationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Preset not found: {0}")]
    PresetNotFound(String),

    #[error("{0}")]
    Other(String),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Auth(_) => EXIT_AUTH_ERROR,
            CliError::DomainNotFound(_) => EXIT_DOMAIN_NOT_FOUND,
            CliError::RecordNotFound(_) => EXIT_RECORD_NOT_FOUND,
            CliError::Validation(_) => EXIT_VALIDATION_ERROR,
            CliError::Network(_) => EXIT_NETWORK_ERROR,
            CliError::VerificationFailed(_) => EXIT_VERIFICATION_FAILED,
            _ => EXIT_GENERAL_ERROR,
        }
    }
}

pub type Result<T> = std::result::Result<T, CliError>;
