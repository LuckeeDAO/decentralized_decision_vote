use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteConfig {
    pub title: String,
    pub description: Option<String>,
    pub options: Vec<String>,
    pub commit_start_height: u64,
    pub commit_end_height: u64,
    pub reveal_start_height: u64,
    pub reveal_end_height: u64,
    pub participants: Vec<String>,
    pub value_template: String,
    pub template_params: Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Commitment { pub voter: String, pub commitment_hex: String, pub ts: i64 }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Reveal { pub voter: String, pub vote_value: Value, pub salt_hex: String, pub ts: i64 }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginationQuery { pub offset: Option<u64>, pub limit: Option<u64> }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Page<T> { pub items: Vec<T>, pub total: u64 }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateVoteRequest { pub config: VoteConfig }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteSummaryDto {
    pub id: String,
    pub title: String,
    pub commit_window: (u64, u64),
    pub reveal_window: (u64, u64),
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteDetailDto {
    pub id: String,
    pub config: VoteConfig,
    pub created_ts: i64,
    pub num_commitments: u64,
    pub num_reveals: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommitRequest { pub voter: String, pub vote_value: Value, pub salt_hex: String }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommitResponse { pub commitment_hex: String, pub ts: i64 }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RevealRequest { pub voter: String, pub vote_value: Value, pub salt_hex: String }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RevealResponse { pub accepted: bool, pub ts: i64 }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChainHeightDto { pub height: u64 }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteResultsDto { pub vote_id: String, pub result: Value }


