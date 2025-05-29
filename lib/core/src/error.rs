use breez_sdk_common::input::ParseError;

pub enum AcceptPaymentProposedFeesError {}
pub enum BuyBitcoinError {}
pub enum FetchFiatCurrenciesError {}
pub enum FetchFiatRatesError {}
pub enum FetchOnchainLimitsError {}
pub enum FetchPaymentProposedFeesError {}
pub enum FetchRecommendedFeesError {}
pub enum GetPaymentError {}
pub enum InitializeLoggingError {}
pub enum ListPaymentsError {}
pub enum ListRefundablesError {}
pub enum LnurlAuthError {}
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
pub enum PrepareBuyBitcoinError {}
pub enum PrepareSendLightningError {}
pub enum PrepareSendLiquidAddressError {}
pub enum PrepareSendLnurlPayError {}
pub enum PrepareReceivePaymentError {}
pub enum PrepareRefundError {}
pub enum ReceivePaymentError {}
pub enum RefundError {}
pub enum RegisterWebhookError {}
pub enum SendBitcoinError {}
pub enum SendLightningError {}
pub enum SendLiquidAddressError {}
pub enum SendLnurlPayError {}
pub enum SignMessageError {}
pub enum UnregisterWebhookError {}
pub enum VerifyMessageError {}
