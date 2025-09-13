use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use tracing::warn;

use crate::registry::TemplateError;

/// Trait for vote value templates
#[async_trait]
pub trait VoteTemplate: Send + Sync + std::fmt::Debug {
    /// Get the template ID
    fn id(&self) -> &'static str;
    
    /// Get the template name
    fn name(&self) -> &'static str;
    
    /// Get the template description
    fn description(&self) -> &'static str;
    
    /// Validate a vote value against the template
    async fn validate(&self, value: &Value, params: &Value) -> Result<(), TemplateError>;
    
    /// Canonicalize a vote value for commitment
    async fn canonicalize(&self, value: &Value, params: &Value) -> Result<Vec<u8>, TemplateError>;
    
    /// Aggregate multiple vote values
    async fn aggregate(&self, values: &[Value], params: &Value) -> Result<Value, TemplateError>;
    
    /// Get the expected value schema
    fn get_schema(&self) -> Value;
}

/// Yes/No voting template
#[derive(Debug)]
pub struct YesNoTemplate;

impl Default for YesNoTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl YesNoTemplate {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl VoteTemplate for YesNoTemplate {
    fn id(&self) -> &'static str {
        "yes_no"
    }
    
    fn name(&self) -> &'static str {
        "Yes/No Vote"
    }
    
    fn description(&self) -> &'static str {
        "Simple yes or no voting"
    }
    
    async fn validate(&self, value: &Value, _params: &Value) -> Result<(), TemplateError> {
        match value.as_bool() {
            Some(_) => Ok(()),
            None => Err(TemplateError::ValidationFailed {
                message: "Value must be a boolean (true/false)".to_string(),
            }),
        }
    }
    
    async fn canonicalize(&self, value: &Value, _params: &Value) -> Result<Vec<u8>, TemplateError> {
        match value.as_bool() {
            Some(b) => Ok(if b { b"yes".to_vec() } else { b"no".to_vec() }),
            None => Err(TemplateError::CanonicalizationFailed {
                message: "Value must be a boolean".to_string(),
            }),
        }
    }
    
    async fn aggregate(&self, values: &[Value], _params: &Value) -> Result<Value, TemplateError> {
        let mut yes_count = 0;
        let mut no_count = 0;
        
        for value in values {
            match value.as_bool() {
                Some(true) => yes_count += 1,
                Some(false) => no_count += 1,
                None => {
                    warn!("Invalid value in yes/no aggregation: {:?}", value);
                }
            }
        }
        
        let mut result = HashMap::new();
        result.insert("yes".to_string(), Value::Number(yes_count.into()));
        result.insert("no".to_string(), Value::Number(no_count.into()));
        result.insert("total".to_string(), Value::Number((yes_count + no_count).into()));
        
        Ok(Value::Object(result.into_iter().collect()))
    }
    
    fn get_schema(&self) -> Value {
        serde_json::json!({
            "type": "boolean",
            "description": "true for yes, false for no"
        })
    }
}

/// Multiple choice voting template
#[derive(Debug)]
pub struct MultipleChoiceTemplate;

impl Default for MultipleChoiceTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl MultipleChoiceTemplate {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl VoteTemplate for MultipleChoiceTemplate {
    fn id(&self) -> &'static str {
        "multiple_choice"
    }
    
