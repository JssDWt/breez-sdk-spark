mod error;
mod models;
mod network;

pub use error::{ParseError, ParseResult};
pub use models::*;
pub use network::Network;

use crate::utils::RestClient;

pub async fn parse(input: &str) -> ParseResult<InputType> {
    todo!()
}

pub async fn parse_with_rest_client<C: RestClient + ?Sized>(
    rest_client: &C,
    input: &str,
) -> ParseResult<InputType> {
    todo!()
}
