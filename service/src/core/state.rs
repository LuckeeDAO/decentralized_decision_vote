use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use tokio::sync::Mutex;
use chrono::Utc;
use crate::core::template::{TemplateRegistry, BitTemplate, OptionIndexTemplate, StringTemplate};
use crate::config::Config;
use crate::store::{VoteStore, memory::MemoryVoteStore};
use crate::service::{VoteService, VoteServiceImpl};

pub struct AppState {
    pub current_height: Arc<AtomicU64>,
    pub votes_count: Mutex<usize>,
    pub registry: Arc<TemplateRegistry>,
    pub started_at: std::time::Instant,
    pub store: Arc<dyn VoteStore>,
    pub service: Arc<dyn VoteService>,
}

impl AppState {
    pub async fn new() -> Arc<Self> {
        let _cfg = Config::load_from_env_or_default().unwrap_or_else(|e| {
            tracing::warn!("config load failed: {} - using defaults", e);
            Config { server: crate::config::ServerConfig { host: "0.0.0.0".into(), port: 8080 }, api: crate::config::ApiAuth { enabled: false, tokens: vec![] } }
        });
        let mut reg = TemplateRegistry::new();
        reg.register(BitTemplate);
        reg.register(OptionIndexTemplate);
        reg.register(StringTemplate);
        let store: Arc<dyn VoteStore> = Arc::new(MemoryVoteStore::default());
        let registry = Arc::new(reg);
        let service: Arc<dyn VoteService> = Arc::new(VoteServiceImpl::new(store.clone(), registry.clone()));
        let state = Arc::new(Self {
            current_height: Arc::new(AtomicU64::new(0)),
            votes_count: Mutex::new(0),
            registry,
            started_at: std::time::Instant::now(),
            store,
            service,
        });
        // background height ticker
        tokio::spawn({
            let st = state.clone();
            async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                loop {
                    interval.tick().await;
                    let h = st.current_height.load(Ordering::Relaxed);
                    st.current_height.store(h.saturating_add(1), Ordering::Relaxed);
                }
            }
        });
        state
    }

    pub async fn get_status_json(&self) -> serde_json::Value {
        let h = self.current_height.load(Ordering::Relaxed);
        let v = *self.votes_count.lock().await;
        let uptime_secs = self.started_at.elapsed().as_secs();
        serde_json::json!({
            "status": "running",
            "current_height": h,
            "active_votes": v,
            "timestamp": Utc::now().to_rfc3339(),
            "uptime_secs": uptime_secs,
        })
    }
}
