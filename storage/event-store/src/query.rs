//! Event query system

use crate::{Event, EventType, EventSeverity, EventStoreError};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 查询条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryCondition {
    /// 等于
    Equals(serde_json::Value),
    /// 不等于
    NotEquals(serde_json::Value),
    /// 大于
    GreaterThan(serde_json::Value),
    /// 大于等于
    GreaterThanOrEqual(serde_json::Value),
    /// 小于
    LessThan(serde_json::Value),
    /// 小于等于
    LessThanOrEqual(serde_json::Value),
    /// 包含
    Contains(String),
    /// 正则匹配
    Regex(String),
    /// 在列表中
    In(Vec<serde_json::Value>),
    /// 不在列表中
    NotIn(Vec<serde_json::Value>),
    /// 存在
    Exists,
    /// 不存在
    NotExists,
}

/// 查询字段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum QueryField {
    /// 事件ID
    Id,
    /// 事件类型
    EventType,
    /// 严重级别
    Severity,
    /// 会话ID
    SessionId,
    /// 用户ID
    UserId,
    /// 来源
    Source,
    /// 消息
    Message,
    /// 时间戳
    Timestamp,
    /// 关联ID
    CorrelationId,
    /// 因果ID
    CausationId,
    /// 版本
    Version,
    /// 数据字段
    Data(String),
}

/// 查询操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryOperator {
    /// 与
    And,
    /// 或
    Or,
    /// 非
    Not,
}

/// 查询表达式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryExpression {
    /// 条件
    Condition(QueryField, QueryCondition),
    /// 复合表达式
    Composite(QueryOperator, Vec<QueryExpression>),
}

/// 排序字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortField {
    /// 时间戳
    Timestamp,
    /// 事件类型
    EventType,
    /// 严重级别
    Severity,
    /// 来源
    Source,
    /// 版本
    Version,
}

/// 排序方向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    /// 升序
    Ascending,
    /// 降序
    Descending,
}

/// 排序规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortRule {
    pub field: SortField,
    pub direction: SortDirection,
}

/// 分页参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub offset: usize,
    pub limit: usize,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 100,
        }
    }
}

/// 事件查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventQuery {
    pub expression: Option<QueryExpression>,
    pub sort_rules: Vec<SortRule>,
    pub pagination: PaginationParams,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl EventQuery {
    pub fn new() -> Self {
        Self {
            expression: None,
            sort_rules: vec![SortRule {
                field: SortField::Timestamp,
                direction: SortDirection::Descending,
            }],
            pagination: PaginationParams::default(),
            time_range: None,
        }
    }

    pub fn with_expression(mut self, expression: QueryExpression) -> Self {
        self.expression = Some(expression);
        self
    }

    pub fn with_sort(mut self, field: SortField, direction: SortDirection) -> Self {
        self.sort_rules = vec![SortRule { field, direction }];
        self
    }

    pub fn with_pagination(mut self, offset: usize, limit: usize) -> Self {
        self.pagination = PaginationParams { offset, limit };
        self
    }

    pub fn with_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.time_range = Some((start, end));
        self
    }
}

impl Default for EventQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub events: Vec<Event>,
    pub total_count: usize,
    pub has_more: bool,
    pub execution_time_ms: u64,
}

