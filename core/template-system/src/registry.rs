use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

use crate::templates::VoteTemplate;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Template not found: {id}")]
    TemplateNotFound { id: String },
    
    #[error("Template validation failed: {message}")]
    ValidationFailed { message: String },
    
    #[error("Template aggregation failed: {message}")]
    AggregationFailed { message: String },
    
    #[error("Template canonicalization failed: {message}")]
    CanonicalizationFailed { message: String },
}

/// Registry for managing vote templates
pub struct TemplateRegistry {
    templates: HashMap<String, Arc<dyn VoteTemplate>>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a new template
    pub fn register<T: VoteTemplate + 'static>(&mut self, template: T) {
        let id = template.id().to_string();
        info!("Registering template: {}", id);
        self.templates.insert(id, Arc::new(template));
    }

    /// Get a template by ID
    pub fn get(&self, id: &str) -> Result<Arc<dyn VoteTemplate>, TemplateError> {
        self.templates.get(id)
            .cloned()
            .ok_or_else(|| TemplateError::TemplateNotFound { id: id.to_string() })
    }

    /// List all registered template IDs
    pub fn list(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Check if a template exists
    pub fn exists(&self, id: &str) -> bool {
        self.templates.contains_key(id)
    }

    /// Get the number of registered templates
    pub fn count(&self) -> usize {
        self.templates.len()
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Template registry with built-in templates
pub struct DefaultTemplateRegistry {
    registry: TemplateRegistry,
}

impl Default for DefaultTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultTemplateRegistry {
    pub fn new() -> Self {
        let mut registry = TemplateRegistry::new();
        
        // Register built-in templates
        registry.register(crate::templates::YesNoTemplate::new());
        registry.register(crate::templates::MultipleChoiceTemplate::new());
        registry.register(crate::templates::NumericRangeTemplate::new());
        registry.register(crate::templates::RankingTemplate::new());
        
        Self { registry }
    }
}

impl std::ops::Deref for DefaultTemplateRegistry {
    type Target = TemplateRegistry;
    
    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}

impl std::ops::DerefMut for DefaultTemplateRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.registry
    }
}
