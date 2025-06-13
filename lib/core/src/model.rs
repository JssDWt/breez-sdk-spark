use breez_sdk_common::{
    fiat::{FiatCurrency, Rate},
    input::{
        BitcoinAddress, Bolt11Invoice, Bolt12Invoice, Bolt12InvoiceRequest, Bolt12Offer,
        LiquidAddress, LnurlPayRequest, LnurlWithdrawRequestData, RawPaymentMethod,
        SilentPaymentAddress, SuccessActionProcessed,
    },
    lnurl::{LnurlCallbackStatus, LnurlErrorData, auth::LnurlAuthRequestData},
};
use maybe_sync::{MaybeSend, MaybeSync};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AcceptPaymentProposedFeesRequest {
    pub response: FetchPaymentProposedFeesResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AcceptPaymentProposedFeesResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AddEventListenerResponse {
    pub listener_id: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Display, EnumString, PartialEq, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum BuyBitcoinProvider {
    #[strum(serialize = "moonpay")]
    Moonpay,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct BuyBitcoinRequest {
    pub prepared: PrepareBuyBitcoinResponse,

    /// The optional URL to redirect to after completing the buy.
    ///
    /// For Moonpay, see <https://dev.moonpay.com/docs/on-ramp-configure-user-journey-params>
    pub redirect_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct BuyBitcoinResponse {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ConnectRequest {
    pub config: Config,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Config {
    pub mnemonic: String,
    pub network: Network,
    pub data_dir: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FeeBreakdown {} // TODO: This type may vary across different SDKs.

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FetchFiatCurrenciesResponse {
    pub currencies: Vec<FiatCurrency>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FetchFiatRatesResponse {
    pub rates: Vec<Rate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FetchOnchainLimitsResponse {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FetchPaymentProposedFeesRequest {
    pub payment_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FetchPaymentProposedFeesResponse {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FetchRecommendedFeesResponse {
    pub fastest_fee: u64,
    pub half_hour_fee: u64,
    pub hour_fee: u64,
    pub economy_fee: u64,
    pub minimum_fee: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct GetInfoResponse {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct InitializeLoggingRequest {
    pub log_dir: String,
    // TODO: Add app logger using tracing crate or create a custom interface for logging
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct InitializeLoggingResponse {}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ListPaymentsRequest {
    pub filters: Option<Vec<PaymentType>>,
    pub states: Option<Vec<PaymentState>>,
    /// Epoch time, in seconds
    pub from_timestamp: Option<u64>,
    /// Epoch time, in seconds
    pub to_timestamp: Option<u64>,
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub sort_ascending: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ListPaymentsResponse {
    pub payments: Vec<Payment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ListRefundablesResponse {
    pub payments: Vec<Payment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LnurlAuthRequest {
    pub data: LnurlAuthRequestData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LnurlAuthResponse {
    // TODO: This should be empty, only on error should it contain an error message?
    pub callback_status: LnurlCallbackStatus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LnurlPayErrorData {
    pub payment_hash: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum LnurlPayResult {
    EndpointSuccess(LnurlPaySuccessData),
    EndpointError(LnurlErrorData),
    PayError(LnurlPayErrorData),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LnurlPaySuccessData {
    pub payment: Payment,
    pub success_action: Option<SuccessActionProcessed>,
}

#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, Serialize, Deserialize)]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum Network {
    Mainnet,
    Regtest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Payment {
    pub amount_msat: u64,
    pub created_at: u64,
    pub fee_msat: u64,
    pub fee_breakdown: FeeBreakdown,
    pub id: String,
    pub payment_method: RawPaymentMethod,
    pub payment_request: String,
    pub payment_type: PaymentType,
    pub status: PaymentState,
    pub details: PaymentDetails,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PaymentDetails {} // TODO: This type may vary across different SDKs.

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Display, EnumString, Eq, Hash, PartialEq, Serialize,
)]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PaymentState {
    #[default]
    Created = 0,
    Pending = 1,
    Complete = 2,
    Failed = 3,
    TimedOut = 4,
    Refundable = 5,
    RefundPending = 6,
    WaitingFeeAcceptance = 7,
}

#[derive(Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, Hash, PartialEq, Serialize)]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PaymentType {
    Receive = 0,
    Send = 1,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LightningAddress {
    pub address: String,
    pub pay_request: LnurlPayRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrepareBuyBitcoinRequest {
    pub provider: BuyBitcoinProvider,
    pub amount_sat: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrepareBuyBitcoinResponse {
    pub req: PrepareBuyBitcoinRequest,
    pub fee_msat: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrepareReceivePaymentRequest {
    pub amount_msat: u64,
    pub message: Option<String>,
    pub receive_method: ReceiveMethod,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrepareReceivePaymentResponse {
    pub req: PrepareReceivePaymentRequest,
    pub fee_msat: u64,
    pub min_payer_amount_msat: u64,
    pub max_payer_amount_msat: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrepareRefundRequest {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrepareRefundResponse {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PrepareSendPaymentRequest {
    BitcoinAddress {
        address: BitcoinAddress,
        amount_sat: u64,
        fee_rate_sat_per_vbyte: Option<u64>,
    },
    Bolt11Invoice {
        invoice: Bolt11Invoice,
        amount_msat: u64,
    },
    Bolt12Invoice {
        invoice: Bolt12Invoice,
    },
    Bolt12Offer {
        offer: Bolt12Offer,
        amount_msat: u64,
        message: Option<String>,
    },
    LightningAddress {
        address: LightningAddress,
        amount_msat: u64,
        message: Option<String>,
    },
    LiquidAddress {
        address: LiquidAddress,
        amount_sat: u64,
    },
    LnurlPay {
        url: LnurlPayRequest,
        amount_msat: u64,
        message: Option<String>,
    },
    SilentPaymentAddress {
        address: SilentPaymentAddress,
        amount_sat: u64,
        fee_rate_sat_per_vbyte: Option<u64>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PrepareSendPaymentResponse {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum ReceiveMethod {
    BitcoinAddress,
    Bolt11Invoice,
    Bolt12InvoiceRequest(Bolt12InvoiceRequest),
    Bolt12Offer,
    LnurlWithdraw(LnurlWithdrawRequestData),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ReceivePaymentRequest {
    pub prepared: PrepareReceivePaymentResponse,
    pub description: Option<String>,
    pub use_description_hash: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ReceivePaymentResponse {
    pub payment_request: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RecommendedFees {
    pub fastest_fee: u64,
    pub half_hour_fee: u64,
    pub hour_fee: u64,
    pub economy_fee: u64,
    pub minimum_fee: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RefundRequest {
    pub prepared: PrepareRefundResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RefundResponse {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RegisterWebhookRequest {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RegisterWebhookResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RemoveEventListenerRequest {
    pub listener_id: String,
}

/// Trait that can be used to react to various [`SdkEvent`]s emitted by the SDK.
#[cfg_attr(feature = "uniffi", uniffi::export(callback_interface))]
pub trait SdkEventListener: MaybeSend + MaybeSync {
    fn on_event(&self, e: SdkEvent);
}

/// Event emitted by the SDK. Add an [`SdkEventListener`] by calling [crate::sdk::Sdk::add_event_listener]
/// to listen for emitted events.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum SdkEvent {
    PaymentFailed(Payment),
    PaymentPending(Payment),
    PaymentRefundable(Payment),
    PaymentRefunded(Payment),
    PaymentRefundPending(Payment),
    PaymentSucceeded(Payment),
    PaymentWaitingConfirmation(Payment),
    PaymentWaitingFeeAcceptance(Payment),
    Synced,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SendPaymentRequest {
    pub prepared: PrepareSendPaymentResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SendPaymentResponse {
    pub payment: Payment,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SignMessageRequest {
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SignMessageResponse {
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct UnregisterWebhookRequest {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct UnregisterWebhookResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct VerifyMessageRequest {
    /// The message that was signed.
    pub message: String,
    /// The public key of the node that signed the message.
    pub pubkey: String,
    /// The zbase encoded signature to verify.
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct VerifyMessageResponse {
    /// Boolean value indicating whether the signature covers the message and
    /// was signed by the given pubkey.
    pub is_valid: bool,
}
