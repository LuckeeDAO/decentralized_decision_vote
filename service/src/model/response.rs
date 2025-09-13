use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T=serde_json::Value> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: Option<T>) -> Self { Self { code: 0, message: "ok".into(), data } }
    pub fn error(msg: &str) -> Self { Self { code: 1, message: msg.into(), data: None } }
}
