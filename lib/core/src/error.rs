use breez_sdk_input::ParseError;

pub enum ParseAndPickError {
    Parse(ParseError),
    Pick(PickPaymentMethodError),
}
impl From<ParseError> for ParseAndPickError {
    fn from(err: ParseError) -> Self {
        ParseAndPickError::Parse(err)
    }
}
impl From<PickPaymentMethodError> for ParseAndPickError {
    fn from(err: PickPaymentMethodError) -> Self {
        ParseAndPickError::Pick(err)
    }
}
pub enum PickPaymentMethodError {
    Unsupported,
}
pub enum PrepareSendBitcoinAddressError {
    InvalidAddress,
    InvalidNetwork,
}
pub enum PrepareSendBolt11InvoiceError {}
pub enum PrepareSendBolt12InvoiceError {}
pub enum PrepareSendBolt12OfferError {}
pub enum PrepareSendLightningAddressError {}
pub enum PrepareSendLiquidAddressError {}
pub enum PrepareSendLnurlPayError {}
