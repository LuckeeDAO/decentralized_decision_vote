use async_trait::async_trait;
use sqlx::{SqlitePool, Row};
use shared_types::*;
use shared_config::DatabaseConfig;
use tracing::{debug, info};

use crate::traits::{VoteStore, StoreError, StoreStats};

/// SQLite implementation of VoteStore
pub struct SqliteVoteStore {
    pool: SqlitePool,
}

impl SqliteVoteStore {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, StoreError> {
        info!("Connecting to SQLite database: {}", config.url);
        
        let pool = SqlitePool::connect(&config.url)
            .await
            .map_err(|e| StoreError::ConnectionError {
                message: format!("Failed to connect to SQLite: {}", e),
            })?;
        
        let store = Self { pool };
        store.init_tables().await?;
        
        Ok(store)
    }
    
    async fn init_tables(&self) -> Result<(), StoreError> {
        info!("Initializing SQLite tables");
        
        // Create votes table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS votes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                template_id TEXT NOT NULL,
                template_params TEXT NOT NULL,
                creator TEXT NOT NULL,
                created_at TEXT NOT NULL,
                commitment_start TEXT NOT NULL,
                commitment_end TEXT NOT NULL,
                reveal_start TEXT NOT NULL,
                reveal_end TEXT NOT NULL,
                status TEXT NOT NULL,
                results TEXT
            )
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Create commitments table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS commitments (
                id TEXT PRIMARY KEY,
                vote_id TEXT NOT NULL,
                voter TEXT NOT NULL,
                commitment_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(vote_id, voter)
            )
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Create reveals table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reveals (
                id TEXT PRIMARY KEY,
                vote_id TEXT NOT NULL,
                voter TEXT NOT NULL,
                value TEXT NOT NULL,
                salt TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(vote_id, voter)
            )
            "#
        )
        .execute(&self.pool)
        .await?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_commitments_vote_id ON commitments(vote_id)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_reveals_vote_id ON reveals(vote_id)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_votes_creator ON votes(creator)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_votes_status ON votes(status)")
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    fn vote_status_to_string(status: &VoteStatus) -> String {
        match status {
            VoteStatus::Created => "created".to_string(),
            VoteStatus::CommitmentPhase => "commitment_phase".to_string(),
            VoteStatus::RevealPhase => "reveal_phase".to_string(),
            VoteStatus::Completed => "completed".to_string(),
            VoteStatus::Cancelled => "cancelled".to_string(),
        }
    }
    
    fn string_to_vote_status(s: &str) -> VoteStatus {
        match s {
            "created" => VoteStatus::Created,
            "commitment_phase" => VoteStatus::CommitmentPhase,
            "reveal_phase" => VoteStatus::RevealPhase,
            "completed" => VoteStatus::Completed,
            "cancelled" => VoteStatus::Cancelled,
            _ => VoteStatus::Created,
        }
    }
}

