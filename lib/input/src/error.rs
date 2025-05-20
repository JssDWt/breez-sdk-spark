pub type ParseResult<T> = Result<T, ParseError>;

pub struct LnUrlErrorData {
    pub reason: String,
}

pub enum ParseError {
    LnUrlError(LnUrlErrorData),
}
