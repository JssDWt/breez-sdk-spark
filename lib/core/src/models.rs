use std::collections::HashMap;

use breez_sdk_input::{
    BitcoinAddress, Bolt11Invoice, Bolt12Invoice, Bolt12Offer, LightningAddress, LiquidAddress,
    LnurlAuthRequestData, LnurlPayRequest, PaymentMethod, ReceiveMethod, SilentPaymentAddress,
};
use serde::{Deserialize, Serialize};

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
    pub bolt11: PaymentMethodSource<LightningPaymentMethod>,
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
pub enum SourcedPaymentMethod {
    Bitcoin(PaymentMethodSource<BitcoinPaymentMethod>),
    Lightning(PaymentMethodSource<LightningPaymentMethod>),
    LnurlPay(PaymentMethodSource<LnurlPaymentMethod>),
    LiquidAddress(PaymentMethodSource<LiquidAddress>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BitcoinPaymentMethod {
    BitcoinAddress(BitcoinAddress),
    SilentPaymentAddress(SilentPaymentAddress),
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

impl From<PaymentMethod> for SourcedPaymentMethod {
    fn from(value: PaymentMethod) -> Self {
        match value {
            PaymentMethod::BitcoinAddress(bitcoin_address) => SourcedPaymentMethod::Bitcoin(
                PaymentMethodSource::Plain(BitcoinPaymentMethod::BitcoinAddress(bitcoin_address)),
            ),
            PaymentMethod::Bolt11Invoice(bolt11_invoice) => SourcedPaymentMethod::Lightning(
                PaymentMethodSource::Plain(LightningPaymentMethod::Bolt11Invoice(bolt11_invoice)),
            ),
            PaymentMethod::Bolt12Invoice(bolt12_invoice) => SourcedPaymentMethod::Lightning(
                PaymentMethodSource::Plain(LightningPaymentMethod::Bolt12Invoice(bolt12_invoice)),
            ),
            PaymentMethod::Bolt12Offer(bolt12_offer) => SourcedPaymentMethod::Lightning(
                PaymentMethodSource::Plain(LightningPaymentMethod::Bolt12Offer(bolt12_offer)),
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