    fn name(&self) -> &'static str {
        "Multiple Choice"
    }
    
    fn description(&self) -> &'static str {
        "Choose one option from multiple choices"
    }
    
    async fn validate(&self, value: &Value, params: &Value) -> Result<(), TemplateError> {
        let choices = params.get("choices")
            .and_then(|c| c.as_array())
            .ok_or_else(|| TemplateError::ValidationFailed {
                message: "Template params must contain 'choices' array".to_string(),
            })?;
        
        let selected = value.as_str()
            .ok_or_else(|| TemplateError::ValidationFailed {
                message: "Value must be a string".to_string(),
            })?;
        
        if !choices.iter().any(|choice| choice.as_str() == Some(selected)) {
            return Err(TemplateError::ValidationFailed {
                message: format!("Invalid choice: {}", selected),
            });
        }
        
        Ok(())
    }
    
    async fn canonicalize(&self, value: &Value, _params: &Value) -> Result<Vec<u8>, TemplateError> {
        match value.as_str() {
            Some(s) => Ok(s.as_bytes().to_vec()),
            None => Err(TemplateError::CanonicalizationFailed {
                message: "Value must be a string".to_string(),
            }),
        }
    }
    
    async fn aggregate(&self, values: &[Value], params: &Value) -> Result<Value, TemplateError> {
        let choices = params.get("choices")
            .and_then(|c| c.as_array())
            .ok_or_else(|| TemplateError::AggregationFailed {
                message: "Template params must contain 'choices' array".to_string(),
            })?;
        
        let mut counts = HashMap::new();
        let mut total = 0;
        
        // Initialize counts for all choices
        for choice in choices {
            if let Some(choice_str) = choice.as_str() {
                counts.insert(choice_str.to_string(), 0);
            }
        }
        
        // Count votes
        for value in values {
            if let Some(choice) = value.as_str() {
                *counts.entry(choice.to_string()).or_insert(0) += 1;
                total += 1;
            }
        }
        
        let mut result = HashMap::new();
        result.insert("total".to_string(), Value::Number(total.into()));
        result.insert("results".to_string(), Value::Object(
            counts.into_iter().map(|(k, v)| (k, Value::Number(v.into()))).collect()
        ));
        
        Ok(Value::Object(result.into_iter().collect()))
    }
    
    fn get_schema(&self) -> Value {
        serde_json::json!({
            "type": "string",
            "description": "One of the available choices"
        })
    }
}

/// Numeric range voting template
#[derive(Debug)]
pub struct NumericRangeTemplate;

impl Default for NumericRangeTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl NumericRangeTemplate {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl VoteTemplate for NumericRangeTemplate {
    fn id(&self) -> &'static str {
        "numeric_range"
    }
    
    fn name(&self) -> &'static str {
        "Numeric Range"
    }
    
    fn description(&self) -> &'static str {
        "Vote with a numeric value within a specified range"
    }
    
    async fn validate(&self, value: &Value, params: &Value) -> Result<(), TemplateError> {
        let num = value.as_f64()
            .ok_or_else(|| TemplateError::ValidationFailed {
                message: "Value must be a number".to_string(),
            })?;
        
        let min = params.get("min")
            .and_then(|m| m.as_f64())
            .unwrap_or(f64::NEG_INFINITY);
        
        let max = params.get("max")
            .and_then(|m| m.as_f64())
            .unwrap_or(f64::INFINITY);
        
        if num < min || num > max {
            return Err(TemplateError::ValidationFailed {
                message: format!("Value {} is outside range [{}, {}]", num, min, max),
            });
        }
        
        Ok(())
    }
    
    async fn canonicalize(&self, value: &Value, _params: &Value) -> Result<Vec<u8>, TemplateError> {
        match value.as_f64() {
            Some(n) => Ok(n.to_string().as_bytes().to_vec()),
            None => Err(TemplateError::CanonicalizationFailed {
                message: "Value must be a number".to_string(),
            }),
        }
    }
    
    async fn aggregate(&self, values: &[Value], _params: &Value) -> Result<Value, TemplateError> {
        let mut sum = 0.0;
        let mut count = 0;
        let mut min_val = f64::INFINITY;
        let mut max_val = f64::NEG_INFINITY;
        
        for value in values {
            if let Some(num) = value.as_f64() {
                sum += num;
                count += 1;
                min_val = min_val.min(num);
                max_val = max_val.max(num);
            }
        }
        
        let mut result = HashMap::new();
        result.insert("count".to_string(), Value::Number(count.into()));
        result.insert("sum".to_string(), Value::Number(serde_json::Number::from_f64(sum).unwrap_or(0.into())));
        result.insert("average".to_string(), Value::Number(serde_json::Number::from_f64(if count > 0 { sum / count as f64 } else { 0.0 }).unwrap_or(0.into())));
        result.insert("min".to_string(), Value::Number(serde_json::Number::from_f64(min_val).unwrap_or(0.into())));
        result.insert("max".to_string(), Value::Number(serde_json::Number::from_f64(max_val).unwrap_or(0.into())));
        
        Ok(Value::Object(result.into_iter().collect()))
    }
    
    fn get_schema(&self) -> Value {
        serde_json::json!({
            "type": "number",
            "description": "A numeric value within the specified range"
        })
    }
}

