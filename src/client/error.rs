use reqwest::StatusCode;
use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AixError {
    #[error("API error ({status}): {message}")]
    Api {
        status: StatusCode,
        message: String,
        error_code: Option<String>,
        response_body: Option<String>,
    },

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Resource not found: {resource_type} '{id}'")]
    NotFound { resource_type: String, id: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Operation timed out after {elapsed:?}")]
    Timeout {
        elapsed: std::time::Duration,
        operation: String,
    },

    #[error("Operation failed: {message}")]
    OperationFailed {
        message: String,
        supplier_error: Option<String>,
    },

    #[error("File upload failed: {0}")]
    Upload(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[allow(dead_code)]
impl AixError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Auth(_) => {
                format!("{self}\n\nRun `aix auth login` to configure credentials.")
            }
            Self::NotFound {
                resource_type, id, ..
            } => format!("Could not find {resource_type} with ID '{id}'."),
            Self::Timeout { operation, .. } => {
                format!("{self}\n\nThe {operation} is taking longer than expected. Try again or increase timeout.")
            }
            _ => self.to_string(),
        }
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Api { status, .. } => {
                matches!(status.as_u16(), 500 | 502 | 503 | 504 | 429)
            }
            Self::Http(e) => e.is_timeout() || e.is_connect(),
            _ => false,
        }
    }
}
