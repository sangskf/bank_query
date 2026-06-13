use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bank {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<Bank>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct BankUpdate {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct AccessLog {
    pub id: i64,
    pub ip: String,
    pub action: String,
    pub accessed_at: String,
}
