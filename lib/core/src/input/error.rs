pub type ParseResult<T> = Result<T, ParseError>;

pub struct LnurlErrorData {
    pub reason: String,
}

pub enum ParseError {
    LnurlError(LnurlErrorData),
}
