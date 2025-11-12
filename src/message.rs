use color_print::cformat;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::sync::mpsc;

use crate::log;

pub type HotPotatoTx = mpsc::UnboundedSender<HotPotato>;
pub type HotPotatoRx = mpsc::UnboundedReceiver<HotPotato>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartFlag(pub bool);

#[derive(Clone, Serialize, Deserialize)]
pub struct HotPotato(pub bool);

#[derive(Clone)]
pub enum HotPotatoState {
    Holding(HotPotato),
    NotHolding,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Request {
    Add(i32, i32),
    Sub(i32, i32),
    Mul(i32, i32),
    Div(i32, i32),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Response {
    Add(i32, i32, i32),
    Sub(i32, i32, i32),
    Mul(i32, i32, i32),
    Div(i32, i32, i32),
    Err(String),
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

impl Request {
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }

    pub fn to_response(&self) -> Response {
        match self {
            Request::Add(a, b) => Response::Add(*a, *b, a + b),
            Request::Sub(a, b) => Response::Sub(*a, *b, a - b),
            Request::Mul(a, b) => Response::Mul(*a, *b, a * b),
            Request::Div(a, b) => Response::Div(*a, *b, a / b),
        }
    }

    pub fn print(&self) {
        match self {
            Request::Add(a, b) => log::info(&cformat!("Asking the server to perform the addition of <bold>{a}</bold> and <bold>{b}</bold>.")),
            Request::Sub(a, b) => log::info(&cformat!("Asking the server to perform the subtraction of <bold>{a}</bold> and <bold>{b}</bold>.")),
            Request::Mul(a, b) => log::info(&cformat!("Asking the server to perform the multiplication of <bold>{a}</bold> and <bold>{b}</bold>.")),
            Request::Div(a, b) => log::info(&cformat!("Asking the server to perform the division of <bold>{a}</bold> and <bold>{b}</bold>.")),
        }
    }
}

impl Response {
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }

    pub fn response_error(message: &str) -> Response {
        Response::Err(message.to_string())
    }

    pub fn print(&self) {
        match self {
            Response::Add(a, b, result) => log::info(&cformat!("The result of the <bold>addition</bold> of <bold>{a}</bold> and <bold>{b}</bold> is <bold>{result}</bold>.")),
            Response::Sub(a, b, result) => log::info(&cformat!("The result of the <bold>subtraction</bold> of <bold>{a}</bold> and <bold>{b}</bold> is <bold>{result}</bold>.")),
            Response::Mul(a, b, result) => log::info(&cformat!("The result of the <bold>multiplication</bold> of <bold>{a}</bold> and <bold>{b}</bold> is <bold>{result}</bold>.")),
            Response::Div(a, b, result) => log::info(&cformat!("The result of the <bold>division</bold> of <bold>{a}</bold> and <bold>{b}</bold> is <bold>{result}</bold>.")),
            Response::Err(e) => log::error(e),
        }
    }
}
