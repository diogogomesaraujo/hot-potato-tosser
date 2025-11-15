use color_print::cformat;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{error::Error, net::SocketAddr};
use tokio::sync::mpsc;

use crate::log;

pub type FindHotPotatoStateTx = mpsc::UnboundedSender<FindHotPotato>;
pub type FindHotPotatoStateRx = mpsc::UnboundedReceiver<FindHotPotato>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartFlag(pub bool);

#[derive(Clone, Serialize, Deserialize)]
pub struct HotPotato(pub bool);

#[derive(Clone, Serialize, Deserialize)]
pub enum HotPotatoState {
    Holding(HotPotato),
    NotHolding,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum FindHotPotato {
    Response {
        hot_potato_state: HotPotatoState,
        previous_peer_address: SocketAddr,
    },
    Request,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ServerRequest {
    Add(i32, i32),
    Sub(i32, i32),
    Mul(i32, i32),
    Div(i32, i32),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ServerResponse {
    Add(i32, i32, i32),
    Sub(i32, i32, i32),
    Mul(i32, i32, i32),
    Div(i32, i32, i32),
    Err(i32, i32, String),
}

impl StartFlag {
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }
}

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
}

impl HotPotatoState {
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }
}

impl FindHotPotato {
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }
}

impl ServerRequest {
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }

    pub fn to_response(&self) -> ServerResponse {
        match self {
            Self::Add(a, b) => match a.checked_add(*b) {
                Some(result) => ServerResponse::Add(*a, *b, result),
                None => ServerResponse::Err(*a, *b, cformat!("Failed to compute the <bold>addition</bold> of <bold>{a}</bold> and <bold>{b}</bold>.")),
            },
            Self::Sub(a, b) => match a.checked_sub(*b) {
                Some(result) => ServerResponse::Sub(*a, *b, result),
                None => ServerResponse::Err(*a, *b, cformat!("Failed to compute the <bold>subtraction</bold> of <bold>{a}</bold> and <bold>{b}</bold>.")),
            },
            Self::Mul(a, b) => match a.checked_mul(*b) {
                Some(result) => ServerResponse::Mul(*a, *b, result),
                None => ServerResponse::Err(*a, *b, cformat!("Failed to compute the <bold>multiplication</bold> of <bold>{a}</bold> and <bold>{b}</bold>.")),
            },
            Self::Div(a, b) => match a.checked_div(*b) {
                Some(result) => ServerResponse::Div(*a, *b, result),
                None => ServerResponse::Err(*a, *b, cformat!("Failed to compute the <bold>division</bold> of <bold>{a}</bold> and <bold>{b}</bold>.")),
            },
        }
    }

    pub fn print(&self) {
        match self {
            Self::Add(a, b) => log::info(&cformat!("Asking the server to perform the <bold>addition</bold> of <bold>{a}</bold> and <bold>{b}</bold>.")),
            Self::Sub(a, b) => log::info(&cformat!("Asking the server to perform the <bold>subtraction</bold> of <bold>{a}</bold> and <bold>{b}</bold>.")),
            Self::Mul(a, b) => log::info(&cformat!("Asking the server to perform the <bold>multiplication</bold> of <bold>{a}</bold> and <bold>{b}</bold>.")),
            Self::Div(a, b) => log::info(&cformat!("Asking the server to perform the <bold>division of</bold> <bold>{a}</bold> and <bold>{b}</bold>.")),
        }
    }

    pub fn generate<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let a = rng.random::<i32>();
        let b = rng.random::<i32>();
        let op = rng.random_range(0..4);

        match op {
            0 => Self::Add(a, b),
            1 => Self::Sub(a, b),
            2 => Self::Mul(a, b),
            3 => Self::Div(a, b),
            _ => Self::Add(0, 0), // shouldn't happen
        }
    }
}

impl ServerResponse {
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }

    pub fn print(&self) {
        match self {
            Self::Add(a, b, result) => log::info(&cformat!("The result of the <bold>addition</bold> of <bold>{a}</bold> and <bold>{b}</bold> is <bold>{result}</bold>.")),
            Self::Sub(a, b, result) => log::info(&cformat!("The result of the <bold>subtraction</bold> of <bold>{a}</bold> and <bold>{b}</bold> is <bold>{result}</bold>.")),
            Self::Mul(a, b, result) => log::info(&cformat!("The result of the <bold>multiplication</bold> of <bold>{a}</bold> and <bold>{b}</bold> is <bold>{result}</bold>.")),
            Self::Div(a, b, result) => log::info(&cformat!("The result of the <bold>division</bold> of <bold>{a}</bold> and <bold>{b}</bold> is <bold>{result}</bold>.")),
            Self::Err(_, _, e) => log::error(e),
        }
    }
}
