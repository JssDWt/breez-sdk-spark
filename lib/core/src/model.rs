use breez_sdk_input::{
    BitcoinAddress, Bolt11Invoice, Bolt12Invoice, Bolt12Offer, LightningAddress, LiquidAddress,
    LnurlAuthRequestData, LnurlPayRequest, PaymentMethod, PaymentRequest, ReceiveRequest,
    SilentPaymentAddress,
};
use breez_sdk_macros::async_trait;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::{
    error::{
        ParseAndPickError, PickPaymentMethodError, PrepareReceivePaymentError,
        PrepareSendBitcoinError, PrepareSendLightningError, PrepareSendLiquidAddressError,
        PrepareSendLnurlPayError, ReceivePaymentError, SendBitcoinError, SendLightningError,
        SendLiquidAddressError, SendLnurlPayError,
    },
    lnurl::{
        LnurlErrorData,
        pay::{LnurlPayErrorData, SuccessActionProcessed},
    },
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BitcoinPaymentMethod {
    BitcoinAddress(BitcoinAddress),
    SilentPaymentAddress(SilentPaymentAddress),
}

#[async_trait]
pub trait BreezServices {
    async fn parse_and_pick(&self, input: &str) -> Result<PickedInputType, ParseAndPickError>;
    async fn pick_payment_method(
        &self,
        payment_request: PaymentRequest,
    ) -> Result<PickedPaymentMethod, PickPaymentMethodError>;
    async fn prepare_send_bitcoin(
        &self,
        req: PrepareSendBitcoinRequest,
    ) -> Result<PrepareSendBitcoinResponse, PrepareSendBitcoinError>;
    async fn prepare_send_lightning(
        &self,
        req: PrepareSendLightningRequest,
    ) -> Result<PrepareSendLightningResponse, PrepareSendLightningError>;
    async fn prepare_send_lnurl_pay(
        &self,
        req: PrepareSendLnurlPayRequest,
    ) -> Result<PrepareSendLnurlPayResponse, PrepareSendLnurlPayError>;
    async fn prepare_send_liquid_address(
        &self,
        req: PrepareSendLiquidAddressRequest,
    ) -> Result<PrepareSendLiquidAddressResponse, PrepareSendLiquidAddressError>;
    async fn prepare_receive_payment(
        &self,
        req: PrepareReceivePaymentRequest,
    ) -> Result<PrepareReceivePaymentResponse, PrepareReceivePaymentError>;
    async fn receive_payment(
        &self,
        req: ReceivePaymentRequest,
    ) -> Result<ReceivePaymentResponse, ReceivePaymentError>;
    async fn send_bitcoin(
        &self,
        req: SendBitcoinRequest,
    ) -> Result<SendBitcoinResponse, SendBitcoinError>;
    async fn send_lightning(
        &self,
        req: SendLightningRequest,
    ) -> Result<SendLightningResponse, SendLightningError>;
    async fn send_lnurl_pay(
        &self,
        req: SendLnurlPayRequest,
    ) -> Result<SendLnurlPayResponse, SendLnurlPayError>;
    async fn send_liquid_address(
        &self,
        req: SendLiquidAddressRequest,
    ) -> Result<SendLiquidAddressResponse, SendLiquidAddressError>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FeeBreakdown {} // TODO: This type may vary across different SDKs.

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

// TODO: Create easier interface for lnurl pay
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LnurlPaymentMethod {
    LnurlPay(LnurlPayRequest),
    LightningAddress(LightningAddress),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MilliSatoshi(pub u64); // TODO: This type may vary across different SDKs. It may include assets in liquid for example.

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Payment {
    pub amount: MilliSatoshi,
    pub created_at: u64,
    pub fee: MilliSatoshi,
    pub fee_breakdown: FeeBreakdown,
    pub id: String,
    pub payment_method: PaymentMethod,
    pub payment_request: String,
    pub payment_type: PaymentType,
    pub status: PaymentState,
    pub details: PaymentDetails,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaymentDetails {} // TODO: This type may vary across different SDKs.

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
