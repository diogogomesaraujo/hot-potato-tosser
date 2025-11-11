use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::sync::mpsc;

pub type HotPotatoTx = mpsc::UnboundedSender<HotPotato>;
pub type HotPotatoRx = mpsc::UnboundedReceiver<HotPotato>;

#[derive(Clone)]
pub enum HotPotatoState {
    Holding(HotPotato),
    NotHolding,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HotPotato(pub bool);

impl HotPotato {
    pub fn new() -> Self {
        Self(true)
    }

    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }

    pub async fn from_json_string_async(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }
}
