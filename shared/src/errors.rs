use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
}

impl ApiErrorResponse {
    pub fn new(message: &str) -> Self {
        ApiErrorResponse {
            error: message.to_string(),
        }
    }
}
