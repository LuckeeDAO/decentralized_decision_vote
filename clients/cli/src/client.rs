use reqwest::Client;
use shared_types::*;
use shared_utils::crypto::{generate_salt, create_commitment};
use thiserror::Error;
use tracing::{debug, error};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("API error: {message}")]
    ApiError { message: String },
}

/// API client for communicating with the vote API
pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Result<Self, ApiError> {
        let client = Client::new();
        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }
    
    /// Create a new vote
    pub async fn create_vote(&self, config: VoteConfig) -> Result<CreateVoteResponse, ApiError> {
        debug!("Creating vote: {}", config.title);
        
        let request = CreateVoteRequest { config };
        let response = self.client
            .post(&format!("{}/api/v1/votes", self.base_url))
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: CreateVoteResponse = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(ApiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            })
        }
    }
    
    /// Get a vote by ID
    pub async fn get_vote(&self, vote_id: &str) -> Result<GetVoteResponse, ApiError> {
        debug!("Getting vote: {}", vote_id);
        
        let response = self.client
            .get(&format!("{}/api/v1/votes/{}", self.base_url, vote_id))
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: GetVoteResponse = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(ApiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            })
        }
    }
    
    /// List votes
    pub async fn list_votes(&self, query: ListQuery) -> Result<ListVotesResponse, ApiError> {
        debug!("Listing votes");
        
        let response = self.client
            .get(&format!("{}/api/v1/votes", self.base_url))
            .query(&query)
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: ListVotesResponse = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(ApiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            })
        }
    }
    
    /// Submit a commitment
    pub async fn commit_vote(&self, vote_id: &str, request: CommitRequest) -> Result<CommitResponse, ApiError> {
        debug!("Submitting commitment for vote: {}", vote_id);
        
        let response = self.client
            .post(&format!("{}/api/v1/votes/{}/commit", self.base_url, vote_id))
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: CommitResponse = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(ApiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            })
        }
    }
    
    /// Submit a reveal
    pub async fn reveal_vote(&self, vote_id: &str, request: RevealRequest) -> Result<RevealResponse, ApiError> {
        debug!("Submitting reveal for vote: {}", vote_id);
        
        let response = self.client
            .post(&format!("{}/api/v1/votes/{}/reveal", self.base_url, vote_id))
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: RevealResponse = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(ApiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            })
        }
    }
    
    /// Get vote results
    pub async fn get_results(&self, vote_id: &str) -> Result<GetResultsResponse, ApiError> {
        debug!("Getting results for vote: {}", vote_id);
        
        let response = self.client
            .get(&format!("{}/api/v1/votes/{}/results", self.base_url, vote_id))
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: GetResultsResponse = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(ApiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            })
        }
    }
    
    /// List templates
    pub async fn list_templates(&self) -> Result<serde_json::Value, ApiError> {
        debug!("Listing templates");
        
        let response = self.client
            .get(&format!("{}/api/v1/templates", self.base_url))
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(ApiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            })
        }
    }
    
    /// Get template information
    pub async fn get_template(&self, template_id: &str) -> Result<serde_json::Value, ApiError> {
        debug!("Getting template: {}", template_id);
        
        let response = self.client
            .get(&format!("{}/api/v1/templates/{}", self.base_url, template_id))
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(ApiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            })
        }
    }
    
    /// Verify vote results
    pub async fn verify_results(&self, vote_id: &str) -> Result<VerifyResultsResponse, ApiError> {
        debug!("Verifying results for vote: {}", vote_id);
        
        let response = self.client
            .get(&format!("{}/api/v1/votes/{}/verify", self.base_url, vote_id))
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: VerifyResultsResponse = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(ApiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            })
        }
    }
    
    /// Health check
    pub async fn health_check(&self) -> Result<HealthResponse, ApiError> {
        debug!("Performing health check");
        
        let response = self.client
            .get(&format!("{}/health", self.base_url))
            .send()
            .await?;
        
        if response.status().is_success() {
            let result: HealthResponse = response.json().await?;
            Ok(result)
        } else {
            let status = response.status();
            let text = response.text().await?;
            Err(ApiError::ApiError {
                message: format!("HTTP {}: {}", status, text),
            })
        }
    }
    
    /// Create a commitment for a vote value
    pub fn create_commitment(&self, value: &str, salt: Option<String>) -> (String, String) {
        let salt = salt.unwrap_or_else(generate_salt);
        let commitment_hash = create_commitment(value, &salt);
        (commitment_hash, salt)
    }
}
