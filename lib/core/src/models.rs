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
pub struct PrepareSendBitcoinAddressRequest {
    pub bitcoin_address: PaymentMethodSource<BitcoinAddress>,
    pub fee_rate_sat_per_kw: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBitcoinAddressResponse {
    pub req: PrepareSendBitcoinAddressRequest,
    pub fee_msat: u64,
    pub fee_msat_breakdown: HashMap<String, u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBolt11InvoiceRequest {
    pub bolt11: PaymentMethodSource<Bolt11Invoice>,
    pub amount_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBolt11InvoiceResponse {
    pub req: PrepareSendBolt11InvoiceRequest,
    pub fee_msat: u64,
    pub maximum_network_fee_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBolt12InvoiceRequest {
    pub bolt12_invoice: PaymentMethodSource<Bolt12Invoice>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBolt12InvoiceResponse {
    pub req: PrepareSendBolt12InvoiceRequest,
    pub fee_msat: u64,
    pub maximum_network_fee_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBolt12OfferRequest {
    pub offer: PaymentMethodSource<Bolt12Offer>,
    pub amount_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendBolt12OfferResponse {
    pub req: PrepareSendBolt12OfferRequest,
    pub fee_msat: u64,
    pub maximum_network_fee_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLightningAddressRequest {
    pub address: PaymentMethodSource<LightningAddress>,
    pub amount_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLightningAddressResponse {
    pub req: PrepareSendLightningAddressRequest,
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
pub struct PrepareSendLnurlPayRequest {
    pub request: PaymentMethodSource<LnurlPayRequest>,
    pub amount_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrepareSendLnurlPayResponse {
    pub req: PrepareSendLnurlPayRequest,
    pub fee_msat: u64,
    pub maximum_network_fee_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SourcedPaymentMethod {
    BitcoinAddress(PaymentMethodSource<BitcoinAddress>),
    Bolt11Invoice(PaymentMethodSource<Bolt11Invoice>),
    Bolt12Invoice(PaymentMethodSource<Bolt12Invoice>),
    Bolt12Offer(PaymentMethodSource<Bolt12Offer>),
    LightningAddress(PaymentMethodSource<LightningAddress>),
    LiquidAddress(PaymentMethodSource<LiquidAddress>),
    LnurlPay(PaymentMethodSource<LnurlPayRequest>),
    SilentPaymentAddress(PaymentMethodSource<SilentPaymentAddress>),
}

impl From<PaymentMethod> for SourcedPaymentMethod {
    fn from(value: PaymentMethod) -> Self {
        match value {
            PaymentMethod::BitcoinAddress(bitcoin_address) => {
                SourcedPaymentMethod::BitcoinAddress(PaymentMethodSource::Plain(bitcoin_address))
            }
            PaymentMethod::Bolt11Invoice(bolt11_invoice) => {
                SourcedPaymentMethod::Bolt11Invoice(PaymentMethodSource::Plain(bolt11_invoice))
            }
            PaymentMethod::Bolt12Invoice(bolt12_invoice) => {
                SourcedPaymentMethod::Bolt12Invoice(PaymentMethodSource::Plain(bolt12_invoice))
            }
            PaymentMethod::Bolt12Offer(bolt12_offer) => {
                SourcedPaymentMethod::Bolt12Offer(PaymentMethodSource::Plain(bolt12_offer))
            }
            PaymentMethod::LightningAddress(lightning_address) => {
                SourcedPaymentMethod::LightningAddress(PaymentMethodSource::Plain(
                    lightning_address,
                ))
            }
            PaymentMethod::LiquidAddress(liquid_address) => {
                SourcedPaymentMethod::LiquidAddress(PaymentMethodSource::Plain(liquid_address))
            }
            PaymentMethod::LnurlPay(lnurl_pay_request) => {
                SourcedPaymentMethod::LnurlPay(PaymentMethodSource::Plain(lnurl_pay_request))
            }
            PaymentMethod::SilentPaymentAddress(silent_payment_address) => {
                SourcedPaymentMethod::SilentPaymentAddress(PaymentMethodSource::Plain(
                    silent_payment_address,
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