#[async_trait]
impl VoteStore for SqliteVoteStore {
    async fn create_vote(&self, vote: Vote) -> Result<(), StoreError> {
        debug!("Creating vote: {}", vote.id);
        
        sqlx::query(
            r#"
            INSERT INTO votes (
                id, title, description, template_id, template_params, creator,
                created_at, commitment_start, commitment_end, reveal_start, reveal_end,
                status, results
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&vote.id)
        .bind(&vote.title)
        .bind(&vote.description)
        .bind(&vote.template_id)
        .bind(serde_json::to_string(&vote.template_params)?)
        .bind(&vote.creator)
        .bind(vote.created_at.to_rfc3339())
        .bind(vote.commitment_start.to_rfc3339())
        .bind(vote.commitment_end.to_rfc3339())
        .bind(vote.reveal_start.to_rfc3339())
        .bind(vote.reveal_end.to_rfc3339())
        .bind(Self::vote_status_to_string(&vote.status))
        .bind(vote.results.as_ref().map(|r| serde_json::to_string(r).unwrap_or_default()))
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn get_vote(&self, id: &str) -> Result<Vote, StoreError> {
        debug!("Getting vote: {}", id);
        
        let row = sqlx::query(
            "SELECT * FROM votes WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| StoreError::VoteNotFound { id: id.to_string() })?;
        
        let vote = Vote {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            template_id: row.get("template_id"),
            template_params: serde_json::from_str(&row.get::<String, _>("template_params"))?,
            creator: row.get("creator"),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&chrono::Utc),
            commitment_start: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("commitment_start"))?.with_timezone(&chrono::Utc),
            commitment_end: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("commitment_end"))?.with_timezone(&chrono::Utc),
            reveal_start: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("reveal_start"))?.with_timezone(&chrono::Utc),
            reveal_end: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("reveal_end"))?.with_timezone(&chrono::Utc),
            status: Self::string_to_vote_status(row.get::<String, _>("status").as_str()),
            results: row.get::<Option<String>, _>("results")
                .and_then(|s| if s.is_empty() { None } else { Some(s) })
                .map(|s| serde_json::from_str(&s))
                .transpose()?,
        };
        
        Ok(vote)
    }

    async fn list_votes(&self, query: ListQuery) -> Result<Page<Vote>, StoreError> {
        debug!("Listing votes: page={}, size={}", query.page, query.page_size);
        
        let mut sql = "SELECT * FROM votes WHERE 1=1".to_string();
        
        if let Some(_status) = &query.status {
            sql.push_str(" AND status = ?");
        }
        
        if let Some(_creator) = &query.creator {
            sql.push_str(" AND creator = ?");
        }
        
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        
        let mut query_builder = sqlx::query(&sql);
        
        if let Some(status) = &query.status {
            query_builder = query_builder.bind(Self::vote_status_to_string(status));
        }
        
        if let Some(creator) = &query.creator {
            query_builder = query_builder.bind(creator.clone());
        }
        
        query_builder = query_builder.bind(query.page_size as i64);
        query_builder = query_builder.bind((query.page * query.page_size) as i64);
        
        let rows = query_builder
            .fetch_all(&self.pool)
            .await?;
        
        let mut items = Vec::new();
        for row in rows {
            let vote = Vote {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                template_id: row.get("template_id"),
                template_params: serde_json::from_str(&row.get::<String, _>("template_params"))?,
                creator: row.get("creator"),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&chrono::Utc),
                commitment_start: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("commitment_start"))?.with_timezone(&chrono::Utc),
                commitment_end: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("commitment_end"))?.with_timezone(&chrono::Utc),
                reveal_start: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("reveal_start"))?.with_timezone(&chrono::Utc),
                reveal_end: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("reveal_end"))?.with_timezone(&chrono::Utc),
                status: Self::string_to_vote_status(row.get::<String, _>("status").as_str()),
                results: row.get::<Option<String>, _>("results")
                    .and_then(|s| if s.is_empty() { None } else { Some(s) })
                    .map(|s| serde_json::from_str(&s))
                    .transpose()?,
            };
            items.push(vote);
        }
        
        // Get total count
        let count_row = sqlx::query("SELECT COUNT(*) as count FROM votes")
            .fetch_one(&self.pool)
            .await?;
        let total = count_row.get::<i64, _>("count") as u32;
        let total_pages = total.div_ceil(query.page_size);
        
        Ok(Page {
            items,
            total,
            page: query.page,
            page_size: query.page_size,
            total_pages,
        })
    }

    async fn update_vote_status(&self, id: &str, status: VoteStatus) -> Result<(), StoreError> {
        debug!("Updating vote status: {} -> {:?}", id, status);
        
        sqlx::query("UPDATE votes SET status = ? WHERE id = ?")
            .bind(Self::vote_status_to_string(&status))
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    async fn update_vote_results(&self, id: &str, results: &VoteResults) -> Result<(), StoreError> {
        debug!("Updating vote results: {}", id);
        
        sqlx::query("UPDATE votes SET results = ? WHERE id = ?")
            .bind(serde_json::to_string(results)?)
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    async fn save_commitment(&self, commitment: Commitment) -> Result<(), StoreError> {
        debug!("Saving commitment: {}", commitment.id);
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO commitments (
                id, vote_id, voter, commitment_hash, salt, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&commitment.id)
        .bind(&commitment.vote_id)
        .bind(&commitment.voter)
        .bind(&commitment.commitment_hash)
        .bind(&commitment.salt)
        .bind(commitment.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn get_commitment(&self, vote_id: &str, voter: &str) -> Result<Option<Commitment>, StoreError> {
        debug!("Getting commitment: {}:{}", vote_id, voter);
        
        let row = sqlx::query(
            "SELECT * FROM commitments WHERE vote_id = ? AND voter = ?"
        )
        .bind(vote_id)
        .bind(voter)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let commitment = Commitment {
                id: row.get("id"),
                vote_id: row.get("vote_id"),
                voter: row.get("voter"),
                commitment_hash: row.get("commitment_hash"),
                salt: row.get("salt"),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&chrono::Utc),
            };
            Ok(Some(commitment))
        } else {
            Ok(None)
        }
    }

    async fn list_commitments(&self, vote_id: &str) -> Result<Vec<Commitment>, StoreError> {
        debug!("Listing commitments for vote: {}", vote_id);
        
        let rows = sqlx::query(
            "SELECT * FROM commitments WHERE vote_id = ? ORDER BY created_at"
        )
        .bind(vote_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut commitments = Vec::new();
        for row in rows {
            let commitment = Commitment {
                id: row.get("id"),
                vote_id: row.get("vote_id"),
                voter: row.get("voter"),
                commitment_hash: row.get("commitment_hash"),
                salt: row.get("salt"),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&chrono::Utc),
            };
            commitments.push(commitment);
        }
        
        Ok(commitments)
    }

    async fn save_reveal(&self, reveal: Reveal) -> Result<(), StoreError> {
        debug!("Saving reveal: {}", reveal.id);
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO reveals (
                id, vote_id, voter, value, salt, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&reveal.id)
        .bind(&reveal.vote_id)
        .bind(&reveal.voter)
        .bind(serde_json::to_string(&reveal.value)?)
        .bind(&reveal.salt)
        .bind(reveal.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn list_reveals(&self, vote_id: &str) -> Result<Vec<Reveal>, StoreError> {
        debug!("Listing reveals for vote: {}", vote_id);
        
        let rows = sqlx::query(
            "SELECT * FROM reveals WHERE vote_id = ? ORDER BY created_at"
        )
        .bind(vote_id)
        .fetch_all(&self.pool)
        .await?;
        
        let mut reveals = Vec::new();
        for row in rows {
            let reveal = Reveal {
                id: row.get("id"),
                vote_id: row.get("vote_id"),
                voter: row.get("voter"),
                value: serde_json::from_str(&row.get::<String, _>("value"))?,
                salt: row.get("salt"),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&chrono::Utc),
            };
            reveals.push(reveal);
        }
        
        Ok(reveals)
    }

    async fn get_reveal(&self, vote_id: &str, voter: &str) -> Result<Option<Reveal>, StoreError> {
        debug!("Getting reveal: {}:{}", vote_id, voter);
        
        let row = sqlx::query(
            "SELECT * FROM reveals WHERE vote_id = ? AND voter = ?"
        )
        .bind(vote_id)
        .bind(voter)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let reveal = Reveal {
                id: row.get("id"),
                vote_id: row.get("vote_id"),
                voter: row.get("voter"),
                value: serde_json::from_str(&row.get::<String, _>("value"))?,
                salt: row.get("salt"),
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&chrono::Utc),
            };
            Ok(Some(reveal))
        } else {
            Ok(None)
        }
    }

    async fn delete_vote(&self, id: &str) -> Result<(), StoreError> {
        debug!("Deleting vote: {}", id);
        
        // Delete in order to respect foreign key constraints
        sqlx::query("DELETE FROM reveals WHERE vote_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        sqlx::query("DELETE FROM commitments WHERE vote_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        sqlx::query("DELETE FROM votes WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }

    async fn get_stats(&self) -> Result<StoreStats, StoreError> {
        debug!("Getting storage stats");
        
        let votes_count = sqlx::query("SELECT COUNT(*) as count FROM votes")
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count") as u32;
        
        let commitments_count = sqlx::query("SELECT COUNT(*) as count FROM commitments")
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count") as u32;
        
        let reveals_count = sqlx::query("SELECT COUNT(*) as count FROM reveals")
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count") as u32;
        
        let active_votes = sqlx::query(
            "SELECT COUNT(*) as count FROM votes WHERE status IN ('created', 'commitment_phase', 'reveal_phase')"
        )
        .fetch_one(&self.pool)
        .await?
        .get::<i64, _>("count") as u32;
        
        let completed_votes = sqlx::query("SELECT COUNT(*) as count FROM votes WHERE status = 'completed'")
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count") as u32;
        
        Ok(StoreStats {
            total_votes: votes_count,
            total_commitments: commitments_count,
            total_reveals: reveals_count,
            active_votes,
            completed_votes,
        })
    }
}
