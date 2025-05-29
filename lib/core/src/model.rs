use breez_sdk_common::{
    fiat::{FiatCurrency, Rate},
    input::{
        BitcoinAddress, Bolt11Invoice, Bolt12Invoice, Bolt12Offer, LightningAddress, LiquidAddress,
        LnurlAuthRequestData, LnurlPayRequest, PaymentMethod, ReceiveRequest,
        SilentPaymentAddress, SuccessActionProcessed,
    },
    lnurl::{LnurlCallbackStatus, LnurlErrorData},
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AcceptPaymentProposedFeesRequest {
    pub response: FetchPaymentProposedFeesResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AcceptPaymentProposedFeesResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BitcoinPaymentMethod {
    BitcoinAddress(BitcoinAddress),
    SilentPaymentAddress(SilentPaymentAddress),
}

#[derive(Debug, Clone, Copy, Deserialize, EnumString, PartialEq, Serialize)]
pub enum BuyBitcoinProvider {
    #[strum(serialize = "moonpay")]
    Moonpay,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuyBitcoinRequest {
    pub prepare_response: PrepareBuyBitcoinResponse,

    /// The optional URL to redirect to after completing the buy.
    ///
    /// For Moonpay, see <https://dev.moonpay.com/docs/on-ramp-configure-user-journey-params>
    pub redirect_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuyBitcoinResponse {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeeBreakdown {} // TODO: This type may vary across different SDKs.

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FetchFiatCurrenciesResponse {
    pub currencies: Vec<FiatCurrency>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FetchFiatRatesResponse {
    pub rates: Vec<Rate>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FetchOnchainLimitsResponse {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FetchPaymentProposedFeesRequest {
    pub payment_id: PaymentId,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FetchPaymentProposedFeesResponse {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FetchRecommendedFeesResponse {
    pub fastest_fee: u64,
    pub half_hour_fee: u64,
    pub hour_fee: u64,
    pub economy_fee: u64,
    pub minimum_fee: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitializeLoggingRequest {
    pub log_dir: String,
    // TODO: Add app logger using tracing crate or create a custom interface for logging
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitializeLoggingResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LightningPaymentRequest {
    pub min_amount: MilliSatoshi,
    pub max_amount: MilliSatoshi,
    pub method: LightningPaymentMethod,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LightningPaymentMethod {
    Bolt11Invoice(Bolt11Invoice),
    Bolt12Invoice(Bolt12Invoice),
    Bolt12Offer(Bolt12Offer),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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
pub struct ListPaymentsResponse {
    pub payments: Vec<Payment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListRefundablesResponse {
    pub payments: Vec<Payment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LnurlAuthRequest {
    pub data: LnurlAuthRequestData,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LnurlAuthResponse {
    // TODO: This should be empty, only on error should it contain an error message?
    pub callback_status: LnurlCallbackStatus,
}

// TODO: Create easier interface for lnurl pay
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LnurlPaymentMethod {
    LnurlPay(LnurlPayRequest),
    LightningAddress(LightningAddress),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LnurlPayErrorData {
    pub payment_hash: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LnurlPayResult {
    EndpointSuccess(LnurlPaySuccessData),
    EndpointError(LnurlErrorData),
    PayError(LnurlPayErrorData),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LnurlPaySuccessData {
    pub payment: Payment,
    pub success_action: Option<SuccessActionProcessed>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MilliSatoshi(pub u64); // TODO: This type may vary across different SDKs. It may include assets in liquid for example.

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Payment {
    pub amount: MilliSatoshi,
    pub created_at: u64,
    pub fee: MilliSatoshi,
    pub fee_breakdown: FeeBreakdown,
    pub id: PaymentId,
    pub payment_method: PaymentMethod,
    pub payment_request: String,
    pub payment_type: PaymentType,
    pub status: PaymentState,
    pub details: PaymentDetails,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaymentDetails {} // TODO: This type may vary across different SDKs.

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaymentId(pub String);

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Display, EnumString, Eq, Hash, PartialEq, Serialize,
)]
#[strum(serialize_all = "lowercase")]
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
pub enum PaymentType {
    Receive = 0,
    Send = 1,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PickedPaymentMethod {
    Bitcoin(BitcoinPaymentMethod),
    Lightning(LightningPaymentRequest),
    LnurlPay(LnurlPaymentMethod),
    LiquidAddress(LiquidAddress),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PickedInputType {
    LnurlAuth(LnurlAuthRequestData),
    PaymentMethod(PickedPaymentMethod),
    ReceiveRequest(ReceiveRequest),
    Url(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareBuyBitcoinRequest {
    pub provider: BuyBitcoinProvider,
    pub amount_sat: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareBuyBitcoinResponse {
    pub req: PrepareBuyBitcoinRequest,
    pub fee: MilliSatoshi,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareReceivePaymentRequest {
    pub amount: MilliSatoshi,
    pub receive_method: ReceiveMethod,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareReceivePaymentResponse {
    pub req: PrepareReceivePaymentRequest,
    pub fee: MilliSatoshi,
    pub min_payer_amount: MilliSatoshi,
    pub max_payer_amount: MilliSatoshi,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareRefundRequest {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareRefundResponse {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBitcoinRequest {
    pub method: BitcoinPaymentMethod,
    pub fee_rate_sat_per_kw: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBitcoinResponse {
    pub req: PrepareSendBitcoinRequest,
    pub fee: MilliSatoshi,
    pub fee_breakdown: FeeBreakdown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLightningRequest {
    pub payment_request: LightningPaymentRequest,
    pub amount: MilliSatoshi,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLightningResponse {
    pub req: PrepareSendLightningRequest,
    pub fee: MilliSatoshi,
    pub fee_breakdown: FeeBreakdown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLnurlPayRequest {
    pub lnurl_pay: LnurlPaymentMethod,
    pub amount: MilliSatoshi,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLnurlPayResponse {
    pub req: PrepareSendLnurlPayRequest,
    pub fee: MilliSatoshi,
    pub fee_breakdown: FeeBreakdown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLiquidAddressRequest {
    pub address: LiquidAddress,
    pub amount: MilliSatoshi,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLiquidAddressResponse {
    pub req: PrepareSendLiquidAddressRequest,
    pub fee: MilliSatoshi,
    pub fee_breakdown: FeeBreakdown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ReceiveMethod {
    BitcoinAddress,
    Bolt11Invoice,
    Bolt12Offer,
    ReceiveRequest(ReceiveRequest),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceivePaymentRequest {
    pub prepared: PrepareReceivePaymentResponse,
    pub description: Option<String>,
    pub use_description_hash: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceivePaymentResponse {
    pub payment_request: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedFees {
    pub fastest_fee: u64,
    pub half_hour_fee: u64,
    pub hour_fee: u64,
    pub economy_fee: u64,
    pub minimum_fee: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RefundRequest {
    pub prepared: PrepareRefundResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RefundResponse {
    // TODO
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterWebhookRequest {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterWebhookResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendBitcoinRequest {
    pub prepared: PrepareSendBitcoinResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendBitcoinResponse {
    pub payment: Payment,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLightningRequest {
    pub prepared: PrepareSendLightningResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLightningResponse {
    pub payment: Payment,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLiquidAddressRequest {
    pub prepared: PrepareSendLiquidAddressResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLiquidAddressResponse {
    pub payment: Payment,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLnurlPayRequest {
    pub prepared: PrepareSendLnurlPayResponse,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLnurlPayResponse {
    pub result: LnurlPayResult,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignMessageRequest {
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignMessageResponse {
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnregisterWebhookRequest {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnregisterWebhookResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VerifyMessageRequest {
    /// The message that was signed.
    pub message: String,
    /// The public key of the node that signed the message.
    pub pubkey: String,
    /// The zbase encoded signature to verify.
    pub signature: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VerifyMessageResponse {
    /// Boolean value indicating whether the signature covers the message and
    /// was signed by the given pubkey.
    pub is_valid: bool,
}
