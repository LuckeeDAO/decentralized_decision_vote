//! Event replay system

use crate::{Event, EventType, EventStoreError};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 回放选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayOptions {
    /// 回放速度倍数（1.0 = 正常速度）
    pub speed_multiplier: f64,
    /// 是否跳过错误事件
    pub skip_errors: bool,
    /// 最大回放事件数
    pub max_events: Option<usize>,
    /// 回放过滤器
    pub filter: Option<ReplayFilter>,
    /// 是否实时回放
    pub real_time: bool,
    /// 回放开始时间
    pub start_time: Option<DateTime<Utc>>,
    /// 回放结束时间
    pub end_time: Option<DateTime<Utc>>,
}

impl Default for ReplayOptions {
    fn default() -> Self {
        Self {
            speed_multiplier: 1.0,
            skip_errors: true,
            max_events: None,
            filter: None,
            real_time: false,
            start_time: None,
            end_time: None,
        }
    }
}

/// 回放过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFilter {
    /// 事件类型过滤
    pub event_types: Option<Vec<EventType>>,
    /// 会话ID过滤
    pub session_ids: Option<Vec<String>>,
    /// 用户ID过滤
    pub user_ids: Option<Vec<Uuid>>,
    /// 来源过滤
    pub sources: Option<Vec<String>>,
    /// 最小严重级别
    pub min_severity: Option<crate::EventSeverity>,
}

/// 回放结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    /// 回放的事件数量
    pub events_processed: usize,
    /// 成功处理的事件数量
    pub events_successful: usize,
    /// 失败的事件数量
    pub events_failed: usize,
    /// 跳过的错误事件数量
    pub errors_skipped: usize,
    /// 回放开始时间
    pub start_time: DateTime<Utc>,
    /// 回放结束时间
    pub end_time: DateTime<Utc>,
    /// 总耗时
    pub duration_ms: u64,
    /// 错误列表
    pub errors: Vec<ReplayError>,
}

/// 回放错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayError {
    pub event_id: Uuid,
    pub error_message: String,
    pub timestamp: DateTime<Utc>,
}

/// 事件处理器 trait
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// 处理事件
    async fn handle_event(&self, event: &Event) -> Result<(), String>;
    
    /// 获取处理器名称
    fn get_name(&self) -> &str;
}

/// 事件回放器
pub struct EventReplayer {
    handlers: Vec<Box<dyn EventHandler>>,
    options: ReplayOptions,
}

impl EventReplayer {
    pub fn new(options: ReplayOptions) -> Self {
        Self {
            handlers: Vec::new(),
            options,
        }
    }

    /// 添加事件处理器
    pub fn add_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    /// 设置回放选项
    pub fn set_options(&mut self, options: ReplayOptions) {
        self.options = options;
    }

    /// 回放事件列表
    pub async fn replay_events(&self, events: Vec<Event>) -> Result<ReplayResult, EventStoreError> {
        let start_time = Utc::now();
        let mut result = ReplayResult {
            events_processed: 0,
            events_successful: 0,
            events_failed: 0,
            errors_skipped: 0,
            start_time,
            end_time: start_time,
            duration_ms: 0,
            errors: Vec::new(),
        };

        info!("Starting event replay with {} events", events.len());

        // 过滤事件
        let filtered_events = self.filter_events(events);
        info!("Filtered to {} events for replay", filtered_events.len());

        // 应用时间范围过滤
        let time_filtered_events = self.apply_time_filter(filtered_events);
        info!("Time filtered to {} events for replay", time_filtered_events.len());

        // 应用最大事件数限制
        let events_to_replay = if let Some(max_events) = self.options.max_events {
            time_filtered_events.into_iter().take(max_events).collect()
        } else {
            time_filtered_events
        };

        info!("Replaying {} events", events_to_replay.len());

        // 回放事件
        for event in events_to_replay {
            result.events_processed += 1;

            // 检查是否应该跳过错误事件
            if self.options.skip_errors && self.is_error_event(&event) {
                result.errors_skipped += 1;
                continue;
            }

            // 处理事件
            match self.process_event(&event).await {
                Ok(_) => {
                    result.events_successful += 1;
                }
                Err(error_msg) => {
                    result.events_failed += 1;
                    result.errors.push(ReplayError {
                        event_id: event.id,
                        error_message: error_msg,
                        timestamp: Utc::now(),
                    });
                }
            }

            // 实时回放延迟
            if self.options.real_time && self.options.speed_multiplier > 0.0 {
                let delay = self.calculate_delay(&event);
                if delay > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                }
            }
        }

        result.end_time = Utc::now();
        result.duration_ms = result.end_time.signed_duration_since(result.start_time).num_milliseconds() as u64;

        info!(
            "Event replay completed: {} processed, {} successful, {} failed, {} errors skipped",
            result.events_processed,
            result.events_successful,
            result.events_failed,
            result.errors_skipped
        );

