use std::{collections::HashMap, u64};

use breez_sdk_input::{
    BitcoinAddress, Bolt11Invoice, Bolt12Invoice, Bolt12Offer, LightningAddress, LiquidAddress,
    LnurlAuthRequestData, LnurlPayRequest, PaymentMethod, ReceiveMethod, SilentPaymentAddress,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::lnurl::{
    LnurlErrorData,
    pay::{LnurlPayErrorData, SuccessActionProcessed},
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LightningPaymentRequest {
    pub min_amount_msat: u64,
    pub max_amount_msat: u64,
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
pub struct Payment {
    pub amount_msat: u64,
    pub created_at: u64,
    pub fee_msat: u64,
    pub fee_msat_breakdown: HashMap<String, u64>,
    pub id: String,
    pub payment_method: PaymentMethod,
    pub payment_request: String,
    pub payment_type: PaymentType,
    pub status: PaymentState,
    // TODO: Include payment details somehow. With a generic type?
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
pub struct PrepareSendBitcoinRequest {
    pub address: PaymentMethodSource<BitcoinPaymentMethod>,
    pub fee_rate_sat_per_kw: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBitcoinResponse {
    pub req: PrepareSendBitcoinRequest,
    pub fee_msat: u64,
    pub fee_msat_breakdown: HashMap<String, u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLightningRequest {
    pub payment_request: PaymentMethodSource<LightningPaymentRequest>,
    pub amount_msat: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLightningResponse {
    pub req: PrepareSendLightningRequest,
    pub fee_msat: u64,
    pub maximum_network_fee_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLnurlPayRequest {
    pub lnurl_pay: PaymentMethodSource<LnurlPaymentMethod>,
    pub amount_msat: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLnurlPayResponse {
    pub req: PrepareSendLnurlPayRequest,
    pub fee_msat: u64,
    pub maximum_network_fee_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLiquidAddressRequest {
    pub address: PaymentMethodSource<LiquidAddress>,
    pub amount_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLiquidAddressResponse {
    pub req: PrepareSendLiquidAddressRequest,
    pub fee_msat: u64,
    pub fee_msat_breakdown: HashMap<String, u64>,
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
pub struct SendLiquidAddressResponse {}

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
    EndpointError { data: LnurlErrorData },
    PayError { data: LnurlPayErrorData },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LnurlPaySuccessData {
    pub payment: Payment,
    pub success_action: Option<SuccessActionProcessed>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SourcedPaymentMethod {
    Bitcoin(PaymentMethodSource<BitcoinPaymentMethod>),
    Lightning(PaymentMethodSource<LightningPaymentRequest>),
    LnurlPay(PaymentMethodSource<LnurlPaymentMethod>),
    LiquidAddress(PaymentMethodSource<LiquidAddress>),
}

impl From<PaymentMethod> for SourcedPaymentMethod {
    fn from(value: PaymentMethod) -> Self {
        match value {
            PaymentMethod::BitcoinAddress(bitcoin_address) => SourcedPaymentMethod::Bitcoin(
                PaymentMethodSource::Plain(BitcoinPaymentMethod::BitcoinAddress(bitcoin_address)),
            ),
            PaymentMethod::Bolt11Invoice(bolt11_invoice) => SourcedPaymentMethod::Lightning(
                PaymentMethodSource::Plain(LightningPaymentRequest {
                    min_amount_msat: bolt11_invoice.amount_msat.unwrap_or(0), // TODO: Set min amount to minimum payable amount.
                    max_amount_msat: bolt11_invoice.amount_msat.unwrap_or(u64::MAX), // TODO: Set max amount to balance.
                    method: LightningPaymentMethod::Bolt11Invoice(bolt11_invoice),
                }),
            ),
            PaymentMethod::Bolt12Invoice(bolt12_invoice) => SourcedPaymentMethod::Lightning(
                PaymentMethodSource::Plain(LightningPaymentRequest {
                    min_amount_msat: bolt12_invoice.amount_msat,
                    max_amount_msat: bolt12_invoice.amount_msat,
                    method: LightningPaymentMethod::Bolt12Invoice(bolt12_invoice),
                }),
            ),
            PaymentMethod::Bolt12Offer(bolt12_offer) => SourcedPaymentMethod::Lightning(
                PaymentMethodSource::Plain(LightningPaymentRequest {
                    min_amount_msat: 0,        // TODO: Set min amount to minimum payable amount.
                    max_amount_msat: u64::MAX, // TODO: Set max amount to balance.
                    method: LightningPaymentMethod::Bolt12Offer(bolt12_offer),
                }),
            ),
            PaymentMethod::LightningAddress(lightning_address) => SourcedPaymentMethod::LnurlPay(
                PaymentMethodSource::Plain(LnurlPaymentMethod::LightningAddress(lightning_address)),
            ),
            PaymentMethod::LiquidAddress(liquid_address) => {
                SourcedPaymentMethod::LiquidAddress(PaymentMethodSource::Plain(liquid_address))
            }
            PaymentMethod::LnurlPay(lnurl_pay_request) => SourcedPaymentMethod::LnurlPay(
                PaymentMethodSource::Plain(LnurlPaymentMethod::LnurlPay(lnurl_pay_request)),
            ),
            PaymentMethod::SilentPaymentAddress(silent_payment_address) => {
                SourcedPaymentMethod::Bitcoin(PaymentMethodSource::Plain(
                    BitcoinPaymentMethod::SilentPaymentAddress(silent_payment_address),
                ))
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SourcedInputType {
    LnurlAuth(LnurlAuthRequestData),
    PaymentMethod(SourcedPaymentMethod),
    ReceiveMethod(ReceiveMethod),
    Url(String),
}