/// 查询构建器
pub struct QueryBuilder {
    query: EventQuery,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            query: EventQuery::new(),
        }
    }

    /// 添加条件
    pub fn where_field(mut self, field: QueryField, condition: QueryCondition) -> Self {
        let new_condition = QueryExpression::Condition(field, condition);
        
        self.query.expression = match self.query.expression {
            Some(existing) => Some(QueryExpression::Composite(
                QueryOperator::And,
                vec![existing, new_condition],
            )),
            None => Some(new_condition),
        };
        
        self
    }

    /// 事件类型等于
    pub fn event_type_equals(mut self, event_type: EventType) -> Self {
        self = self.where_field(
            QueryField::EventType,
            QueryCondition::Equals(serde_json::to_value(event_type).unwrap()),
        );
        self
    }

    /// 严重级别等于
    pub fn severity_equals(mut self, severity: EventSeverity) -> Self {
        self = self.where_field(
            QueryField::Severity,
            QueryCondition::Equals(serde_json::to_value(severity).unwrap()),
        );
        self
    }

    /// 会话ID等于
    pub fn session_id_equals(mut self, session_id: String) -> Self {
        self = self.where_field(
            QueryField::SessionId,
            QueryCondition::Equals(serde_json::Value::String(session_id)),
        );
        self
    }

    /// 用户ID等于
    pub fn user_id_equals(mut self, user_id: Uuid) -> Self {
        self = self.where_field(
            QueryField::UserId,
            QueryCondition::Equals(serde_json::to_value(user_id).unwrap()),
        );
        self
    }

    /// 来源包含
    pub fn source_contains(mut self, source: String) -> Self {
        self = self.where_field(
            QueryField::Source,
            QueryCondition::Contains(source),
        );
        self
    }

    /// 消息包含
    pub fn message_contains(mut self, message: String) -> Self {
        self = self.where_field(
            QueryField::Message,
            QueryCondition::Contains(message),
        );
        self
    }

    /// 时间范围
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.query.time_range = Some((start, end));
        self
    }

    /// 排序
    pub fn order_by(mut self, field: SortField, direction: SortDirection) -> Self {
        self.query.sort_rules = vec![SortRule { field, direction }];
        self
    }

    /// 分页
    pub fn paginate(mut self, offset: usize, limit: usize) -> Self {
        self.query.pagination = PaginationParams { offset, limit };
        self
    }

    /// 构建查询
    pub fn build(self) -> EventQuery {
        self.query
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 查询执行器
pub struct QueryExecutor;

impl QueryExecutor {
    /// 执行查询
    pub fn execute(query: &EventQuery, events: &[Event]) -> Result<QueryResult, EventStoreError> {
        let start_time = std::time::Instant::now();
        
        // 应用时间范围过滤
        let mut filtered_events = if let Some((start, end)) = query.time_range {
            events
                .iter()
                .filter(|event| event.timestamp >= start && event.timestamp <= end)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            events.to_vec()
        };
        
        // 应用表达式过滤
        if let Some(ref expression) = query.expression {
            filtered_events = Self::apply_expression(expression, &filtered_events)?;
        }
        
        // 应用排序
        Self::apply_sorting(&mut filtered_events, &query.sort_rules);
        
        // 应用分页
        let total_count = filtered_events.len();
        let has_more = query.pagination.offset + query.pagination.limit < total_count;
        let paginated_events = filtered_events
            .into_iter()
            .skip(query.pagination.offset)
            .take(query.pagination.limit)
            .collect();
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(QueryResult {
            events: paginated_events,
            total_count,
            has_more,
            execution_time_ms: execution_time,
        })
    }

    /// 应用查询表达式
    fn apply_expression(
        expression: &QueryExpression,
        events: &[Event],
    ) -> Result<Vec<Event>, EventStoreError> {
        match expression {
            QueryExpression::Condition(field, condition) => {
                Ok(events
                    .iter()
                    .filter(|event| Self::evaluate_condition(event, field, condition))
                    .cloned()
                    .collect())
            }
            QueryExpression::Composite(operator, expressions) => {
                match operator {
                    QueryOperator::And => {
                        let mut result = events.to_vec();
                        for expr in expressions {
                            result = Self::apply_expression(expr, &result)?;
                        }
                        Ok(result)
                    }
                    QueryOperator::Or => {
                        let mut result = Vec::new();
                        for expr in expressions {
                            let expr_result = Self::apply_expression(expr, events)?;
                            for event in expr_result {
                                if !result.iter().any(|e: &Event| e.id == event.id) {
                                    result.push(event);
                                }
                            }
                        }
                        Ok(result)
                    }
                    QueryOperator::Not => {
                        if expressions.len() != 1 {
                            return Err(EventStoreError::Query("NOT operator requires exactly one expression".to_string()));
                        }
                        let excluded = Self::apply_expression(&expressions[0], events)?;
                        let excluded_ids: std::collections::HashSet<Uuid> = excluded.iter().map(|e| e.id).collect();
                        Ok(events
                            .iter()
                            .filter(|event| !excluded_ids.contains(&event.id))
                            .cloned()
                            .collect())
                    }
                }
            }
        }
    }

    /// 评估查询条件
    fn evaluate_condition(event: &Event, field: &QueryField, condition: &QueryCondition) -> bool {
        let value = Self::get_field_value(event, field);
        
        match condition {
            QueryCondition::Equals(expected) => &value == expected,
            QueryCondition::NotEquals(expected) => &value != expected,
            QueryCondition::GreaterThan(expected) => {
                if let (Some(v), Some(e)) = (value.as_str(), expected.as_str()) {
                    v > e
                } else if let (Some(v), Some(e)) = (value.as_f64(), expected.as_f64()) {
                    v > e
                } else {
                    false
                }
            }
            QueryCondition::GreaterThanOrEqual(expected) => {
                if let (Some(v), Some(e)) = (value.as_str(), expected.as_str()) {
                    v >= e
                } else if let (Some(v), Some(e)) = (value.as_f64(), expected.as_f64()) {
                    v >= e
                } else {
                    false
                }
            }
            QueryCondition::LessThan(expected) => {
                if let (Some(v), Some(e)) = (value.as_str(), expected.as_str()) {
                    v < e
                } else if let (Some(v), Some(e)) = (value.as_f64(), expected.as_f64()) {
                    v < e
                } else {
                    false
                }
            }
            QueryCondition::LessThanOrEqual(expected) => {
                if let (Some(v), Some(e)) = (value.as_str(), expected.as_str()) {
                    v <= e
                } else if let (Some(v), Some(e)) = (value.as_f64(), expected.as_f64()) {
                    v <= e
                } else {
                    false
                }
            }
            QueryCondition::Contains(substring) => {
                if let Some(v) = value.as_str() {
                    v.contains(substring)
                } else {
                    false
                }
            }
            QueryCondition::Regex(pattern) => {
                if let Some(v) = value.as_str() {
                    // 简化实现，实际应用中应该使用正则表达式库
                    v.contains(pattern)
                } else {
                    false
                }
            }
            QueryCondition::In(values) => values.contains(&value),
            QueryCondition::NotIn(values) => !values.contains(&value),
            QueryCondition::Exists => !value.is_null(),
            QueryCondition::NotExists => value.is_null(),
        }
    }

    /// 获取字段值
    fn get_field_value(event: &Event, field: &QueryField) -> serde_json::Value {
        match field {
            QueryField::Id => serde_json::to_value(event.id).unwrap(),
            QueryField::EventType => serde_json::to_value(&event.event_type).unwrap(),
            QueryField::Severity => serde_json::to_value(&event.severity).unwrap(),
            QueryField::SessionId => event.session_id.as_ref()
                .map(|s| serde_json::Value::String(s.clone()))
                .unwrap_or(serde_json::Value::Null),
            QueryField::UserId => event.user_id
                .map(|id| serde_json::to_value(id).unwrap())
                .unwrap_or(serde_json::Value::Null),
            QueryField::Source => serde_json::Value::String(event.source.clone()),
            QueryField::Message => serde_json::Value::String(event.message.clone()),
            QueryField::Timestamp => serde_json::to_value(event.timestamp).unwrap(),
            QueryField::CorrelationId => event.correlation_id
                .map(|id| serde_json::to_value(id).unwrap())
                .unwrap_or(serde_json::Value::Null),
            QueryField::CausationId => event.causation_id
                .map(|id| serde_json::to_value(id).unwrap())
                .unwrap_or(serde_json::Value::Null),
            QueryField::Version => serde_json::Value::Number(event.version.into()),
            QueryField::Data(key) => event.data.get(key)
                .cloned()
                .unwrap_or(serde_json::Value::Null),
        }
    }

    /// 应用排序
    fn apply_sorting(events: &mut Vec<Event>, sort_rules: &[SortRule]) {
        events.sort_by(|a, b| {
            for rule in sort_rules {
                let comparison = match rule.field {
                    SortField::Timestamp => a.timestamp.cmp(&b.timestamp),
                    SortField::EventType => {
                        serde_json::to_value(&a.event_type).unwrap()
                            .as_str()
                            .unwrap_or("")
                            .cmp(serde_json::to_value(&b.event_type).unwrap().as_str().unwrap_or(""))
                    }
                    SortField::Severity => {
                        serde_json::to_value(&a.severity).unwrap()
                            .as_str()
                            .unwrap_or("")
                            .cmp(serde_json::to_value(&b.severity).unwrap().as_str().unwrap_or(""))
                    }
                    SortField::Source => a.source.cmp(&b.source),
                    SortField::Version => a.version.cmp(&b.version),
                };
                
                if comparison != std::cmp::Ordering::Equal {
                    return match rule.direction {
                        SortDirection::Ascending => comparison,
                        SortDirection::Descending => comparison.reverse(),
                    };
                }
            }
            std::cmp::Ordering::Equal
        });
    }
}
