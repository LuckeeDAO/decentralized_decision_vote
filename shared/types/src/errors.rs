use thiserror::Error;
use shared_utils::validation::ValidationError;

#[derive(Error, Debug)]
pub enum VoteError {
    #[error("Vote not found: {id}")]
    VoteNotFound { id: String },
    
    #[error("Invalid vote configuration: {message}")]
    InvalidConfig { message: String },
    
    #[error("Vote is not in the correct state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },
    
    #[error("Commitment phase is not active")]
    CommitmentPhaseNotActive,
    
    #[error("Reveal phase is not active")]
    RevealPhaseNotActive,
    
    #[error("Vote has already ended")]
    VoteEnded,
    
    #[error("Invalid commitment: {message}")]
    InvalidCommitment { message: String },
    
    #[error("Invalid reveal: {message}")]
    InvalidReveal { message: String },
    
    #[error("Template error: {message}")]
    TemplateError { message: String },
    
    #[error("Storage error: {message}")]
    StorageError { message: String },
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Bad request: {message}")]
    BadRequest { message: String },
    
    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },
    
    #[error("Forbidden: {message}")]
    Forbidden { message: String },
    
    #[error("Not found: {message}")]
    NotFound { message: String },
    
    #[error("Internal server error: {message}")]
    InternalError { message: String },
    
    #[error("Vote error: {0}")]
    VoteError(#[from] VoteError),
}

impl ApiError {
    pub fn status_code(&self) -> u16 {
        match self {
            ApiError::BadRequest { .. } => 400,
            ApiError::Unauthorized { .. } => 401,
            ApiError::Forbidden { .. } => 403,
            ApiError::NotFound { .. } => 404,
            ApiError::InternalError { .. } => 500,
            ApiError::VoteError(_) => 500,
        }
    }
}
