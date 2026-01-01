//! Module that contains all the different message types sent in the network.

use crate::log;
use color_print::cformat;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Addresses {
    pub own_address: String,
    pub peer_address: String,
}

/// Struct that represents the message sent to ensure peers start the token ring after all peers have connected to the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartFlag(pub bool);

/// Struct that represents the token sent between peers to ensure mutual exclusion.
#[derive(Clone, Serialize, Deserialize)]
pub struct HotPotato(pub bool);

#[derive(Clone, Serialize, Deserialize)]
pub struct RedistributeHotPotato();

/// Enum that tells if a peer is holding the hot potato or not.
#[derive(Clone, Serialize, Deserialize)]
pub enum HotPotatoState {
    Holding(HotPotato),
    NotHolding,
}

/// Enum that represents the possible requests a peer can send to the server.
#[derive(Clone, Serialize, Deserialize)]
pub enum ServerRequest {
    Add(i32, i32),
    Sub(i32, i32),
    Mul(i32, i32),
    Div(i32, i32),
}

/// Enum that represents the possible responses a peer can receive from the server.
#[derive(Clone, Serialize, Deserialize)]
pub enum ServerResponse {
    Add(i32, i32, i32),
    Sub(i32, i32, i32),
    Mul(i32, i32, i32),
    Div(i32, i32, i32),
    Err(i32, i32, String),
}

impl Addresses {
    pub fn new(own_address: String, peer_address: String) -> Self {
        Self {
            own_address,
            peer_address,
        }
    }

    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }
}

impl RedistributeHotPotato {
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }
}

impl StartFlag {
    /// Function that returns the start flag as a JSON formatted `String`.
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    /// Function that parses the start flag from a JSON formatted `String`.
    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }
}

impl HotPotato {
    /// Function that creates a new hot potato.
    pub fn new() -> Self {
        Self(true)
    }

    /// Function that returns the hot potato as a JSON formatted `String`.
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    /// Function that parses the hot potato from a JSON formatted `String`.
    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }
}

impl HotPotatoState {
    /// Function that returns the hot potato state as a JSON formatted `String`.
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    /// Function that parses the hot potato state from a JSON formatted `String`.
    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }
}

impl ServerRequest {
    /// Function that returns the server request as a JSON formatted `String`.
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    /// Function that parses the server request from a JSON formatted `String`.
    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }

    /// Function that computes the response of a request.
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

    /// Function that prints a request.
    pub fn print(&self) {
        match self {
            Self::Add(a, b) => log::info(&cformat!("Asking the server to perform the <bold>addition</bold> of <bold>{a}</bold> and <bold>{b}</bold>.")),
            Self::Sub(a, b) => log::info(&cformat!("Asking the server to perform the <bold>subtraction</bold> of <bold>{a}</bold> and <bold>{b}</bold>.")),
            Self::Mul(a, b) => log::info(&cformat!("Asking the server to perform the <bold>multiplication</bold> of <bold>{a}</bold> and <bold>{b}</bold>.")),
            Self::Div(a, b) => log::info(&cformat!("Asking the server to perform the <bold>division of</bold> <bold>{a}</bold> and <bold>{b}</bold>.")),
        }
    }

    /// Function that generates a request.
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
    /// Function that returns the server response as a JSON formatted `String`.
    pub fn to_json_string(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }

    /// Function that parses the server response from a JSON formatted `String`.
    pub fn from_json_string(token: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_str::<Self>(token)?)
    }

    /// Function that prints a response from the server.
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
