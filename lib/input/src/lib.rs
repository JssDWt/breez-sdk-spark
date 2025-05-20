mod error;
mod models;
mod network;

pub use error::{ParseError, ParseResult};
pub use models::*;
pub use network::Network;

pub async fn parse(input: &str) -> ParseResult<InputType> {
    todo!()
}
