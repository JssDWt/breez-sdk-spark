use breez_sdk_common::input::ParseError;

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
pub enum PrepareSendBitcoinError {
    InvalidAddress,
    InvalidNetwork,
}
pub enum PrepareSendLightningError {}
pub enum PrepareSendLiquidAddressError {}
pub enum PrepareSendLnurlPayError {}
pub enum PrepareReceivePaymentError {}
pub enum ReceivePaymentError {}
pub enum SendBitcoinError {}
pub enum SendLightningError {}
pub enum SendLiquidAddressError {}
pub enum SendLnurlPayError {}
