use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::sync::mpsc;

#[derive(Clone, Serialize, Deserialize)]
pub enum Request {
    Add(i32, i32),
    Sub(i32, i32),
    Mul(i32, i32),
    Div(i32, i32),
}

impl Request {
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Response {
    Ok { value: String, r#type: String },
    Err(String),
}

impl Response {
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }

    pub fn print(&self) {}
}