/// Ranking voting template
#[derive(Debug)]
pub struct RankingTemplate;

impl Default for RankingTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl RankingTemplate {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl VoteTemplate for RankingTemplate {
    fn id(&self) -> &'static str {
        "ranking"
    }
    
    fn name(&self) -> &'static str {
        "Ranking"
    }
    
    fn description(&self) -> &'static str {
        "Rank options in order of preference"
    }
    
    async fn validate(&self, value: &Value, params: &Value) -> Result<(), TemplateError> {
        let ranking = value.as_array()
            .ok_or_else(|| TemplateError::ValidationFailed {
                message: "Value must be an array".to_string(),
            })?;
        
        let options = params.get("options")
            .and_then(|o| o.as_array())
            .ok_or_else(|| TemplateError::ValidationFailed {
                message: "Template params must contain 'options' array".to_string(),
            })?;
        
        // Check that all options are ranked exactly once
        if ranking.len() != options.len() {
            return Err(TemplateError::ValidationFailed {
                message: "Ranking must include all options exactly once".to_string(),
            });
        }
        
        for item in ranking {
            let item_str = item.as_str()
                .ok_or_else(|| TemplateError::ValidationFailed {
                    message: "Ranking items must be strings".to_string(),
                })?;
            
            if !options.iter().any(|opt| opt.as_str() == Some(item_str)) {
                return Err(TemplateError::ValidationFailed {
                    message: format!("Invalid option in ranking: {}", item_str),
                });
            }
        }
        
        Ok(())
    }
    
    async fn canonicalize(&self, value: &Value, _params: &Value) -> Result<Vec<u8>, TemplateError> {
        match value.as_array() {
            Some(arr) => {
                let ranking_str = arr.iter()
                    .map(|v| v.as_str().unwrap_or(""))
                    .collect::<Vec<_>>()
                    .join(",");
                Ok(ranking_str.as_bytes().to_vec())
            }
            None => Err(TemplateError::CanonicalizationFailed {
                message: "Value must be an array".to_string(),
            }),
        }
    }
    
    async fn aggregate(&self, values: &[Value], params: &Value) -> Result<Value, TemplateError> {
        let options = params.get("options")
            .and_then(|o| o.as_array())
            .ok_or_else(|| TemplateError::AggregationFailed {
                message: "Template params must contain 'options' array".to_string(),
            })?;
        
        let mut scores = HashMap::new();
        
        // Initialize scores for all options
        for option in options {
            if let Some(option_str) = option.as_str() {
                scores.insert(option_str.to_string(), 0.0);
            }
        }
        
        // Calculate Borda count scores
        for value in values {
            if let Some(ranking) = value.as_array() {
                for (position, item) in ranking.iter().enumerate() {
                    if let Some(option) = item.as_str() {
                        let score = (ranking.len() - position) as f64;
                        *scores.entry(option.to_string()).or_insert(0.0) += score;
                    }
                }
            }
        }
        
        // Sort by score
        let mut sorted_scores: Vec<_> = scores.into_iter().collect();
        sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let mut result = HashMap::new();
        result.insert("ranking".to_string(), Value::Array(
            sorted_scores.iter().map(|(option, score)| {
                serde_json::json!({
                    "option": option,
                    "score": score
                })
            }).collect()
        ));
        
        Ok(Value::Object(result.into_iter().collect()))
    }
    
    fn get_schema(&self) -> Value {
        serde_json::json!({
            "type": "array",
            "description": "Array of options in order of preference"
        })
    }
}
