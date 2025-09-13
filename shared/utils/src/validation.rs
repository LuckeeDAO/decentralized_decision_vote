use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },
    
    #[error("Invalid value for field {field}: {message}")]
    InvalidValue { field: String, message: String },
    
    #[error("Value too long for field {field}: max {max}, got {actual}")]
    ValueTooLong { field: String, max: usize, actual: usize },
    
    #[error("Value too short for field {field}: min {min}, got {actual}")]
    ValueTooShort { field: String, min: usize, actual: usize },
}

/// Validate that a string is not empty
pub fn validate_not_empty(value: &str, field_name: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::RequiredFieldMissing {
            field: field_name.to_string(),
        });
    }
    Ok(())
}

/// Validate string length
pub fn validate_string_length(
    value: &str,
    field_name: &str,
    min: Option<usize>,
    max: Option<usize>,
) -> Result<(), ValidationError> {
    let len = value.len();
    
    if let Some(min_len) = min {
        if len < min_len {
            return Err(ValidationError::ValueTooShort {
                field: field_name.to_string(),
                min: min_len,
                actual: len,
            });
        }
    }
    
    if let Some(max_len) = max {
        if len > max_len {
            return Err(ValidationError::ValueTooLong {
                field: field_name.to_string(),
                max: max_len,
                actual: len,
            });
        }
    }
    
    Ok(())
}

/// Validate email format (basic validation)
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if !email.contains('@') || !email.contains('.') {
        return Err(ValidationError::InvalidValue {
            field: "email".to_string(),
            message: "Invalid email format".to_string(),
        });
    }
    Ok(())
}

/// Validate JSON value is not null
pub fn validate_not_null(value: &Value, field_name: &str) -> Result<(), ValidationError> {
    if value.is_null() {
        return Err(ValidationError::RequiredFieldMissing {
            field: field_name.to_string(),
        });
    }
    Ok(())
}

/// Validate that a number is within a range
pub fn validate_number_range(
    value: f64,
    field_name: &str,
    min: Option<f64>,
    max: Option<f64>,
) -> Result<(), ValidationError> {
    if let Some(min_val) = min {
        if value < min_val {
            return Err(ValidationError::InvalidValue {
                field: field_name.to_string(),
                message: format!("Value must be at least {}", min_val),
            });
        }
    }
    
    if let Some(max_val) = max {
        if value > max_val {
            return Err(ValidationError::InvalidValue {
                field: field_name.to_string(),
                message: format!("Value must be at most {}", max_val),
            });
        }
    }
    
    Ok(())
}
