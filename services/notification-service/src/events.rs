//! Event handling for notification service

use crate::{NotificationType, NotificationMessage, EventSubscriber};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tracing::{info, warn, error};
use uuid::Uuid;

/// 通知事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub id: Uuid,
    pub event_type: NotificationType,
    pub session_id: Option<String>,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

impl NotificationEvent {
    pub fn new(
        event_type: NotificationType,
        session_id: Option<String>,
        data: HashMap<String, serde_json::Value>,
        source: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            session_id,
            data,
            timestamp: chrono::Utc::now(),
            source,
        }
    }

    pub fn with_data(mut self, key: String, value: serde_json::Value) -> Self {
        self.data.insert(key, value);
        self
    }
}

/// 事件处理器
pub struct EventHandler {
    subscribers: HashMap<Uuid, EventSubscriber>,
    event_sender: broadcast::Sender<NotificationEvent>,
    #[allow(dead_code)]
    event_receiver: broadcast::Receiver<NotificationEvent>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = broadcast::channel(1000);
        Self {
            subscribers: HashMap::new(),
            event_sender: sender,
            event_receiver: receiver,
        }
    }

    /// 订阅事件
    pub fn subscribe(&mut self, subscriber: EventSubscriber) -> Result<Uuid> {
        let id = subscriber.id;
        info!("Subscribing event subscriber: {} with ID: {}", subscriber.name, id);
        self.subscribers.insert(id, subscriber);
        Ok(id)
    }

    /// 取消订阅
    pub fn unsubscribe(&mut self, subscriber_id: Uuid) -> Result<()> {
        if let Some(subscriber) = self.subscribers.remove(&subscriber_id) {
            info!("Unsubscribed event subscriber: {} with ID: {}", subscriber.name, subscriber_id);
        } else {
            warn!("Attempted to unsubscribe non-existent subscriber: {}", subscriber_id);
        }
        Ok(())
    }

    /// 发布事件
    pub fn publish_event(&self, event: NotificationEvent) -> Result<()> {
        info!("Publishing event: {:?} with ID: {}", event.event_type, event.id);
        
        // 发送到广播通道
        if let Err(e) = self.event_sender.send(event.clone()) {
            error!("Failed to send event to broadcast channel: {}", e);
            return Err(e.into());
        }

        // 通知相关订阅者
        self.notify_subscribers(&event)?;
        
        Ok(())
    }

    /// 通知订阅者
    fn notify_subscribers(&self, event: &NotificationEvent) -> Result<()> {
        let mut notified_count = 0;
        
        for (subscriber_id, subscriber) in &self.subscribers {
            if !subscriber.active {
                continue;
            }

            // 检查是否订阅了此事件类型
            if !subscriber.event_types.contains(&event.event_type) {
                continue;
            }

            // 应用过滤器
            if !self.apply_filters(subscriber, event) {
                continue;
            }

            // 创建通知消息
            let _message = self.create_notification_message(subscriber, event)?;
            
            // 这里应该发送到通知队列，暂时只记录日志
            info!(
                "Notifying subscriber {} (ID: {}) about event {} (ID: {})",
                subscriber.name, subscriber_id, event.event_type, event.id
            );
            
            notified_count += 1;
        }

        info!("Notified {} subscribers about event {}", notified_count, event.id);
        Ok(())
    }

    /// 应用过滤器
    fn apply_filters(&self, subscriber: &EventSubscriber, event: &NotificationEvent) -> bool {
        for (key, expected_value) in &subscriber.filters {
            if let Some(actual_value) = event.data.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// 创建通知消息
    fn create_notification_message(
        &self,
        subscriber: &EventSubscriber,
        event: &NotificationEvent,
    ) -> Result<NotificationMessage> {
        let (title, content) = self.generate_message_content(event);
        
        let message = NotificationMessage::new(
            event.event_type.clone(),
            self.get_priority_for_event(&event.event_type),
            title,
            content,
            subscriber.name.clone(),
        ).with_metadata("event_id".to_string(), serde_json::Value::String(event.id.to_string()))
         .with_metadata("session_id".to_string(), 
                       event.session_id.as_ref()
                           .map(|s| serde_json::Value::String(s.clone()))
                           .unwrap_or(serde_json::Value::Null));

        Ok(message)
    }

    /// 生成消息内容
    fn generate_message_content(&self, event: &NotificationEvent) -> (String, String) {
        match &event.event_type {
            NotificationType::SessionCreated => (
                "新会话已创建".to_string(),
                format!("会话 {} 已成功创建", 
                       event.session_id.as_deref().unwrap_or("未知"))
            ),
            NotificationType::CommitmentSubmitted => (
                "承诺已提交".to_string(),
                format!("会话 {} 中有新的承诺提交", 
                       event.session_id.as_deref().unwrap_or("未知"))
            ),
            NotificationType::RevealPhaseStarted => (
                "揭示阶段开始".to_string(),
                format!("会话 {} 的揭示阶段已开始", 
                       event.session_id.as_deref().unwrap_or("未知"))
            ),
            NotificationType::RevealCompleted => (
                "揭示阶段完成".to_string(),
                format!("会话 {} 的揭示阶段已完成", 
                       event.session_id.as_deref().unwrap_or("未知"))
            ),
            NotificationType::ResultGenerated => (
                "结果已生成".to_string(),
                format!("会话 {} 的结果已生成", 
                       event.session_id.as_deref().unwrap_or("未知"))
            ),
            NotificationType::SystemError => (
                "系统错误".to_string(),
                format!("系统发生错误: {}", 
                       event.data.get("error").and_then(|v| v.as_str()).unwrap_or("未知错误"))
            ),
            NotificationType::Custom(custom_type) => (
                format!("自定义通知: {}", custom_type),
                format!("收到自定义事件: {}", custom_type)
            ),
        }
    }

    /// 获取事件优先级
    fn get_priority_for_event(&self, event_type: &NotificationType) -> crate::NotificationPriority {
        match event_type {
            NotificationType::SystemError => crate::NotificationPriority::Critical,
            NotificationType::ResultGenerated => crate::NotificationPriority::High,
            NotificationType::RevealPhaseStarted | NotificationType::RevealCompleted => {
                crate::NotificationPriority::High
            },
            NotificationType::CommitmentSubmitted => crate::NotificationPriority::Normal,
            NotificationType::SessionCreated => crate::NotificationPriority::Low,
            NotificationType::Custom(_) => crate::NotificationPriority::Normal,
        }
    }

    /// 获取事件接收器
    pub fn get_event_receiver(&self) -> broadcast::Receiver<NotificationEvent> {
        self.event_sender.subscribe()
    }

    /// 获取订阅者列表
    pub fn get_subscribers(&self) -> Vec<&EventSubscriber> {
        self.subscribers.values().collect()
    }

    /// 获取活跃订阅者数量
    pub fn get_active_subscriber_count(&self) -> usize {
        self.subscribers.values().filter(|s| s.active).count()
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventHandler {
    fn clone(&self) -> Self {
        let (sender, receiver) = broadcast::channel(1000);
        Self {
            subscribers: self.subscribers.clone(),
            event_sender: sender,
            event_receiver: receiver,
        }
    }
}