        Ok(result)
    }

    /// 过滤事件
    fn filter_events(&self, events: Vec<Event>) -> Vec<Event> {
        if let Some(ref filter) = self.options.filter {
            events
                .into_iter()
                .filter(|event| self.matches_filter(event, filter))
                .collect()
        } else {
            events
        }
    }

    /// 应用时间过滤器
    fn apply_time_filter(&self, events: Vec<Event>) -> Vec<Event> {
        events
            .into_iter()
            .filter(|event| {
                if let Some(start_time) = self.options.start_time {
                    if event.timestamp < start_time {
                        return false;
                    }
                }
                if let Some(end_time) = self.options.end_time {
                    if event.timestamp > end_time {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// 检查是否匹配过滤器
    fn matches_filter(&self, event: &Event, filter: &ReplayFilter) -> bool {
        // 事件类型过滤
        if let Some(ref event_types) = filter.event_types {
            if !event_types.contains(&event.event_type) {
                return false;
            }
        }

        // 会话ID过滤
        if let Some(ref session_ids) = filter.session_ids {
            if let Some(ref session_id) = event.session_id {
                if !session_ids.contains(session_id) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // 用户ID过滤
        if let Some(ref user_ids) = filter.user_ids {
            if let Some(user_id) = event.user_id {
                if !user_ids.contains(&user_id) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // 来源过滤
        if let Some(ref sources) = filter.sources {
            if !sources.contains(&event.source) {
                return false;
            }
        }

        // 严重级别过滤
        if let Some(ref min_severity) = filter.min_severity {
            if !self.severity_greater_or_equal(&event.severity, min_severity) {
                return false;
            }
        }

        true
    }

    /// 检查严重级别是否大于等于
    fn severity_greater_or_equal(&self, severity: &crate::EventSeverity, min_severity: &crate::EventSeverity) -> bool {
        let severity_level = match severity {
            crate::EventSeverity::Debug => 0,
            crate::EventSeverity::Info => 1,
            crate::EventSeverity::Warning => 2,
            crate::EventSeverity::Error => 3,
            crate::EventSeverity::Critical => 4,
        };

        let min_level = match min_severity {
            crate::EventSeverity::Debug => 0,
            crate::EventSeverity::Info => 1,
            crate::EventSeverity::Warning => 2,
            crate::EventSeverity::Error => 3,
            crate::EventSeverity::Critical => 4,
        };

        severity_level >= min_level
    }

    /// 检查是否为错误事件
    fn is_error_event(&self, event: &Event) -> bool {
        matches!(event.severity, crate::EventSeverity::Error | crate::EventSeverity::Critical)
    }

    /// 处理事件
    async fn process_event(&self, event: &Event) -> Result<(), String> {
        for handler in &self.handlers {
            if let Err(e) = handler.handle_event(event).await {
                return Err(format!("Handler {} failed: {}", handler.get_name(), e));
            }
        }
        Ok(())
    }

    /// 计算延迟时间
    fn calculate_delay(&self, _event: &Event) -> u64 {
        // 简化实现，实际应用中应该根据事件间的时间间隔计算
        if self.options.speed_multiplier > 0.0 {
            (1000.0 / self.options.speed_multiplier) as u64
        } else {
            0
        }
    }
}

/// 日志事件处理器
pub struct LogEventHandler {
    name: String,
}

impl LogEventHandler {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait::async_trait]
impl EventHandler for LogEventHandler {
    async fn handle_event(&self, event: &Event) -> Result<(), String> {
        info!(
            "Replaying event: {} - {} - {} - {}",
            event.id,
            event.event_type,
            event.severity,
            event.message
        );
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

/// 统计事件处理器
pub struct StatisticsEventHandler {
    name: String,
    stats: std::sync::Arc<std::sync::Mutex<ReplayStatistics>>,
}

#[derive(Debug, Default)]
pub struct ReplayStatistics {
    total_events: usize,
    events_by_type: HashMap<String, usize>,
    events_by_severity: HashMap<String, usize>,
    events_by_source: HashMap<String, usize>,
}

impl Clone for ReplayStatistics {
    fn clone(&self) -> Self {
        Self {
            total_events: self.total_events,
            events_by_type: self.events_by_type.clone(),
            events_by_severity: self.events_by_severity.clone(),
            events_by_source: self.events_by_source.clone(),
        }
    }
}

impl StatisticsEventHandler {
    pub fn new(name: String) -> Self {
        Self {
            name,
            stats: std::sync::Arc::new(std::sync::Mutex::new(ReplayStatistics::default())),
        }
    }

    pub fn get_statistics(&self) -> ReplayStatistics {
        self.stats.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl EventHandler for StatisticsEventHandler {
    async fn handle_event(&self, event: &Event) -> Result<(), String> {
        let mut stats = self.stats.lock().unwrap();
        stats.total_events += 1;
        
        // 按类型统计
        let event_type_str = format!("{:?}", event.event_type);
        *stats.events_by_type.entry(event_type_str).or_insert(0) += 1;
        
        // 按严重级别统计
        let severity_str = format!("{:?}", event.severity);
        *stats.events_by_severity.entry(severity_str).or_insert(0) += 1;
        
        // 按来源统计
        *stats.events_by_source.entry(event.source.clone()).or_insert(0) += 1;
        
        Ok(())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

