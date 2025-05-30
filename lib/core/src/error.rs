use breez_sdk_common::{buy::moonpay::MoonpayProvider, input::ParseError};
use thiserror::Error;

use crate::BuyBitcoinProvider;

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum AcceptPaymentProposedFeesError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum BuyBitcoinError {
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Invalid network: can only buy bitcoin on mainnet")]
    InvalidNetwork,
    #[error("Provider error: {provider}, error: {error}")]
    ProviderError {
        provider: BuyBitcoinProvider,
        error: String,
    },
    #[error(transparent)]
    ReceiveError(#[from] ReceivePaymentError),
    #[error("General error: {0}")]
    General(String),
}

impl From<PrepareBuyBitcoinError> for BuyBitcoinError {
    fn from(err: PrepareBuyBitcoinError) -> Self {
        match err {
            PrepareBuyBitcoinError::InvalidAmount(amount) => BuyBitcoinError::InvalidAmount(amount),
            PrepareBuyBitcoinError::InvalidNetwork => BuyBitcoinError::InvalidNetwork,
            PrepareBuyBitcoinError::ReceiveError(err) => BuyBitcoinError::General(err.to_string()),
        }
    }
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ConnectError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum FetchFiatCurrenciesError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum FetchFiatRatesError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum FetchOnchainLimitsError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum FetchPaymentProposedFeesError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum FetchRecommendedFeesError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum GetInfoError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum GetPaymentError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum InitializeLoggingError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ListPaymentsError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ListRefundablesError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum LnurlAuthError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ParseAndPickError {
    #[error("Error parsing input: {0}")]
    Parse(ParseError),
    #[error("Error picking payment method: {0}")]
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

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PickPaymentMethodError {
    #[error("Unsupported payment method")]
    Unsupported,
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PrepareSendBitcoinError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PrepareBuyBitcoinError {
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid network")]
    InvalidNetwork,

    #[error(transparent)]
    ReceiveError(#[from] PrepareReceivePaymentError),
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PrepareSendLightningError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PrepareSendLiquidAddressError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PrepareSendLnurlPayError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PrepareReceivePaymentError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum PrepareRefundError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ReceivePaymentError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum RefundError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum RegisterWebhookError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum SendBitcoinError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum SendLightningError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum SendLiquidAddressError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum SendLnurlPayError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum SignMessageError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum StopError {
    #[error("Failed to send stop signal")]
    SendSignalFailed,
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum UnregisterWebhookError {}

#[derive(Debug, Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum VerifyMessageError {}
