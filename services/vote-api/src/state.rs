use std::sync::Arc;
use shared_config::AppConfig;
use vote_engine::{VoteEngine, services::MemoryVoteService};
use template_system::DefaultTemplateRegistry;
use commitment_engine::{CommitmentEngine, algorithms::Sha256CommitmentAlgorithm};
use vote_store::{VoteStore, MemoryVoteStore, SqliteVoteStore, PostgresVoteStore};
use tracing::info;

/// Application state containing all services and configuration
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub vote_engine: Arc<VoteEngine>,
    pub template_registry: Arc<DefaultTemplateRegistry>,
    #[allow(dead_code)]
    pub commitment_engine: Arc<CommitmentEngine>,
    #[allow(dead_code)]
    pub vote_store: Arc<dyn VoteStore>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing application state");
        
        // Initialize vote store based on configuration
        let vote_store: Arc<dyn VoteStore> = if config.database.url.starts_with("sqlite:") {
            info!("Using SQLite vote store");
            Arc::new(SqliteVoteStore::new(&config.database).await?)
        } else if config.database.url.starts_with("postgresql:") || config.database.url.starts_with("postgres:") {
            info!("Using PostgreSQL vote store");
            Arc::new(PostgresVoteStore::new(&config.database).await?)
        } else {
            info!("Using in-memory vote store");
            Arc::new(MemoryVoteStore::new())
        };
        
        // Initialize template registry
        let template_registry = Arc::new(DefaultTemplateRegistry::new());
        
        // Initialize commitment engine
        let commitment_algorithm = Arc::new(Sha256CommitmentAlgorithm::new());
        let commitment_engine = Arc::new(CommitmentEngine::new(commitment_algorithm));
        
        // Initialize vote engine with memory service (for now)
        // TODO: Replace with proper vote service that uses the vote store
        let vote_service = Arc::new(MemoryVoteService::new());
        let vote_engine = Arc::new(VoteEngine::new(vote_service));
        
        Ok(Self {
            config,
            vote_engine,
            template_registry,
            commitment_engine,
            vote_store,
        })
    }
}
