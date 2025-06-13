use serde::{Deserialize, Serialize};

use crate::{lnurl::auth::LnurlAuthRequestData, network::BitcoinNetwork, utils::default_true};

/// Wrapper for the decrypted [`AesSuccessActionData`] payload
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AesSuccessActionDataDecrypted {
    /// Contents description, up to 144 characters
    pub description: String,

    /// Decrypted content
    pub plaintext: String,
}

/// Result of decryption of [`AesSuccessActionData`] payload
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum AesSuccessActionDataResult {
    Decrypted { data: AesSuccessActionDataDecrypted },
    ErrorStatus { reason: String },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bip21 {
    pub amount_sat: Option<u64>,
    pub asset_id: Option<String>,
    pub uri: String,
    pub extras: Vec<Bip21Extra>,
    pub label: Option<String>,
    pub message: Option<String>,
    pub payment_methods: Vec<RawPaymentMethod>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bip21Extra {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bip353 {
    pub address: String,
    pub bip_21: Bip21,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct BitcoinAddress {
    pub details: RawBitcoinAddress,
    pub source: PaymentRequestSource,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bolt11Invoice {
    pub details: RawBolt11Invoice,
    pub source: PaymentRequestSource,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bolt11RouteHint {
    pub hops: Vec<Bolt11RouteHintHop>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bolt11RouteHintHop {
    /// The `node_id` of the non-target end of the route
    pub src_node_id: String,
    /// The `short_channel_id` of this channel
    pub short_channel_id: String,
    /// The fees which must be paid to use this channel
    pub fees_base_msat: u32,
    pub fees_proportional_millionths: u32,

    /// The difference in CLTV values between this node and the next node.
    pub cltv_expiry_delta: u16,
    /// The minimum value, in msat, which must be relayed to the next hop.
    pub htlc_minimum_msat: Option<u64>,
    /// The maximum value in msat available for routing with a single HTLC.
    pub htlc_maximum_msat: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bolt12Invoice {
    pub details: RawBolt12Invoice,
    pub source: PaymentRequestSource,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bolt12InvoiceRequest {
    pub details: RawBolt12InvoiceRequest,
    pub source: PaymentRequestSource,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bolt12OfferBlindedPath {
    pub blinded_hops: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bolt12Offer {
    pub details: RawBolt12Offer,
    pub source: PaymentRequestSource,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum InputType {
    BitcoinAddress(BitcoinAddress),
    Bolt11Invoice(Bolt11Invoice),
    Bolt12Invoice(Bolt12Invoice),
    Bolt12InvoiceRequest(Bolt12InvoiceRequest),
    Bolt12Offer(Bolt12Offer),
    LightningAddress(LightningAddress),
    LiquidAddress(LiquidAddress),
    LnurlAuth(LnurlAuthRequestData),
    LnurlPay(LnurlPayRequest),
    LnurlWithdraw(LnurlWithdrawRequestData),
    SilentPaymentAddress(SilentPaymentAddress),
    Url(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LightningAddress {
    pub address: String,
    pub pay_request: LnurlPayRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LiquidAddress {
    pub details: RawLiquidAddress,
    pub source: PaymentRequestSource,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
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

    #[serde(skip)]
    pub url: String,

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
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LnurlWithdrawRequestData {
    pub callback: String,
    pub k1: String,
    pub default_description: String,
    /// The minimum amount, in millisats, that this LNURL-withdraw endpoint accepts
    pub min_withdrawable: u64,
    /// The maximum amount, in millisats, that this LNURL-withdraw endpoint accepts
    pub max_withdrawable: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct MessageSuccessActionData {
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PaymentRequestSource {
    pub bip_21_uri: Option<String>,
    pub bip_353_address: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
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
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RawBitcoinAddress {
    pub address: String,
    pub network: BitcoinNetwork,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RawBolt11Invoice {
    pub amount_msat: Option<u64>,
    pub description: Option<String>,
    pub description_hash: Option<String>,
    pub expiry: u64,
    pub invoice: String,
    pub min_final_cltv_expiry_delta: u64,
    pub network: BitcoinNetwork,
    pub payee_pubkey: String,
    pub payment_hash: String,
    pub payment_secret: String,
    pub routing_hints: Vec<Bolt11RouteHint>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RawBolt12Invoice {
    // TODO: Fill fields
    pub amount_msat: u64,
    pub invoice: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RawBolt12InvoiceRequest {
    // TODO: Fill fields
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RawBolt12Offer {
    pub absolute_expiry: Option<u64>,
    pub chains: Vec<String>,
    pub description: Option<String>,
    pub issuer: Option<String>,
    pub min_amount: Option<Amount>,
    pub offer: String,
    pub paths: Vec<Bolt12OfferBlindedPath>,
    pub signing_pubkey: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum RawInputType {
    Bip21(Bip21),
    Bip353(Bip353),
    Bolt12InvoiceRequest(RawBolt12InvoiceRequest),
    LnurlAuth(LnurlAuthRequestData),
    LnurlWithdraw(LnurlWithdrawRequestData),
    PaymentMethod(RawPaymentMethod),
    Url(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RawLiquidAddress {
    pub address: String,
    pub network: BitcoinNetwork,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum RawPaymentMethod {
    BitcoinAddress(RawBitcoinAddress),
    Bolt11Invoice(RawBolt11Invoice),
    Bolt12Invoice(RawBolt12Invoice),
    Bolt12Offer(RawBolt12Offer),
    LightningAddress(LightningAddress),
    LiquidAddress(RawLiquidAddress),
    LnurlPay(LnurlPayRequest),
    SilentPaymentAddress(RawSilentPaymentAddress),
}

impl RawPaymentMethod {
    pub fn get_type(&self) -> PaymentMethodType {
        match self {
            RawPaymentMethod::BitcoinAddress(_) => PaymentMethodType::BitcoinAddress,
            RawPaymentMethod::Bolt11Invoice(_) => PaymentMethodType::Bolt11Invoice,
            RawPaymentMethod::Bolt12Invoice(_) => PaymentMethodType::Bolt12Invoice,
            RawPaymentMethod::Bolt12Offer(_) => PaymentMethodType::Bolt12Offer,
            RawPaymentMethod::LightningAddress(_) => PaymentMethodType::LightningAddress,
            RawPaymentMethod::LiquidAddress(_) => PaymentMethodType::LiquidAddress,
            RawPaymentMethod::LnurlPay(_) => PaymentMethodType::LnurlPay,
            RawPaymentMethod::SilentPaymentAddress(_) => PaymentMethodType::SilentPaymentAddress,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RawSilentPaymentAddress {
    pub address: String,
    pub network: BitcoinNetwork,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SilentPaymentAddress {
    pub details: RawSilentPaymentAddress,
    pub source: PaymentRequestSource,
}

/// [`SuccessAction`] where contents are ready to be consumed by the caller
///
/// Contents are identical to [`SuccessAction`], except for AES where the ciphertext is decrypted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum SuccessActionProcessed {
    /// See [`SuccessAction::Aes`] for received payload
    ///
    /// See [`AesSuccessActionDataDecrypted`] for decrypted payload
    Aes { result: AesSuccessActionDataResult },

    /// See [`SuccessAction::Message`]
    Message { data: MessageSuccessActionData },

    /// See [`SuccessAction::Url`]
    Url { data: UrlSuccessActionData },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct UrlSuccessActionData {
    /// Contents description, up to 144 characters
    pub description: String,

    /// URL of the success action
    pub url: String,

    /// Indicates the success URL domain matches the LNURL callback domain.
    ///
    /// See <https://github.com/lnurl/luds/blob/luds/09.md>
    #[serde(default = "default_true")]
    pub matches_callback_domain: bool,
}
