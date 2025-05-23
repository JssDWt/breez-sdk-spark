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
pub struct Bip21Source<P> {
    pub bip_21_uri: String,
    pub payment_method: P,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bip353Source<P> {
    pub bip_353_uri: String,
    pub bip_21: Bip21Source<P>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BitcoinPaymentMethod {
    BitcoinAddress(BitcoinAddress),
    SilentPaymentAddress(SilentPaymentAddress),
}

#[async_trait]
pub trait BreezServices<A, D, F> {
    async fn parse_and_pick(&self, input: &str) -> Result<SourcedInputType<A>, ParseAndPickError>;
    async fn pick_payment_method(
        &self,
        payment_request: PaymentRequest,
    ) -> Result<SourcedPaymentMethod<A>, PickPaymentMethodError>;
    async fn prepare_send_bitcoin(
        &self,
        req: PrepareSendBitcoinRequest,
    ) -> Result<PrepareSendBitcoinResponse<A, F>, PrepareSendBitcoinError>;
    async fn prepare_send_lightning(
        &self,
        req: PrepareSendLightningRequest<A>,
    ) -> Result<PrepareSendLightningResponse<A, F>, PrepareSendLightningError>;
    async fn prepare_send_lnurl_pay(
        &self,
        req: PrepareSendLnurlPayRequest<A>,
    ) -> Result<PrepareSendLnurlPayResponse<A, F>, PrepareSendLnurlPayError>;
    async fn prepare_send_liquid_address(
        &self,
        req: PrepareSendLiquidAddressRequest<A>,
    ) -> Result<PrepareSendLiquidAddressResponse<A, F>, PrepareSendLiquidAddressError>;
    async fn prepare_receive_payment(
        &self,
        req: PrepareReceivePaymentRequest<A>,
    ) -> Result<PrepareReceivePaymentResponse<A>, PrepareReceivePaymentError>;
    async fn receive_payment(
        &self,
        req: ReceivePaymentRequest<A>,
    ) -> Result<ReceivePaymentResponse, ReceivePaymentError>;
    async fn send_bitcoin(
        &self,
        req: SendBitcoinRequest<A, F>,
    ) -> Result<SendBitcoinResponse<A, D, F>, SendBitcoinError>;
    async fn send_lightning(
        &self,
        req: SendLightningRequest<A, F>,
    ) -> Result<SendLightningResponse<A, D, F>, SendLightningError>;
    async fn send_lnurl_pay(
        &self,
        req: SendLnurlPayRequest<A, F>,
    ) -> Result<SendLnurlPayResponse<A, D, F>, SendLnurlPayError>;
    async fn send_liquid_address(
        &self,
        req: SendLiquidAddressRequest<A, F>,
    ) -> Result<SendLiquidAddressResponse<A, D, F>, SendLiquidAddressError>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LightningPaymentRequest<A> {
    pub min_amount: A,
    pub max_amount: A,
    pub method: LightningPaymentMethod,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LightningPaymentMethod {
    Bolt11Invoice(Bolt11Invoice),
    Bolt12Invoice(Bolt12Invoice),
    Bolt12Offer(Bolt12Offer),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LnurlPaymentMethod {
    LnurlPay(LnurlPayRequest),
    LightningAddress(LightningAddress),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Payment<A, D, F> {
    pub amount: A,
    pub created_at: u64,
    pub fee: A,
    pub fee_breakdown: F,
    pub id: String,
    pub payment_method: PaymentMethod,
    pub payment_request: String,
    pub payment_type: PaymentType,
    pub status: PaymentState,
    pub details: D,
}

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
pub enum PaymentMethodSource<P> {
    Bip21(Bip21Source<P>),
    Bip353(Bip353Source<P>),
    Plain(P),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareReceivePaymentRequest<A> {
    pub amount: A,
    pub receive_method: ReceiveMethod,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareReceivePaymentResponse<A> {
    pub req: PrepareReceivePaymentRequest<A>,
    pub fee: A,
    pub min_payer_amount: A,
    pub max_payer_amount: A,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBitcoinRequest {
    pub address: PaymentMethodSource<BitcoinPaymentMethod>,
    pub fee_rate_sat_per_kw: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBitcoinResponse<A, F> {
    pub req: PrepareSendBitcoinRequest,
    pub fee: A,
    pub fee_breakdown: F,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLightningRequest<A> {
    pub payment_request: PaymentMethodSource<LightningPaymentRequest<A>>,
    pub amount: A,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLightningResponse<A, F> {
    pub req: PrepareSendLightningRequest<A>,
    pub fee: A,
    pub fee_breakdown: F,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLnurlPayRequest<A> {
    pub lnurl_pay: PaymentMethodSource<LnurlPaymentMethod>,
    pub amount: A,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLnurlPayResponse<A, F> {
    pub req: PrepareSendLnurlPayRequest<A>,
    pub fee: A,
    pub fee_breakdown: F,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLiquidAddressRequest<A> {
    pub address: PaymentMethodSource<LiquidAddress>,
    pub amount: Option<A>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLiquidAddressResponse<A, F> {
    pub req: PrepareSendLiquidAddressRequest<A>,
    pub fee: A,
    pub fee_breakdown: F,
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Display, EnumString, Eq, Hash, PartialEq, Serialize,
)]
#[strum(serialize_all = "lowercase")]
pub enum ReceiveMethod {
    BitcoinAddress,
    #[default]
    Bolt11Invoice,
    Bolt12Offer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceivePaymentRequest<A> {
    pub prepared: PrepareReceivePaymentResponse<A>,
    pub description: Option<String>,
    pub use_description_hash: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReceivePaymentResponse {
    pub payment_request: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendBitcoinRequest<A, F> {
    pub prepared: PrepareSendBitcoinResponse<A, F>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendBitcoinResponse<A, D, F> {
    pub payment: Payment<A, D, F>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLightningRequest<A, F> {
    pub prepared: PrepareSendLightningResponse<A, F>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLightningResponse<A, D, F> {
    pub payment: Payment<A, D, F>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLiquidAddressRequest<A, F> {
    pub prepared: PrepareSendLiquidAddressResponse<A, F>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLiquidAddressResponse<A, D, F> {
    pub payment: Payment<A, D, F>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLnurlPayRequest<A, F> {
    pub prepared: PrepareSendLnurlPayResponse<A, F>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendLnurlPayResponse<A, D, F> {
    pub result: LnurlPayResult<A, D, F>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LnurlPayResult<A, D, F> {
    EndpointSuccess(LnurlPaySuccessData<A, D, F>),
    EndpointError { data: LnurlErrorData },
    PayError { data: LnurlPayErrorData },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LnurlPaySuccessData<A, D, F> {
    pub payment: Payment<A, D, F>,
    pub success_action: Option<SuccessActionProcessed>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SourcedPaymentMethod<A> {
    Bitcoin(PaymentMethodSource<BitcoinPaymentMethod>),
    Lightning(PaymentMethodSource<LightningPaymentRequest<A>>),
    LnurlPay(PaymentMethodSource<LnurlPaymentMethod>),
    LiquidAddress(PaymentMethodSource<LiquidAddress>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SourcedInputType<A> {
    LnurlAuth(LnurlAuthRequestData),
    PaymentMethod(SourcedPaymentMethod<A>),
    ReceiveRequest(ReceiveRequest),
    Url(String),
}
