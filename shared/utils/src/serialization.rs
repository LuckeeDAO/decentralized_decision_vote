use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Invalid JSON value: {message}")]
    InvalidValue { message: String },
}

/// Safely serialize a value to JSON
pub fn to_json<T: Serialize>(value: &T) -> Result<String, SerializationError> {
    serde_json::to_string(value).map_err(SerializationError::JsonError)
}

/// Safely serialize a value to pretty JSON
pub fn to_json_pretty<T: Serialize>(value: &T) -> Result<String, SerializationError> {
    serde_json::to_string_pretty(value).map_err(SerializationError::JsonError)
}

/// Safely deserialize JSON to a value
pub fn from_json<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, SerializationError> {
    serde_json::from_str(json).map_err(SerializationError::JsonError)
}

/// Safely deserialize JSON value to a typed value
pub fn from_json_value<T: for<'de> Deserialize<'de>>(value: &Value) -> Result<T, SerializationError> {
    serde_json::from_value(value.clone()).map_err(SerializationError::JsonError)
}

/// Convert a JSON value to a string representation
pub fn json_value_to_string(value: &Value) -> Result<String, SerializationError> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Ok("null".to_string()),
        _ => to_json(value),
    }
}

/// Safely parse JSON from bytes
pub fn from_json_bytes<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, SerializationError> {
    serde_json::from_slice(bytes).map_err(SerializationError::JsonError)
}

/// Safely serialize to JSON bytes
pub fn to_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, SerializationError> {
    serde_json::to_vec(value).map_err(SerializationError::JsonError)
}
