use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("API error ({code}): {message}")]
    ApiResponse { code: String, message: String },

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Domain not found: {0}")]
    DomainNotFound(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::DeError),
}

impl ApiError {
    pub fn from_api_response(errors: Vec<(String, String)>) -> Self {
        if errors.is_empty() {
            return ApiError::InvalidResponse("Unknown API error".to_string());
        }

        let (code, message) = &errors[0];

        if code.starts_with("1011") || message.to_lowercase().contains("authentication") {
            return ApiError::Auth(message.clone());
        }

        if code == "2019166" || message.to_lowercase().contains("not found") {
            return ApiError::DomainNotFound(message.clone());
        }

        ApiError::ApiResponse {
            code: code.clone(),
            message: message.clone(),
        }
    }
}
