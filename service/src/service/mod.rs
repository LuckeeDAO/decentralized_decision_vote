use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use chrono::Utc;
use sha2::{Sha256, Digest};
use hex::ToHex;

use crate::model::vote::*;
use crate::store::{VoteStore, StoreError};
use crate::core::template::TemplateRegistry;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError { 
    #[error("bad request: {0}")] BadRequest(String), 
    #[error("not found")] NotFound, 
    #[error("conflict")] Conflict, 
    #[error("forbidden")] Forbidden, 
    #[error("internal")] Internal 
}

impl From<StoreError> for ServiceError {
    fn from(e: StoreError) -> Self { match e { StoreError::NotFound => ServiceError::NotFound, StoreError::Conflict => ServiceError::Conflict, _ => ServiceError::Internal } }
}

#[async_trait]
pub trait VoteService: Send + Sync {
    async fn create_vote(&self, cfg: VoteConfig) -> Result<String, ServiceError>;
    async fn list_votes(&self, offset: u64, limit: u64) -> Result<(Vec<VoteSummaryDto>, u64), ServiceError>;
    async fn get_vote(&self, id: &str) -> Result<VoteDetailDto, ServiceError>;
    async fn commit(&self, id: &str, voter: &str, raw_value: Value, salt_hex: String) -> Result<CommitResponse, ServiceError>;
    async fn reveal(&self, id: &str, voter: &str, raw_value: Value, salt_hex: String) -> Result<RevealResponse, ServiceError>;
    async fn results(&self, id: &str) -> Result<VoteResultsDto, ServiceError>;
}

pub struct VoteServiceImpl {
    store: Arc<dyn VoteStore>,
    registry: Arc<TemplateRegistry>,
}

impl VoteServiceImpl {
    pub fn new(store: Arc<dyn VoteStore>, registry: Arc<TemplateRegistry>) -> Self { Self { store, registry } }
}

#[async_trait]
impl VoteService for VoteServiceImpl {
    async fn create_vote(&self, cfg: VoteConfig) -> Result<String, ServiceError> {
        // basic sanity
        if cfg.commit_start_height > cfg.commit_end_height || cfg.reveal_start_height > cfg.reveal_end_height { return Err(ServiceError::BadRequest("invalid windows".into())); }
        // template exists
        let _ = self.registry.get(&cfg.value_template).map_err(ServiceError::BadRequest)?;
        self.store.create_vote(cfg).await.map_err(Into::into)
    }

    async fn list_votes(&self, offset: u64, limit: u64) -> Result<(Vec<VoteSummaryDto>, u64), ServiceError> {
        self.store.list_votes(offset, limit).await.map_err(Into::into)
    }

    async fn get_vote(&self, id: &str) -> Result<VoteDetailDto, ServiceError> {
        self.store.get_vote(id).await.map_err(Into::into)
    }

    async fn commit(&self, id: &str, voter: &str, raw_value: Value, salt_hex: String) -> Result<CommitResponse, ServiceError> {
        let vote = self.store.get_vote(id).await?;
        if !vote.config.participants.is_empty() && !vote.config.participants.iter().any(|p| p == voter) { return Err(ServiceError::Forbidden); }
        let tpl = self.registry.get(&vote.config.value_template).map_err(ServiceError::BadRequest)?;
        tpl.validate(&raw_value, &vote.config.template_params).map_err(ServiceError::BadRequest)?;
        let canon = tpl.canonicalize(&raw_value, &vote.config.template_params).map_err(ServiceError::BadRequest)?;
        let salt_bytes = hex::decode(&salt_hex).map_err(|_| ServiceError::BadRequest("bad salt".into()))?;
        let mut hasher = Sha256::new();
        hasher.update(b"commit|");
        hasher.update(&canon);
        hasher.update(b"|");
        hasher.update(&salt_bytes);
        let commitment_hex: String = hasher.finalize().encode_hex();
        let ts = Utc::now().timestamp();
        self.store.put_commitment(id, Commitment { voter: voter.to_string(), commitment_hex: commitment_hex.clone(), ts }).await?;
        Ok(CommitResponse { commitment_hex, ts })
    }

    async fn reveal(&self, id: &str, voter: &str, raw_value: Value, salt_hex: String) -> Result<RevealResponse, ServiceError> {
        let vote = self.store.get_vote(id).await?;
        let tpl = self.registry.get(&vote.config.value_template).map_err(ServiceError::BadRequest)?;
        tpl.validate(&raw_value, &vote.config.template_params).map_err(ServiceError::BadRequest)?;
        let canon = tpl.canonicalize(&raw_value, &vote.config.template_params).map_err(ServiceError::BadRequest)?;
        let salt_bytes = hex::decode(&salt_hex).map_err(|_| ServiceError::BadRequest("bad salt".into()))?;
        // recompute and compare with stored commitment
        let mut hasher = Sha256::new();
        hasher.update(b"commit|");
        hasher.update(&canon);
        hasher.update(b"|");
        hasher.update(&salt_bytes);
        let commitment_hex: String = hasher.finalize().encode_hex();
        if let Some(comm) = self.store.get_commitment(id, voter).await? {
            if comm.commitment_hex != commitment_hex { return Err(ServiceError::BadRequest("commitment mismatch".into())); }
        } else { return Err(ServiceError::BadRequest("no commitment".into())); }
        let ts = Utc::now().timestamp();
        self.store.put_reveal(id, Reveal { voter: voter.to_string(), vote_value: raw_value, salt_hex, ts }).await?;
        Ok(RevealResponse { accepted: true, ts })
    }

    async fn results(&self, id: &str) -> Result<VoteResultsDto, ServiceError> {
        let vote = self.store.get_vote(id).await?;
        let reveals = self.store.list_reveals(id).await?;
        let values: Vec<Value> = reveals.into_iter().map(|r| r.vote_value).collect();
        let tpl = self.registry.get(&vote.config.value_template).map_err(ServiceError::BadRequest)?;
        let aggregated = tpl.reduce(&values);
        Ok(VoteResultsDto { vote_id: id.to_string(), result: aggregated })
    }
}


