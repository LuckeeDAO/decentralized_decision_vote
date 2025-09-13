use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub trait VoteValueTemplate: Send + Sync {
    fn id(&self) -> &'static str;
    fn validate(&self, raw: &Value, params: &Value) -> Result<(), String>;
    fn canonicalize(&self, raw: &Value, params: &Value) -> Result<Vec<u8>, String>;
    fn reduce(&self, values: &[Value]) -> Value { serde_json::json!(values.len()) }
}

#[derive(Default)]
pub struct TemplateRegistry {
    inner: HashMap<String, Arc<dyn VoteValueTemplate>>,
}

impl TemplateRegistry {
    pub fn new() -> Self { Self { inner: HashMap::new() } }
    pub fn register<T: VoteValueTemplate + 'static>(&mut self, t: T) {
        self.inner.insert(t.id().to_string(), Arc::new(t));
    }
    pub fn get(&self, id: &str) -> Result<Arc<dyn VoteValueTemplate>, String> {
        self.inner.get(id).cloned().ok_or_else(|| format!("template not found: {}", id))
    }
    pub fn list_ids(&self) -> Vec<String> { self.inner.keys().cloned().collect() }
}

pub struct BitTemplate;
impl VoteValueTemplate for BitTemplate {
    fn id(&self) -> &'static str { "bit" }
    fn validate(&self, raw: &Value, _params: &Value) -> Result<(), String> {
        match raw {
            Value::Bool(_) => Ok(()),
            Value::Number(n) if n.as_u64() == Some(0) || n.as_u64() == Some(1) => Ok(()),
            _ => Err("bit expects boolean or 0/1".into()),
        }
    }
    fn canonicalize(&self, raw: &Value, _params: &Value) -> Result<Vec<u8>, String> {
        let b = match raw {
            Value::Bool(b) => *b as u8,
            Value::Number(n) if n.as_u64() == Some(0) || n.as_u64() == Some(1) => n.as_u64().unwrap() as u8,
            _ => return Err("bit expects boolean or 0/1".into()),
        };
        Ok(vec![b])
    }
}

pub struct OptionIndexTemplate;
impl VoteValueTemplate for OptionIndexTemplate {
    fn id(&self) -> &'static str { "option_index" }
    fn validate(&self, raw: &Value, params: &Value) -> Result<(), String> {
        let max = params.get("max").and_then(|v| v.as_u64()).ok_or("missing param max")?;
        let idx = raw.as_u64().ok_or("option_index expects number")?;
        if idx < max { Ok(()) } else { Err(format!("index out of range: {} >= {}", idx, max)) }
    }
    fn canonicalize(&self, raw: &Value, params: &Value) -> Result<Vec<u8>, String> {
        self.validate(raw, params)?;
        let idx = raw.as_u64().unwrap();
        Ok(idx.to_be_bytes().to_vec())
    }
}

pub struct StringTemplate;
impl VoteValueTemplate for StringTemplate {
    fn id(&self) -> &'static str { "string" }
    fn validate(&self, raw: &Value, params: &Value) -> Result<(), String> {
        if !raw.is_string() { return Err("string expects string".into()); }
        if let Some(max_len) = params.get("max_len").and_then(|v| v.as_u64()) {
            if raw.as_str().unwrap().len() as u64 > max_len { return Err("string too long".into()); }
        }
        Ok(())
    }
    fn canonicalize(&self, raw: &Value, _params: &Value) -> Result<Vec<u8>, String> {
        Ok(raw.as_str().unwrap().as_bytes().to_vec())
    }
}
