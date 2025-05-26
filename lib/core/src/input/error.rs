use thiserror::Error;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct LnurlErrorData {
    pub reason: String,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("empty input")]
    EmptyInput,
    #[error("invalid input")]
    InvalidInput,
    #[error("Lnurl error: {}", .0.reason)]
    LnurlError(LnurlErrorData),
}
