//! This project is an implementation of Mutual Exclusion with the Token Ring Algorithm system,
//! developed in Rust using Tokio for asynchronous communication.
//!
//! # Assignment
//!
//! - Create a ring network with 5 peers (call them p1 to p5) such as the one represented in
//! the figure above. Each peer must be in a different machine (m1 to m5). Besides these 5
//! peers, create also a calculatormulti server called server that runs on machine m6.
//!
//! - Each peer only knows the IP address of the machine where the next peer is located: p1knows the IP of m2,
//! p2 knows the IP of m3, . . . , p5 has the IP of m1, thus closing the
//! ring. All peers know the IP address of the machine where the server is located (m6).
//! These can be passed to the peer via the command line, e.g., for peer p2 you would run
//! the following command in machine m2: $ peer IP-of-m3 IP-of-m6.
//!
//! - One of the threads in each peer generates requests for the server following a Poisson
//! distribution with a frequency of 4 per minute. The operation and the arguments for
//! each request are also random. These requests are placed in a local queue.
//!
//! - Another thread in that peer runs in a loop waiting for a message (that can only come
//! from the previous peer). This message is designated a token. The token can be a void
//! message. The peer that holds it at any given moment has exclusive access to server,
//! effectively implementing mutual exclusion.
//!
//! - When the peer receives the token, it checks if it has requests for the server in its local
//! queue. If it does, it holds the token until all requests are processed at the server; after
//! the results are received and printed on the terminal, the peer restarts the forwarding of
//! the token. If it does not, it forwards the token to the next peer in the ring.
//!
//! - **EXTRA MARKS:** implement a scheme that enables the token to continue to move
//! in the event of a failure in one of the peers.

use std::time::Duration;

use crate::message::*;
use crate::poisson::*;
use crate::sync::*;

pub mod log;
pub mod message;
pub mod peer;
pub mod poisson;
pub mod server;
pub mod sync;

/// Constant value for the rate used in the Poisson distribution.
pub const RATE: f64 = 1.;

pub const TIMEOUT_DURATION: Duration = Duration::from_secs(5);
