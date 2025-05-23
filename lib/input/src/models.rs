use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::network::Network;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Amount {
    Bitcoin {
        amount_msat: u64,
    },
    /// An amount of currency specified using ISO 4712.
    Currency {
        /// The currency that the amount is denominated in.
        iso4217_code: String,
        /// The amount in the currency unit adjusted by the ISO 4712 exponent (e.g., USD cents).
        fractional_amount: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bip21 {
    pub amount_sat: Option<u64>,
    pub asset_id: Option<String>,
    pub bip_21: String,
    pub extras: HashMap<String, String>,
    pub label: Option<String>,
    pub message: Option<String>,
    pub payment_methods: Vec<PaymentMethod>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bip353 {
    pub address: String,
    pub bip_21: Bip21,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BitcoinAddress {
    pub address: String,
    pub network: Network,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bolt11Invoice {
    pub bolt11: String,
    pub network: Network,
    pub payee_pubkey: String,
    pub payment_hash: String,
    pub description: Option<String>,
    pub description_hash: Option<String>,
    pub amount_msat: Option<u64>,
    pub timestamp: u64,
    pub expiry: u64,
    pub routing_hints: Vec<Bolt11RouteHint>,
    pub payment_secret: Vec<u8>,
    pub min_final_cltv_expiry_delta: u64,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bolt11RouteHint {
    pub hops: Vec<Bolt11RouteHintHop>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bolt11RouteHintHop {
    /// The node_id of the non-target end of the route
    pub src_node_id: String,
    /// The short_channel_id of this channel
    pub short_channel_id: String,
    /// The fees which must be paid to use this channel
    pub fees_base_msat: u32,
    pub fees_proportional_millionths: u32,

    /// The difference in CLTV values between this node and the next node.
    pub cltv_expiry_delta: u64,
    /// The minimum value, in msat, which must be relayed to the next hop.
    pub htlc_minimum_msat: Option<u64>,
    /// The maximum value in msat available for routing with a single HTLC.
    pub htlc_maximum_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bolt12Invoice {
    // TODO: Fill fields
    pub amount_msat: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bolt12InvoiceRequest {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bolt12Offer {
    /// String representation of the Bolt12 offer
    pub offer: String,
    pub chains: Vec<String>,
    /// If set, it represents the minimum amount that an invoice must have to be valid for this offer
    pub min_amount: Option<Amount>,
    pub description: Option<String>,
    /// Epoch time from which an invoice should no longer be requested. If None, the offer does not expire.
    pub absolute_expiry: Option<u64>,
    pub issuer: Option<String>,
    /// The public key used by the recipient to sign invoices.
    pub signing_pubkey: Option<String>,
    pub paths: Vec<Bolt12OfferBlindedPath>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Bolt12OfferBlindedPath {
    /// For each blinded hop, we store the node ID (pubkey as hex).
    pub blinded_hops: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LightningAddress {
    pub address: String,
    pub pay_request: LnurlPayRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LiquidAddress {
    pub address: String,
    pub network: Network,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LnurlAuthRequestData {
    /// Hex encoded 32 bytes of challenge
    pub k1: String,

    /// When available, one of: register, login, link, auth
    pub action: Option<String>,

    /// Indicates the domain of the LNURL-auth service, to be shown to the user when asking for
    /// auth confirmation, as per LUD-04 spec.
    #[serde(skip_serializing, skip_deserializing)]
    pub domain: String,

    /// Indicates the URL of the LNURL-auth service, including the query arguments. This will be
    /// extended with the signed challenge and the linking key, then called in the second step of the workflow.
    #[serde(skip_serializing, skip_deserializing)]
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LnurlPayRequest {
    pub callback: String,
    /// The minimum amount, in millisats, that this LNURL-pay endpoint accepts
    pub min_sendable: u64,
    /// The maximum amount, in millisats, that this LNURL-pay endpoint accepts
    pub max_sendable: u64,
    /// As per LUD-06, `metadata` is a raw string (e.g. a json representation of the inner map).
    /// Use `metadata_vec()` to get the parsed items.
    #[serde(rename(deserialize = "metadata"))]
    pub metadata_str: String,
    /// The comment length accepted by this endpoint
    ///
    /// See <https://github.com/lnurl/luds/blob/luds/12.md>
    #[serde(default)]
    pub comment_allowed: u16,

    /// Indicates the domain of the LNURL-pay service, to be shown to the user when asking for
    /// payment input, as per LUD-06 spec.
    ///
    /// Note: this is not the domain of the callback, but the domain of the LNURL-pay endpoint.
    #[serde(skip)]
    pub domain: String,

    /// Value indicating whether the recipient supports Nostr Zaps through NIP-57.
    ///
    /// See <https://github.com/nostr-protocol/nips/blob/master/57.md>
    #[serde(default)]
    pub allows_nostr: bool,

    /// Optional recipient's lnurl provider's Nostr pubkey for NIP-57. If it exists it should be a
    /// valid BIP 340 public key in hex.
    ///
    /// See <https://github.com/nostr-protocol/nips/blob/master/57.md>
    /// See <https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki>
    pub nostr_pubkey: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LnurlWithdrawRequestData {
    pub callback: String,
    pub k1: String,
    pub default_description: String,
    /// The minimum amount, in millisats, that this LNURL-withdraw endpoint accepts
    pub min_withdrawable: u64,
    /// The maximum amount, in millisats, that this LNURL-withdraw endpoint accepts
    pub max_withdrawable: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum InputType {
    LnurlAuth(LnurlAuthRequestData),
    PaymentRequest(PaymentRequest),
    ReceiveMethod(ReceiveMethod),
    Url(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaymentRequest {
    Bip21(Bip21),
    Bip353(Bip353),
    Plain(PaymentMethod),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaymentMethod {
    BitcoinAddress(BitcoinAddress),
    Bolt11Invoice(Bolt11Invoice),
    Bolt12Invoice(Bolt12Invoice),
    Bolt12Offer(Bolt12Offer),
    LightningAddress(LightningAddress),
    LiquidAddress(LiquidAddress),
    LnurlPay(LnurlPayRequest),
    SilentPaymentAddress(SilentPaymentAddress),
}

impl PaymentMethod {
    pub fn get_type(&self) -> PaymentMethodType {
        match self {
            PaymentMethod::BitcoinAddress(_) => PaymentMethodType::BitcoinAddress,
            PaymentMethod::Bolt11Invoice(_) => PaymentMethodType::Bolt11Invoice,
            PaymentMethod::Bolt12Invoice(_) => PaymentMethodType::Bolt12Invoice,
            PaymentMethod::Bolt12Offer(_) => PaymentMethodType::Bolt12Offer,
            PaymentMethod::LightningAddress(_) => PaymentMethodType::LightningAddress,
            PaymentMethod::LiquidAddress(_) => PaymentMethodType::LiquidAddress,
            PaymentMethod::LnurlPay(_) => PaymentMethodType::LnurlPay,
            PaymentMethod::SilentPaymentAddress(_) => PaymentMethodType::SilentPaymentAddress,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PaymentMethodType {
    BitcoinAddress,
    Bolt11Invoice,
    Bolt12Invoice,
    Bolt12Offer,
    LightningAddress,
    LiquidAddress,
    LnurlPay,
    SilentPaymentAddress,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ReceiveMethod {
    Bolt12InvoiceRequest(Bolt12InvoiceRequest),
    LnurlWithdraw(LnurlWithdrawRequestData),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SilentPaymentAddress {
    pub address: String,
    pub network: Network,
}
