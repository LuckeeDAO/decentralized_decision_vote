use serde_json::Value;
use crate::registry::TemplateError;

/// Validator for template-related operations
pub struct TemplateValidator;

impl Default for TemplateValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate template parameters
    pub fn validate_template_params(&self, template_id: &str, params: &Value) -> Result<(), TemplateError> {
        match template_id {
            "yes_no" => self.validate_yes_no_params(params),
            "multiple_choice" => self.validate_multiple_choice_params(params),
            "numeric_range" => self.validate_numeric_range_params(params),
            "ranking" => self.validate_ranking_params(params),
            _ => Err(TemplateError::ValidationFailed {
                message: format!("Unknown template ID: {}", template_id),
            }),
        }
    }

    fn validate_yes_no_params(&self, _params: &Value) -> Result<(), TemplateError> {
        // Yes/No template doesn't require specific parameters
        Ok(())
    }

    fn validate_multiple_choice_params(&self, params: &Value) -> Result<(), TemplateError> {
        let choices = params.get("choices")
            .and_then(|c| c.as_array())
            .ok_or_else(|| TemplateError::ValidationFailed {
                message: "Template params must contain 'choices' array".to_string(),
            })?;

        if choices.is_empty() {
            return Err(TemplateError::ValidationFailed {
                message: "Choices array cannot be empty".to_string(),
            });
        }

        if choices.len() > 20 {
            return Err(TemplateError::ValidationFailed {
                message: "Too many choices (maximum 20)".to_string(),
            });
        }

        for choice in choices {
            if !choice.is_string() {
                return Err(TemplateError::ValidationFailed {
                    message: "All choices must be strings".to_string(),
                });
            }
        }

        Ok(())
    }

    fn validate_numeric_range_params(&self, params: &Value) -> Result<(), TemplateError> {
        let min = params.get("min").and_then(|m| m.as_f64());
        let max = params.get("max").and_then(|m| m.as_f64());

        if let (Some(min_val), Some(max_val)) = (min, max) {
            if min_val >= max_val {
                return Err(TemplateError::ValidationFailed {
                    message: "Min value must be less than max value".to_string(),
                });
            }
        }

        Ok(())
    }

    fn validate_ranking_params(&self, params: &Value) -> Result<(), TemplateError> {
        let options = params.get("options")
            .and_then(|o| o.as_array())
            .ok_or_else(|| TemplateError::ValidationFailed {
                message: "Template params must contain 'options' array".to_string(),
            })?;

        if options.is_empty() {
            return Err(TemplateError::ValidationFailed {
                message: "Options array cannot be empty".to_string(),
            });
        }

        if options.len() > 10 {
            return Err(TemplateError::ValidationFailed {
                message: "Too many options (maximum 10)".to_string(),
            });
        }

        for option in options {
            if !option.is_string() {
                return Err(TemplateError::ValidationFailed {
                    message: "All options must be strings".to_string(),
                });
            }
        }

        Ok(())
    }
}
