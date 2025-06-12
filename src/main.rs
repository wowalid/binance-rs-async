extern crate lazy_static;
#[macro_use]
extern crate serde;
extern crate serde_qs as qs;

pub use util::bool_to_string;
pub use util::bool_to_string_some;

pub mod client;
pub mod errors;
pub mod util;

pub mod account;
pub mod api;
pub mod config;
pub mod futures;
pub mod general;
pub mod margin;
pub mod market;
pub mod rest_model;
pub mod savings;
pub mod userstream;
pub mod wallet;
pub mod websockets;
pub mod ws_model;
#[tokio::main]
async fn main() {}
