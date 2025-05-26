use lightning::bolt11_invoice::Bolt11InvoiceDescriptionRef;
use serde::Deserialize;
use tracing::{debug, error};

use crate::{
    dns_resolver, error::{ServiceConnectivityError, ServiceConnectivityErrorKind}, input::{ParseError, PaymentMethod, PaymentRequest, PaymentRequestSource}, utils::{ReqwestRestClient, RestClient}
};

use super::{error::ParseResult, Bip21, Bolt11RouteHint, Bolt11RouteHintHop, Bolt12InvoiceRequest, Bolt12Offer, Bolt12OfferBlindedPath, DetailedBolt11Invoice, DetailedBolt12Invoice, DetailedBolt12Offer, InputType, LightningAddress, LnurlAuthRequestData, LnurlErrorData, LnurlPayRequest, LnurlWithdrawRequestData, ReceiveRequest};

const BIP_21_PREFIX: &str = "bitcoin:";
const BIP_21_PREFIX_LEN: usize = BIP_21_PREFIX.len();
const BIP_353_USER_BITCOIN_PAYMENT_PREFIX: &str = "user._bitcoin-payment";
const LIGHTNING_PREFIX: &str = "lightning:";
const LIGHTNING_PREFIX_LEN: usize = LIGHTNING_PREFIX.len();
const LNURL_HRP: &str = "lnurl";

pub async fn parse(input: &str) -> ParseResult<InputType> {
    InputParser::new(ReqwestRestClient::new()?)
        .parse(input)
        .await
}

pub struct InputParser<C> {
    rest_client: C,
}

impl<C> InputParser<C>
where
    C: RestClient + Send + Sync,
{
    pub fn new(rest_client: C) -> Self {
        InputParser { rest_client }
    }

    pub async fn parse(&self, input: &str) -> ParseResult<InputType> {
        let input = input.trim();
        if input.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        if input.contains('@') {
            if let Some(bip_21) = self.parse_bip_353(input).await {
                return Ok(InputType::PaymentRequest(PaymentRequest::Bip21(bip_21)));
            }

            if let Some(lightning_address) = self.parse_lightning_address(input).await {
                return Ok(InputType::PaymentRequest(PaymentRequest::PaymentMethod(
                    PaymentMethod::LightningAddress(lightning_address),
                )));
            }
        }

        if has_bip_21_prefix(input) {
            let source = PaymentRequestSource {
                bip_21_uri: Some(input.to_string()),
                bip_353_address: None,
            };
            if let Some(bip_21) = self.parse_bip_21(input, source).await {
                return Ok(InputType::PaymentRequest(PaymentRequest::Bip21(bip_21)));
            }
        }

        let source = PaymentRequestSource::default();
        if let Some(input_type) = self.parse_lightning(input, &source).await {
            return Ok(input_type);
        }

        if let Some(input_type) = self.parse_bitcoin(input, &source).await {
            return Ok(input_type);
        }

        Err(ParseError::InvalidInput)
    }

    async fn parse_bip_21(&self, input: &str, source: PaymentRequestSource) -> Option<Bip21> {
        todo!()
    }

    async fn parse_bip_353(&self, input: &str) -> Option<Bip21> {
        // BIP-353 addresses may have a ₿ prefix, so strip it if present
        let (local_part, domain) = input.strip_prefix('₿').unwrap_or(input).split_once('@')?;

        // Validate both parts are within the DNS label size limit.
        // See <https://datatracker.ietf.org/doc/html/rfc1035#section-2.3.4>
        if local_part.len() > 63 || domain.len() > 63 {
            return None;
        }

        // Query for TXT records of a domain
        let dns_name = format!(
            "{}.{}.{}",
            local_part, BIP_353_USER_BITCOIN_PAYMENT_PREFIX, domain
        );
        let records = match dns_resolver::txt_lookup(dns_name).await {
            Ok(records) => records,
            Err(e) => {
                debug!("No BIP353 TXT records found: {}", e);
                return None;
            }
        };

        let bip_21 = extract_bip353_record(records)?;
        self.parse_bip_21(
            &bip_21,
            PaymentRequestSource {
                bip_21_uri: Some(bip_21.clone()),
                bip_353_address: Some(input.to_string()),
            },
        )
        .await
    }

    async fn parse_bitcoin(&self, input: &str, source: &PaymentRequestSource) -> Option<InputType> {
        todo!()
    }

    async fn parse_lightning(
        &self,
        input: &str,
        source: &PaymentRequestSource,
    ) -> Option<InputType> {
        let input = match has_lightning_prefix(input) {
            true => &input[LIGHTNING_PREFIX_LEN..], // Strip the lightning: prefix regardless of case
            false => input,
        };

        if let Some(bolt11) = parse_bolt11(input, source) {
            return Some(InputType::PaymentRequest(PaymentRequest::PaymentMethod(
                PaymentMethod::Bolt11Invoice(bolt11),
            )));
        }

        if let Some(bolt12_offer) = parse_bolt12_offer(input, source) {
            return Some(InputType::PaymentRequest(PaymentRequest::PaymentMethod(
                PaymentMethod::Bolt12Offer(bolt12_offer),
            )));
        }

        if let Some(bolt12_invoice) = parse_bolt12_invoice(input, source) {
            return Some(InputType::PaymentRequest(PaymentRequest::PaymentMethod(
                PaymentMethod::Bolt12Invoice(bolt12_invoice),
            )));
        }

        if let Some(bolt12_invoice_request) = parse_bolt12_invoice_request(input, source) {
            return Some(InputType::ReceiveRequest(
                crate::input::ReceiveRequest::Bolt12InvoiceRequest(bolt12_invoice_request),
            ));
        }

        if let Some(lnurl) = self.parse_lnurl(input, source).await {
            return Some(lnurl);
        }

        None
    }

    async fn parse_lightning_address(&self, input: &str) -> Option<LightningAddress> {
        if !input.contains('@') {
            return None;
        }

        let (user, domain) = input.strip_prefix('₿').unwrap_or(input).split_once('@')?;

        // It is safe to downcase the domains since they are case-insensitive.
        // https://www.rfc-editor.org/rfc/rfc3986#section-3.2.2
        let (user, domain) = (user.to_lowercase(), domain.to_lowercase());
        
        if !user
            .chars()
            .all(|c| c.is_alphanumeric() || ['-', '_', '.'].contains(&c))
        {
            return None;
        }

        let scheme = match domain.ends_with(".onion") {
            true => "http://",
            false => "https://",
        };

        let url = match reqwest::Url::parse(&format!("{scheme}{domain}/.well-known/lnurlp/{user}"))
        {
            Ok(url) => url,
            Err(_) => return None, // TODO: log or return error.
        };

        let input_type = match self.resolve_lnurl(&url, &PaymentRequestSource::default()).await {
            Some(lnurl) => lnurl,
            None => return None, // TODO: log or return error.
        };

        let address = format!("{user}@{domain}");
        match input_type {
            InputType::PaymentRequest(PaymentRequest::PaymentMethod(PaymentMethod::LnurlPay(
                pay_request,
            ))) => Some(LightningAddress {
                address,
                pay_request,
            }),
            _ => None, // TODO: log or return error.
        }
    }

    async fn parse_lnurl(&self, input: &str, source: &PaymentRequestSource) -> Option<InputType> {
        let mut input = match bech32::decode(input) {
            Ok((hrp, data)) => {
                let hrp = hrp.to_lowercase();
                if hrp != LNURL_HRP {
                    return None;
                }
                let decoded = match String::from_utf8(data) {
                    Ok(decoded) => decoded,
                    Err(_) => return None,
                };

                decoded
            }
            Err(_) => input.to_string(),
        };

        let lowercase = input.to_lowercase();
        let supported_prefixes = ["lnurlp", "lnurlw", "keyauth"];

        // Treat prefix: and prefix:// the same, to cover both vendor implementations
        // https://github.com/lnbits/lnbits/pull/762#issue-1309702380
        for pref in supported_prefixes {
            let scheme_simple = &format!("{pref}:");
            let scheme_simple_len = scheme_simple.len();
            let scheme_authority = format!("{pref}://");
            if lowercase.starts_with(scheme_simple) && !lowercase.starts_with(&scheme_authority) {
                let mut fixed = scheme_authority;
                fixed.push_str(&lowercase[scheme_simple_len..]);
                input = fixed;
                break;
            }
        }

        let parsed_url = match reqwest::Url::parse(&input) {
            Ok(url) => url,
            Err(_) => return None, // TODO: log or return error.
        };

        let domain = match parsed_url.domain() {
            Some(domain) => domain,
            None => return None, // TODO: log or return error.
        };

        let mut url = parsed_url.clone();
        match parsed_url.scheme() {
            "http" => {
                if !domain.ends_with(".onion") {
                    // TODO: log or return error.
                    return None;
                }
            }
            "https" => {
                if domain.ends_with(".onion") {
                    // TODO: log or return error.
                    return None;
                }
            }
            "lnurlp" | "lnurlw" | "keyauth" => {
                if domain.ends_with(".onion") {
                    url.set_scheme("http").ok()?;
                } else {
                    url.set_scheme("https").ok()?;
                }
            }
            &_ => return None, // TODO: log or return error.
        }

        self.resolve_lnurl(&url, source).await
    }

    async fn resolve_lnurl(
        &self,
        url: &reqwest::Url,
        source: &PaymentRequestSource,
    ) -> Option<InputType> {
        if let Some(query) = url.query() {
            if query.contains("tag=login") {
                let data = self.validate_lnurl_request(url).await?;
                return Some(InputType::LnurlAuth(data));
            }
        }

        let (response, _) = self.rest_client.get(&url.to_string()).await?;
        let lnurl_data: LnurlRequestData = parse_json(&response)?;
        let domain = url
            .domain()
            .ok_or(LnurlError::MissingDomain)?
            .to_string();
        let ln_address = url.to_string();
        Some(match lnurl_data {
            LnurlRequestData::PayRequest { data } => InputType::PaymentRequest(PaymentRequest::PaymentMethod(PaymentMethod::LnurlPay(LnurlPayRequest { domain, ..data }))),
            LnurlRequestData::WithdrawRequest { data } => InputType::ReceiveRequest(ReceiveRequest::LnurlWithdraw(data)),
            LnurlRequestData::AuthRequest { data } => InputType::LnurlAuth(data),
            LnurlRequestData::Error { data } => todo!(),
        })
    }

    async fn validate_lnurl_request(
        &self,
        url: &reqwest::Url,
    ) -> Result<LnurlAuthRequestData, LnurlError> {
        let query_pairs = url.query_pairs();

        let k1 = query_pairs
            .into_iter()
            .find(|(key, _)| key == "k1")
            .map(|(_, v)| v.to_string())
            .ok_or(LnurlError::MissingK1)?;

        let maybe_action = query_pairs
            .into_iter()
            .find(|(key, _)| key == "action")
            .map(|(_, v)| v.to_string());

        let k1_bytes =
            hex::decode(&k1).map_err(|e| LnurlError::InvalidK1)?;
        if k1_bytes.len() != 32 {
            return Err(LnurlError::InvalidK1);
        }

        if let Some(action) = &maybe_action {
            if !["register", "login", "link", "auth"].contains(&action.as_str()) {
                return Err(LnurlError::UnsupportedAction);
            }
        }

        Ok(LnurlAuthRequestData {
            k1,
            action: maybe_action,
            domain: url.domain().ok_or(LnurlError::MissingDomain)?.to_string(),
            url: url.to_string(),
        })
    }
}

enum LnurlError {
    MissingK1,
    InvalidK1,
    UnsupportedAction,
    MissingDomain,
}
fn format_short_channel_id(id: u64) -> String {
    let block_num = (id >> 40) as u32;
    let tx_num = ((id >> 16) & 0xFFFFFF) as u32;
    let tx_out = (id & 0xFFFF) as u16;
    format!("{block_num}x{tx_num}x{tx_out}")
}

fn has_bip_21_prefix(input: &str) -> bool {
    has_prefix(input, BIP_21_PREFIX)
}

fn has_lightning_prefix(input: &str) -> bool {
    has_prefix(input, LIGHTNING_PREFIX)
}

fn has_prefix(input: &str, prefix: &str) -> bool {
    input
        .to_lowercase()
        .starts_with(prefix.to_lowercase().as_str())
}

fn extract_bip353_record(records: Vec<String>) -> Option<String> {
    let bip353_record = records
        .into_iter()
        .filter(|record| has_bip_21_prefix(record))
        .collect::<Vec<String>>();

    if bip353_record.len() > 1 {
        error!(
            "Invalid decoded TXT data. Multiple records found ({})",
            bip353_record.len()
        );

        return None;
    }

    bip353_record.into_iter().nth(0)
}

fn parse_bolt11(input: &str, source: &PaymentRequestSource) -> Option<DetailedBolt11Invoice> {
    let bolt11: lightning::bolt11_invoice::Bolt11Invoice = match input.parse() {
        Ok(invoice) => invoice,
        Err(_) => return None,
    };

    Some(DetailedBolt11Invoice {
        amount_msat: bolt11.amount_milli_satoshis(),
        description: match bolt11.description() {
            Bolt11InvoiceDescriptionRef::Direct(description) => Some(description.to_string()),
            Bolt11InvoiceDescriptionRef::Hash(_) => None,
        },
        description_hash: match bolt11.description() {
            Bolt11InvoiceDescriptionRef::Direct(_) => None,
            Bolt11InvoiceDescriptionRef::Hash(sha256) => Some(sha256.0.to_string()),
        },
        expiry: bolt11.expiry_time().as_secs(),
        invoice: super::Bolt11Invoice {
            bolt11: input.to_string(),
            source: source.clone(),
        },
        min_final_cltv_expiry_delta: bolt11.min_final_cltv_expiry_delta(),
        network: bolt11.network().into(),
        payee_pubkey: bolt11.get_payee_pub_key().to_string(),
        payment_hash: bolt11.payment_hash().to_string(),
        payment_secret: hex::encode(&bolt11.payment_secret().0),
        routing_hints: bolt11
            .route_hints()
            .into_iter()
            .map(|hint| Bolt11RouteHint {
                hops: hint
                    .0
                    .into_iter()
                    .map(|hop| Bolt11RouteHintHop {
                        src_node_id: hop.src_node_id.to_string(),
                        short_channel_id: format_short_channel_id(hop.short_channel_id),
                        fees_base_msat: hop.fees.base_msat,
                        fees_proportional_millionths: hop.fees.proportional_millionths,
                        cltv_expiry_delta: hop.cltv_expiry_delta,
                        htlc_minimum_msat: hop.htlc_minimum_msat,
                        htlc_maximum_msat: hop.htlc_maximum_msat,
                    })
                    .collect(),
            })
            .collect(),
        timestamp: bolt11.duration_since_epoch().as_secs(),
    })
}

fn parse_bolt12_offer(input: &str, source: &PaymentRequestSource) -> Option<DetailedBolt12Offer> {
    let offer: lightning::offers::offer::Offer = match input.parse() {
        Ok(offer) => offer,
        Err(_) => return None,
    };

    let min_amount = match offer.amount() {
        Some(lightning::offers::offer::Amount::Bitcoin { amount_msats }) => {
            Some(super::Amount::Bitcoin { amount_msat: amount_msats })
        }
        Some(lightning::offers::offer::Amount::Currency { iso4217_code, amount }) => {
            Some(super::Amount::Currency {
                iso4217_code: String::from_utf8(iso4217_code.to_vec()).ok()?,
                fractional_amount: amount,
            })
        }
        None => None,
    };

    Some(DetailedBolt12Offer {
        absolute_expiry: offer.absolute_expiry().map(|e| e.as_secs()),
        chains: offer.chains().into_iter().map(|c| c.to_string()).collect(),
        description: offer.description().map(|d| d.to_string()),
        issuer: offer.issuer().map(|i| i.to_string()),
        min_amount,
        offer: Bolt12Offer {
            offer: input.to_string(),
            source: source.clone(),
        },
        paths: offer
            .paths()
            .into_iter()
            .map(|p| Bolt12OfferBlindedPath {
                blinded_hops: p
                    .blinded_hops()
                    .into_iter()
                    .map(|h| h.blinded_node_id.to_string())
                    .collect(),
            })
            .collect(),
        signing_pubkey: offer.issuer_signing_pubkey().map(|p| p.to_string()),
    })
}

fn parse_bolt12_invoice(
    input: &str,
    source: &PaymentRequestSource,
) -> Option<DetailedBolt12Invoice> {
    todo!()
}

fn parse_bolt12_invoice_request(
    input: &str,
    source: &PaymentRequestSource,
) -> Option<Bolt12InvoiceRequest> {
    todo!()
}

pub fn parse_json<T>(json: &str) -> Result<T, ServiceConnectivityError>
where
    for<'a> T: serde::de::Deserialize<'a>,
{
    serde_json::from_str::<T>(json).map_err(|e| {
        ServiceConnectivityError::new(ServiceConnectivityErrorKind::Json, e.to_string())
    })
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum LnurlRequestData {
    PayRequest {
        #[serde(flatten)]
        data: LnurlPayRequest,
    },
    WithdrawRequest {
        #[serde(flatten)]
        data: LnurlWithdrawRequestData,
    },
    #[serde(rename = "login")]
    AuthRequest {
        #[serde(flatten)]
        data: LnurlAuthRequestData,
    },
    Error {
        #[serde(flatten)]
        data: LnurlErrorData,
    },
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::{anyhow, Result};
    use bitcoin::bech32;
    use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
    use serde_json::json;

    use crate::input::parser::InputParser;
    use crate::input::{Bip21, BitcoinAddress, InputType, ParseError, PaymentMethod, PaymentRequest, PaymentRequestSource};
    use crate::test_utils::mock_rest_client::MockRestClient;
    use crate::utils::RestClient;

    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[breez_sdk_macros::async_test_all]
    async fn test_generic_invalid_input() {
        let mock_rest_client = MockRestClient::new();
        let input_parser = InputParser::new(mock_rest_client);

        let result = input_parser.parse("invalid_input").await;

        assert!(matches!(
            result,
            Err(crate::input::ParseError::InvalidInput)
        ));
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_trim_input() {
        let mock_rest_client = MockRestClient::new();
        let input_parser = InputParser::new(mock_rest_client);
        for address in [
            r#"1andreas3batLhQa2FawWjeyjCqyBzypd"#,
            r#"1andreas3batLhQa2FawWjeyjCqyBzypd "#,
            r#"1andreas3batLhQa2FawWjeyjCqyBzypd
            "#,
            r#"
            1andreas3batLhQa2FawWjeyjCqyBzypd
            "#,
            r#" 1andreas3batLhQa2FawWjeyjCqyBzypd
            "#,
        ] {
            let result = input_parser.parse(address).await;
            assert!(matches!(
                result,
                Ok(crate::input::InputType::PaymentRequest(PaymentRequest::PaymentMethod(PaymentMethod::BitcoinAddress(_))))
            ));
        }
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_bitcoin_address() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let input_parser = InputParser::new(mock_rest_client);
        for address in [
            "1andreas3batLhQa2FawWjeyjCqyBzypd",
            "12c6DSiU4Rq3P4ZxziKxzrL5LmMBrzjrJX",
            "bc1qxhmdufsvnuaaaer4ynz88fspdsxq2h9e9cetdj",
            "3CJ7cNxChpcUykQztFSqKFrMVQDN4zTTsp",
        ] {
            let result = input_parser.parse(address).await;
            assert!(matches!(
                result,
                Ok(crate::input::InputType::PaymentRequest(PaymentRequest::PaymentMethod(PaymentMethod::BitcoinAddress(_))))
            ));
        }
        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_bitcoin_address_bip21() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let input_parser = InputParser::new(mock_rest_client);
        // Addresses from https://github.com/Kixunil/bip21/blob/master/src/lib.rs

        // Invalid address with the `bitcoin:` prefix
        assert!(matches!(
            input_parser.parse("bitcoin:testinvalidaddress").await,
            Err(ParseError::InvalidInput)
        ));

        let addr = "1andreas3batLhQa2FawWjeyjCqyBzypd";

        // Valid address with the `bitcoin:` prefix
        assert!(matches!(
            input_parser.parse(&format!("bitcoin:{addr}")).await,
            Ok(InputType::PaymentRequest(PaymentRequest::Bip21(Bip21 { amount_sat, asset_id, uri, extras, label, message, payment_methods })))
            if payment_methods.len() == 1 && matches!(&payment_methods[0], PaymentMethod::BitcoinAddress(BitcoinAddress { address, network, source }) if address == addr)
        ));
        
        // Address with amount
        assert!(matches!(
            input_parser.parse(&format!("bitcoin:{addr}?amount=0.00002000")).await,
            Ok(InputType::PaymentRequest(PaymentRequest::Bip21(Bip21 { amount_sat, asset_id, uri, extras, label, message, payment_methods })))
            if payment_methods.len() == 1
                && amount_sat == Some(2000)
                && matches!(&payment_methods[0], PaymentMethod::BitcoinAddress(BitcoinAddress { address, network, source }) if address == addr)
        ));

        // Address with amount and label
        let lbl = "test-label";
        assert!(matches!(
            input_parser.parse(&format!("bitcoin:{addr}?amount=0.00002000&label={lbl}")).await,
            Ok(InputType::PaymentRequest(PaymentRequest::Bip21(Bip21 { amount_sat, asset_id, uri, extras, label, message, payment_methods })))
            if payment_methods.len() == 1
                && amount_sat == Some(2000)
                && label.as_deref() == Some(lbl)
                && matches!(&payment_methods[0], PaymentMethod::BitcoinAddress(BitcoinAddress { address, network, source }) if address == addr)
        ));

        // Address with amount, label and message
        let msg = "test-message";
        assert!(matches!(
            input_parser.parse(&format!("bitcoin:{addr}?amount=0.00002000&label={lbl}&message={msg}")).await,
            Ok(InputType::PaymentRequest(PaymentRequest::Bip21(Bip21 { amount_sat, asset_id, uri, extras, label, message, payment_methods })))
            if payment_methods.len() == 1
                && amount_sat == Some(2000)
                && label.as_deref() == Some(lbl)
                && message.as_deref() == Some(msg)
                && matches!(&payment_methods[0], PaymentMethod::BitcoinAddress(BitcoinAddress { address, network, source }) if address == addr)
        ));

        Ok(())
    }

    /// BIP21 amounts which can lead to rounding errors.
    /// The format is: (sat amount, BIP21 BTC amount)
    pub(crate) fn get_bip21_rounding_test_vectors() -> Vec<(u64, f64)> {
        vec![
            (999, 0.0000_0999),
            (1_000, 0.0000_1000),
            (59_810, 0.0005_9810),
        ]
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_bitcoin_address_bip21_rounding() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let input_parser = InputParser::new(mock_rest_client);
        for (amt, amount_btc) in get_bip21_rounding_test_vectors() {
            let addr = format!("bitcoin:1andreas3batLhQa2FawWjeyjCqyBzypd?amount={amount_btc}");

            assert!(matches!(
                input_parser.parse(&format!("bitcoin:{addr}?amount=0.00002000")).await,
                Ok(InputType::PaymentRequest(PaymentRequest::Bip21(Bip21 { amount_sat, asset_id, uri, extras, label, message, payment_methods })))
                if payment_methods.len() == 1
                    && amount_sat == Some(amt)
                    && matches!(&payment_methods[0], PaymentMethod::BitcoinAddress(BitcoinAddress { address, network, source }) if address == &addr)
            ));
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_liquid_address() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        assert!(parse_with_rest_client(rest_client.as_ref(), "tlq1qqw5ur50rnvcx33vmljjtnez3hrtl6n7vs44tdj2c9fmnxrrgzgwnhw6jtpn8cljkmlr8tgfw9hemrr5y8u2nu024hhak3tpdk", None)
            .await
            .is_ok());
        assert!(parse_with_rest_client(rest_client.as_ref(), "liquidnetwork:tlq1qqw5ur50rnvcx33vmljjtnez3hrtl6n7vs44tdj2c9fmnxrrgzgwnhw6jtpn8cljkmlr8tgfw9hemrr5y8u2nu024hhak3tpdk", None)
            .await
            .is_ok());
        assert!(parse_with_rest_client(rest_client.as_ref(), "wrong-net:tlq1qqw5ur50rnvcx33vmljjtnez3hrtl6n7vs44tdj2c9fmnxrrgzgwnhw6jtpn8cljkmlr8tgfw9hemrr5y8u2nu024hhak3tpdk", None).await.is_err());
        assert!(parse_with_rest_client(
            rest_client.as_ref(),
            "liquidnetwork:testinvalidaddress",
            None
        )
        .await
        .is_err());

        let address: elements::Address = "tlq1qqw5ur50rnvcx33vmljjtnez3hrtl6n7vs44tdj2c9fmnxrrgzgwnhw6jtpn8cljkmlr8tgfw9hemrr5y8u2nu024hhak3tpdk".parse()?;
        let amount_btc = 0.00001; // 1000 sats
        let label = "label";
        let message = "this%20is%20a%20message";
        let asset_id = elements::issuance::AssetId::LIQUID_BTC.to_string();
        let output = parse_with_rest_client(rest_client.as_ref(), &format!(
                    "liquidnetwork:{}?amount={amount_btc}&assetid={asset_id}&label={label}&message={message}",
                    address
                ),
                           None)
        .await?;

        if let InputType::LiquidAddress {
            address: liquid_address_data,
        } = output
        {
            assert_eq!(Network::Bitcoin, liquid_address_data.network);
            assert_eq!(address.to_string(), liquid_address_data.address.to_string());
            assert_eq!(
                Some((amount_btc * 100_000_000.0) as u64),
                liquid_address_data.amount_sat
            );
            assert_eq!(Some(label.to_string()), liquid_address_data.label);
            assert_eq!(
                Some(urlencoding::decode(message).unwrap().into_owned()),
                liquid_address_data.message
            );
        } else {
            panic!("Invalid input type received");
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_bolt11() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);
        let bolt11 = "lnbc110n1p38q3gtpp5ypz09jrd8p993snjwnm68cph4ftwp22le34xd4r8ftspwshxhmnsdqqxqyjw5qcqpxsp5htlg8ydpywvsa7h3u4hdn77ehs4z4e844em0apjyvmqfkzqhhd2q9qgsqqqyssqszpxzxt9uuqzymr7zxcdccj5g69s8q7zzjs7sgxn9ejhnvdh6gqjcy22mss2yexunagm5r2gqczh8k24cwrqml3njskm548aruhpwssq9nvrvz";

        // Invoice without prefix
        assert!(matches!(
            parse_with_rest_client(rest_client.as_ref(), bolt11, None).await?,
            InputType::Bolt11 { invoice: _invoice }
        ));

        // Invoice with prefix
        let invoice_with_prefix = format!("lightning:{bolt11}");
        assert!(matches!(
            parse_with_rest_client(rest_client.as_ref(), &invoice_with_prefix, None).await?,
            InputType::Bolt11 { invoice: _invoice }
        ));

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_capitalized_bolt11() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);
        let bolt11 = "LNBC110N1P38Q3GTPP5YPZ09JRD8P993SNJWNM68CPH4FTWP22LE34XD4R8FTSPWSHXHMNSDQQXQYJW5QCQPXSP5HTLG8YDPYWVSA7H3U4HDN77EHS4Z4E844EM0APJYVMQFKZQHHD2Q9QGSQQQYSSQSZPXZXT9UUQZYMR7ZXCDCCJ5G69S8Q7ZZJS7SGXN9EJHNVDH6GQJCY22MSS2YEXUNAGM5R2GQCZH8K24CWRQML3NJSKM548ARUHPWSSQ9NVRVZ";

        // Invoice without prefix
        assert!(matches!(
            parse_with_rest_client(rest_client.as_ref(), bolt11, None).await?,
            InputType::Bolt11 { invoice: _invoice }
        ));

        // Invoice with prefix
        let invoice_with_prefix = format!("LIGHTNING:{bolt11}");
        assert!(matches!(
            parse_with_rest_client(rest_client.as_ref(), &invoice_with_prefix, None).await?,
            InputType::Bolt11 { invoice: _invoice }
        ));

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_bolt11_with_fallback_bitcoin_address() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);
        let addr = "1andreas3batLhQa2FawWjeyjCqyBzypd";
        let bolt11 = "lnbc110n1p38q3gtpp5ypz09jrd8p993snjwnm68cph4ftwp22le34xd4r8ftspwshxhmnsdqqxqyjw5qcqpxsp5htlg8ydpywvsa7h3u4hdn77ehs4z4e844em0apjyvmqfkzqhhd2q9qgsqqqyssqszpxzxt9uuqzymr7zxcdccj5g69s8q7zzjs7sgxn9ejhnvdh6gqjcy22mss2yexunagm5r2gqczh8k24cwrqml3njskm548aruhpwssq9nvrvz";

        // Address and invoice
        // BOLT11 is the first URI arg (preceded by '?')
        let addr_1 = format!("bitcoin:{addr}?lightning={bolt11}");
        assert!(matches!(
            parse_with_rest_client(rest_client.as_ref(), &addr_1, None).await?,
            InputType::Bolt11 { invoice: _invoice }
        ));

        // Address, amount and invoice
        // BOLT11 is not the first URI arg (preceded by '&')
        let addr_2 = format!("bitcoin:{addr}?amount=0.00002000&lightning={bolt11}");
        assert!(matches!(
            parse_with_rest_client(rest_client.as_ref(), &addr_2, None).await?,
            InputType::Bolt11 { invoice: _invoice }
        ));

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_url() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);
        assert!(matches!(
            parse_with_rest_client(rest_client.as_ref(), "https://breez.technology", None).await?,
            InputType::Url { url: _url }
        ));
        assert!(matches!(
            parse_with_rest_client(rest_client.as_ref(), "https://breez.technology/", None).await?,
            InputType::Url { url: _url }
        ));
        assert!(matches!(
            parse_with_rest_client(
                rest_client.as_ref(),
                "https://breez.technology/test-path",
                None
            )
            .await?,
            InputType::Url { url: _url }
        ));
        assert!(matches!(
            parse_with_rest_client(
                rest_client.as_ref(),
                "https://breez.technology/test-path?arg1=val1&arg2=val2",
                None
            )
            .await?,
            InputType::Url { url: _url }
        ));
        // `lightning` query param is not an LNURL.
        assert!(matches!(
            parse_with_rest_client(
                rest_client.as_ref(),
                "https://breez.technology?lightning=nonsense",
                None
            )
            .await?,
            InputType::Url { url: _url }
        ));

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_node_id() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&[0xab; 32])?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        mock_external_parser(&mock_rest_client, "".to_string(), 400);
        mock_external_parser(&mock_rest_client, "".to_string(), 400);
        mock_external_parser(&mock_rest_client, "".to_string(), 400);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        match parse_with_rest_client(rest_client.as_ref(), &public_key.to_string(), None).await? {
            InputType::NodeId { node_id } => {
                assert_eq!(node_id, public_key.to_string());
            }
            _ => return Err(anyhow!("Unexpected type")),
        }

        // Other formats and sizes
        assert!(parse_with_rest_client(
            rest_client.as_ref(),
            "012345678901234567890123456789012345678901234567890123456789mnop",
            None
        )
        .await
        .is_err());
        assert!(
            parse_with_rest_client(rest_client.as_ref(), "0123456789", None)
                .await
                .is_err()
        );
        assert!(
            parse_with_rest_client(rest_client.as_ref(), "abcdefghij", None)
                .await
                .is_err()
        );

        // Plain Node ID
        assert!(parse_with_rest_client(
            rest_client.as_ref(),
            "03864ef025fde8fb587d989186ce6a4a186895ee44a926bfc370e2c366597a3f8f",
            None
        )
        .await
        .is_ok());
        // Plain Node ID (66 hex chars) with @ separator and any string afterwards
        assert!(parse_with_rest_client(
            rest_client.as_ref(),
            "03864ef025fde8fb587d989186ce6a4a186895ee44a926bfc370e2c366597a3f8f@",
            None
        )
        .await
        .is_ok());
        assert!(parse_with_rest_client(
            rest_client.as_ref(),
            "03864ef025fde8fb587d989186ce6a4a186895ee44a926bfc370e2c366597a3f8f@sdfsffs",
            None
        )
        .await
        .is_ok());
        assert!(parse_with_rest_client(
            rest_client.as_ref(),
            "03864ef025fde8fb587d989186ce6a4a186895ee44a926bfc370e2c366597a3f8f@1.2.3.4:1234",
            None
        )
        .await
        .is_ok());

        // Invalid Node ID (66 chars ending in non-hex-chars) with @ separator and any string afterwards -> invalid
        assert!(parse_with_rest_client(
            rest_client.as_ref(),
            "03864ef025fde8fb587d989186ce6a4a186895ee44a926bfc370e2c366597a3zzz@",
            None
        )
        .await
        .is_err());
        assert!(parse_with_rest_client(
            rest_client.as_ref(),
            "03864ef025fde8fb587d989186ce6a4a186895ee44a926bfc370e2c366597a3zzz@sdfsffs",
            None
        )
        .await
        .is_err());
        assert!(parse_with_rest_client(
            rest_client.as_ref(),
            "03864ef025fde8fb587d989186ce6a4a186895ee44a926bfc370e2c366597a3zzz@1.2.3.4:1234",
            None
        )
        .await
        .is_err());

        Ok(())
    }

    #[sdk_macros::test_all]
    fn test_lnurl_pay_lud_01() -> Result<()> {
        // Covers cases in LUD-01: Base LNURL encoding and decoding
        // https://github.com/lnurl/luds/blob/luds/01.md

        // HTTPS allowed with clearnet domains
        assert_eq!(
            lnurl_decode(&bech32::encode(
                "LNURL",
                "https://domain.com".to_base32(),
                Variant::Bech32
            )?)?,
            ("domain.com".into(), "https://domain.com".into(), None)
        );

        // HTTP not allowed with clearnet domains
        assert!(lnurl_decode(&bech32::encode(
            "LNURL",
            "http://domain.com".to_base32(),
            Variant::Bech32
        )?)
        .is_err());

        // HTTP allowed with onion domains
        assert_eq!(
            lnurl_decode(&bech32::encode(
                "LNURL",
                "http://3fdsf.onion".to_base32(),
                Variant::Bech32
            )?)?,
            ("3fdsf.onion".into(), "http://3fdsf.onion".into(), None)
        );

        // HTTPS not allowed with onion domains
        assert!(lnurl_decode(&bech32::encode(
            "LNURL",
            "https://3fdsf.onion".to_base32(),
            Variant::Bech32
        )?)
        .is_err());

        let decoded_url = "https://service.com/api?q=3fc3645b439ce8e7f2553a69e5267081d96dcd340693afabe04be7b0ccd178df";
        let lnurl_raw = "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";

        assert_eq!(
            lnurl_decode(lnurl_raw)?,
            ("service.com".into(), decoded_url.into(), None)
        );

        // Uppercase and lowercase allowed, but mixed case is invalid
        assert!(lnurl_decode(&lnurl_raw.to_uppercase()).is_ok());
        assert!(lnurl_decode(&lnurl_raw.to_lowercase()).is_ok());
        assert!(lnurl_decode(&format!(
            "{}{}",
            lnurl_raw[..5].to_uppercase(),
            lnurl_raw[5..].to_lowercase()
        ))
        .is_err());

        Ok(())
    }

    fn mock_lnurl_withdraw_endpoint(mock_rest_client: &MockRestClient, error: Option<String>) {
        let (response_body, status_code) = match error {
            None => (json!({
                "tag": "withdrawRequest",
                "callback": "https://localhost/lnurl-withdraw/callback/e464f841c44dbdd86cee4f09f4ccd3ced58d2e24f148730ec192748317b74538",
                "k1": "37b4c919f871c090830cc47b92a544a30097f03430bc39670b8ec0da89f01a81",
                "minWithdrawable": 3000,
                "maxWithdrawable": 12000,
                "defaultDescription": "sample withdraw",
            }).to_string(), 200),
            Some(err_reason) => (json!({
                "status": "ERROR",
                "reason": err_reason
            })
            .to_string(), 400),
        };

        mock_rest_client.add_response(MockResponse::new(status_code, response_body));
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_withdraw_lud_03() -> Result<(), Box<dyn std::error::Error>> {
        let mock_rest_client = MockRestClient::new();
        // Covers cases in LUD-03: withdrawRequest base spec
        // https://github.com/lnurl/luds/blob/luds/03.md

        let path = "/lnurl-withdraw?session=bc893fafeb9819046781b47d68fdcf88fa39a28898784c183b42b7ac13820d81";
        mock_lnurl_withdraw_endpoint(&mock_rest_client, None);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        let lnurl_withdraw_encoded = "lnurl1dp68gurn8ghj7mr0vdskc6r0wd6z7mrww4exctthd96xserjv9mn7um9wdekjmmw843xxwpexdnxzen9vgunsvfexq6rvdecx93rgdmyxcuxverrvcursenpxvukzv3c8qunsdecx33nzwpnvg6ryc3hv93nzvecxgcxgwp3h33lxk";
        assert_eq!(
            lnurl_decode(lnurl_withdraw_encoded)?,
            ("localhost".into(), format!("https://localhost{path}"), None,)
        );

        if let InputType::LnUrlWithdraw { data: wd } =
            parse_with_rest_client(rest_client.as_ref(), lnurl_withdraw_encoded, None).await?
        {
            assert_eq!(wd.callback, "https://localhost/lnurl-withdraw/callback/e464f841c44dbdd86cee4f09f4ccd3ced58d2e24f148730ec192748317b74538");
            assert_eq!(
                wd.k1,
                "37b4c919f871c090830cc47b92a544a30097f03430bc39670b8ec0da89f01a81"
            );
            assert_eq!(wd.min_withdrawable, 3000);
            assert_eq!(wd.max_withdrawable, 12000);
            assert_eq!(wd.default_description, "sample withdraw");
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_withdraw_in_url() -> Result<(), Box<dyn std::error::Error>> {
        let mock_rest_client = MockRestClient::new();
        let path = "/lnurl-withdraw?session=bc893fafeb9819046781b47d68fdcf88fa39a28898784c183b42b7ac13820d81";
        mock_lnurl_withdraw_endpoint(&mock_rest_client, None);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        let lnurl_withdraw_encoded = "lnurl1dp68gurn8ghj7mr0vdskc6r0wd6z7mrww4exctthd96xserjv9mn7um9wdekjmmw843xxwpexdnxzen9vgunsvfexq6rvdecx93rgdmyxcuxverrvcursenpxvukzv3c8qunsdecx33nzwpnvg6ryc3hv93nzvecxgcxgwp3h33lxk";
        assert_eq!(
            lnurl_decode(lnurl_withdraw_encoded)?,
            ("localhost".into(), format!("https://localhost{path}"), None,)
        );
        let url = format!("https://bitcoin.org?lightning={lnurl_withdraw_encoded}");

        if let InputType::LnUrlWithdraw { data: wd } =
            parse_with_rest_client(rest_client.as_ref(), &url, None).await?
        {
            assert_eq!(wd.callback, "https://localhost/lnurl-withdraw/callback/e464f841c44dbdd86cee4f09f4ccd3ced58d2e24f148730ec192748317b74538");
            assert_eq!(
                wd.k1,
                "37b4c919f871c090830cc47b92a544a30097f03430bc39670b8ec0da89f01a81"
            );
            assert_eq!(wd.min_withdrawable, 3000);
            assert_eq!(wd.max_withdrawable, 12000);
            assert_eq!(wd.default_description, "sample withdraw");
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_auth_lud_04() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);
        // Covers cases in LUD-04: `auth` base spec
        // https://github.com/lnurl/luds/blob/luds/04.md

        // No action specified
        let decoded_url = "https://localhost/lnurl-login?tag=login&k1=1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822";
        let lnurl_auth_encoded = "lnurl1dp68gurn8ghj7mr0vdskc6r0wd6z7mrww4excttvdankjm3lw3skw0tvdankjm3xdvcn6vtp8q6n2dfsx5mrjwtrxdjnqvtzv56rzcnyv3jrxv3sxqmkyenrvv6kve3exv6nqdtyv43nqcmzvdsnvdrzx33rsenxx5unqc3cxgeqgntfgu";
        assert_eq!(
            lnurl_decode(lnurl_auth_encoded)?,
            ("localhost".into(), decoded_url.into(), None)
        );

        if let InputType::LnUrlAuth { data: ad } =
            parse_with_rest_client(rest_client.as_ref(), lnurl_auth_encoded, None).await?
        {
            assert_eq!(
                ad.k1,
                "1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822"
            );
            assert_eq!(ad.domain, "localhost".to_string());
            assert_eq!(ad.action, None);
        }

        // Action = register
        let _decoded_url = "https://localhost/lnurl-login?tag=login&k1=1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822&action=register";
        let lnurl_auth_encoded = "lnurl1dp68gurn8ghj7mr0vdskc6r0wd6z7mrww4excttvdankjm3lw3skw0tvdankjm3xdvcn6vtp8q6n2dfsx5mrjwtrxdjnqvtzv56rzcnyv3jrxv3sxqmkyenrvv6kve3exv6nqdtyv43nqcmzvdsnvdrzx33rsenxx5unqc3cxgezvctrw35k7m3awfjkw6tnw3jhys2umys";
        if let InputType::LnUrlAuth { data: ad } =
            parse_with_rest_client(rest_client.as_ref(), lnurl_auth_encoded, None).await?
        {
            assert_eq!(
                ad.k1,
                "1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822"
            );
            assert_eq!(ad.domain, "localhost".to_string());
            assert_eq!(ad.action, Some("register".into()));
        }

        // Action = login
        let _decoded_url = "https://localhost/lnurl-login?tag=login&k1=1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822&action=login";
        let lnurl_auth_encoded = "lnurl1dp68gurn8ghj7mr0vdskc6r0wd6z7mrww4excttvdankjm3lw3skw0tvdankjm3xdvcn6vtp8q6n2dfsx5mrjwtrxdjnqvtzv56rzcnyv3jrxv3sxqmkyenrvv6kve3exv6nqdtyv43nqcmzvdsnvdrzx33rsenxx5unqc3cxgezvctrw35k7m3ad3hkw6tw2acjtx";
        if let InputType::LnUrlAuth { data: ad } =
            parse_with_rest_client(rest_client.as_ref(), lnurl_auth_encoded, None).await?
        {
            assert_eq!(
                ad.k1,
                "1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822"
            );
            assert_eq!(ad.domain, "localhost".to_string());
            assert_eq!(ad.action, Some("login".into()));
        }

        // Action = link
        let _decoded_url = "https://localhost/lnurl-login?tag=login&k1=1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822&action=link";
        let lnurl_auth_encoded = "lnurl1dp68gurn8ghj7mr0vdskc6r0wd6z7mrww4excttvdankjm3lw3skw0tvdankjm3xdvcn6vtp8q6n2dfsx5mrjwtrxdjnqvtzv56rzcnyv3jrxv3sxqmkyenrvv6kve3exv6nqdtyv43nqcmzvdsnvdrzx33rsenxx5unqc3cxgezvctrw35k7m3ad35ku6cc8mvs6";
        if let InputType::LnUrlAuth { data: ad } =
            parse_with_rest_client(rest_client.as_ref(), lnurl_auth_encoded, None).await?
        {
            assert_eq!(
                ad.k1,
                "1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822"
            );
            assert_eq!(ad.domain, "localhost".to_string());
            assert_eq!(ad.action, Some("link".into()));
        }

        // Action = auth
        let _decoded_url = "https://localhost/lnurl-login?tag=login&k1=1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822&action=auth";
        let lnurl_auth_encoded = "lnurl1dp68gurn8ghj7mr0vdskc6r0wd6z7mrww4excttvdankjm3lw3skw0tvdankjm3xdvcn6vtp8q6n2dfsx5mrjwtrxdjnqvtzv56rzcnyv3jrxv3sxqmkyenrvv6kve3exv6nqdtyv43nqcmzvdsnvdrzx33rsenxx5unqc3cxgezvctrw35k7m3av96hg6qmg6zgu";
        if let InputType::LnUrlAuth { data: ad } =
            parse_with_rest_client(rest_client.as_ref(), lnurl_auth_encoded, None).await?
        {
            assert_eq!(
                ad.k1,
                "1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822"
            );
            assert_eq!(ad.domain, "localhost".to_string());
            assert_eq!(ad.action, Some("auth".into()));
        }

        // Action = another, invalid type
        let _decoded_url = "https://localhost/lnurl-login?tag=login&k1=1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822&action=invalid";
        let lnurl_auth_encoded = "lnurl1dp68gurn8ghj7mr0vdskc6r0wd6z7mrww4excttvdankjm3lw3skw0tvdankjm3xdvcn6vtp8q6n2dfsx5mrjwtrxdjnqvtzv56rzcnyv3jrxv3sxqmkyenrvv6kve3exv6nqdtyv43nqcmzvdsnvdrzx33rsenxx5unqc3cxgezvctrw35k7m3ad9h8vctvd9jq2s4vfw";
        assert!(
            parse_with_rest_client(rest_client.as_ref(), lnurl_auth_encoded, None)
                .await
                .is_err()
        );

        Ok(())
    }

    fn mock_lnurl_pay_endpoint(mock_rest_client: &MockRestClient, error: Option<String>) {
        let response_body = match error {
            None => json!({
                "callback":"https://localhost/lnurl-pay/callback/db945b624265fc7f5a8d77f269f7589d789a771bdfd20e91a3cf6f50382a98d7",
                "tag": "payRequest",
                "maxSendable": 16000,
                "minSendable": 4000,
                "metadata": "[
                    [\"text/plain\",\"WRhtV\"],
                    [\"text/long-desc\",\"MBTrTiLCFS\"],
                    [\"image/png;base64\",\"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\"]
                ]",
                "commentAllowed": 0,
                "payerData":{
                    "name": { "mandatory":false },
                    "pubkey": { "mandatory":false },
                    "identifier": { "mandatory":false },
                    "email":{ "mandatory":false },
                    "auth": { "mandatory":false, "k1":"18ec6d5b96db6f219baed2f188aee7359fcf5bea11bb7d5b47157519474c2222" }
                }
            }).to_string(),
            Some(err_reason) => json!({
                "status": "ERROR",
                "reason": err_reason
            })
            .to_string(),
        };

        mock_rest_client.add_response(MockResponse::new(200, response_body));
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_pay_lud_06() -> Result<(), Box<dyn std::error::Error>> {
        let mock_rest_client = MockRestClient::new();
        // Covers cases in LUD-06: payRequest base spec
        // https://github.com/lnurl/luds/blob/luds/06.md
        let lnurl_pay_encoded = "lnurl1dp68gurn8ghj7mr0vdskc6r0wd6z7mrww4excttsv9un7um9wdekjmmw84jxywf5x43rvv35xgmr2enrxanr2cfcvsmnwe3jxcukvde48qukgdec89snwde3vfjxvepjxpjnjvtpxd3kvdnxx5crxwpjvyunsephsz36jf";
        let path =
            "/lnurl-pay?session=db945b624265fc7f5a8d77f269f7589d789a771bdfd20e91a3cf6f50382a98d7";
        mock_lnurl_pay_endpoint(&mock_rest_client, None);
        mock_lnurl_pay_endpoint(&mock_rest_client, None);
        mock_lnurl_pay_endpoint(&mock_rest_client, None);
        mock_lnurl_pay_endpoint(&mock_rest_client, None);
        mock_lnurl_pay_endpoint(&mock_rest_client, None);
        mock_lnurl_pay_endpoint(&mock_rest_client, None);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        assert_eq!(
            lnurl_decode(lnurl_pay_encoded)?,
            ("localhost".into(), format!("https://localhost{path}"), None)
        );

        if let InputType::LnUrlPay { data: pd, .. } =
            parse_with_rest_client(rest_client.as_ref(), lnurl_pay_encoded, None).await?
        {
            assert_eq!(pd.callback, "https://localhost/lnurl-pay/callback/db945b624265fc7f5a8d77f269f7589d789a771bdfd20e91a3cf6f50382a98d7");
            assert_eq!(pd.max_sendable, 16000);
            assert_eq!(pd.min_sendable, 4000);
            assert_eq!(pd.comment_allowed, 0);
            assert_eq!(pd.domain, "localhost");

            assert_eq!(pd.metadata_vec()?.len(), 3);
            assert_eq!(
                pd.metadata_vec()?.first().ok_or("Key not found")?.key,
                "text/plain"
            );
            assert_eq!(
                pd.metadata_vec()?.first().ok_or("Key not found")?.value,
                "WRhtV"
            );
            assert_eq!(
                pd.metadata_vec()?.get(1).ok_or("Key not found")?.key,
                "text/long-desc"
            );
            assert_eq!(
                pd.metadata_vec()?.get(1).ok_or("Key not found")?.value,
                "MBTrTiLCFS"
            );
            assert_eq!(
                pd.metadata_vec()?.get(2).ok_or("Key not found")?.key,
                "image/png;base64"
            );
        }

        for lnurl_pay in [
            lnurl_pay_encoded.to_uppercase().as_str(),
            format!("lightning:{}", lnurl_pay_encoded).as_str(),
            format!("lightning:{}", lnurl_pay_encoded.to_uppercase()).as_str(),
            format!("LIGHTNING:{}", lnurl_pay_encoded).as_str(),
            format!("LIGHTNING:{}", lnurl_pay_encoded.to_uppercase()).as_str(),
        ] {
            assert!(matches!(
                parse_with_rest_client(rest_client.as_ref(), lnurl_pay, None).await?,
                InputType::LnUrlPay { .. }
            ));
        }
        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_pay_lud_16_ln_address() -> Result<(), Box<dyn std::error::Error>> {
        let mock_rest_client = MockRestClient::new();
        // Covers cases in LUD-16: Paying to static internet identifiers (LN Address)
        // https://github.com/lnurl/luds/blob/luds/16.md

        let ln_address = "user@domain.net";
        mock_lnurl_pay_endpoint(&mock_rest_client, None);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        if let InputType::LnUrlPay { data: pd, .. } =
            parse_with_rest_client(rest_client.as_ref(), ln_address, None).await?
        {
            assert_eq!(pd.callback, "https://localhost/lnurl-pay/callback/db945b624265fc7f5a8d77f269f7589d789a771bdfd20e91a3cf6f50382a98d7");
            assert_eq!(pd.max_sendable, 16000);
            assert_eq!(pd.min_sendable, 4000);
            assert_eq!(pd.comment_allowed, 0);
            assert_eq!(pd.domain, "domain.net");
            assert_eq!(pd.ln_address, Some(ln_address.to_string()));

            assert_eq!(pd.metadata_vec()?.len(), 3);
            assert_eq!(
                pd.metadata_vec()?.first().ok_or("Key not found")?.key,
                "text/plain"
            );
            assert_eq!(
                pd.metadata_vec()?.first().ok_or("Key not found")?.value,
                "WRhtV"
            );
            assert_eq!(
                pd.metadata_vec()?.get(1).ok_or("Key not found")?.key,
                "text/long-desc"
            );
            assert_eq!(
                pd.metadata_vec()?.get(1).ok_or("Key not found")?.value,
                "MBTrTiLCFS"
            );
            assert_eq!(
                pd.metadata_vec()?.get(2).ok_or("Key not found")?.key,
                "image/png;base64"
            );
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_pay_lud_16_ln_address_with_prefix() -> Result<(), Box<dyn std::error::Error>>
    {
        let mock_rest_client = MockRestClient::new();
        // Covers cases in LUD-16, with BIP-353 prefix.

        let ln_address = "₿user@domain.net";
        let server_ln_address = "user@domain.net";
        mock_lnurl_pay_endpoint(&mock_rest_client, None);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        if let InputType::LnUrlPay { data: pd, .. } =
            parse_with_rest_client(rest_client.as_ref(), ln_address, None).await?
        {
            assert_eq!(pd.callback, "https://localhost/lnurl-pay/callback/db945b624265fc7f5a8d77f269f7589d789a771bdfd20e91a3cf6f50382a98d7");
            assert_eq!(pd.max_sendable, 16000);
            assert_eq!(pd.min_sendable, 4000);
            assert_eq!(pd.comment_allowed, 0);
            assert_eq!(pd.domain, "domain.net");
            assert_eq!(pd.ln_address, Some(server_ln_address.to_string()));
        } else {
            panic!("input was not ln address")
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_pay_lud_16_ln_address_error() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        // Covers cases in LUD-16: Paying to static internet identifiers (LN Address)
        // https://github.com/lnurl/luds/blob/luds/16.md

        let ln_address = "error@domain.com";
        let expected_err = "Error msg from LNURL endpoint found via LN Address";
        mock_lnurl_pay_endpoint(&mock_rest_client, Some(expected_err.to_string()));
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        if let InputType::LnUrlError { data: msg } =
            parse_with_rest_client(rest_client.as_ref(), ln_address, None).await?
        {
            assert_eq!(msg.reason, expected_err);
            return Ok(());
        }

        Err(anyhow!("Unrecognized input type"))
    }

    #[sdk_macros::test_all]
    fn test_ln_address_lud_16_decode() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            lnurl_decode("user@domain.onion")?,
            (
                "domain.onion".into(),
                "http://domain.onion/.well-known/lnurlp/user".into(),
                Some("user@domain.onion".into()),
            )
        );
        assert_eq!(
            lnurl_decode("user@domain.com")?,
            (
                "domain.com".into(),
                "https://domain.com/.well-known/lnurlp/user".into(),
                Some("user@domain.com".into()),
            )
        );
        assert_eq!(
            lnurl_decode("user@domain.net")?,
            (
                "domain.net".into(),
                "https://domain.net/.well-known/lnurlp/user".into(),
                Some("user@domain.net".into()),
            )
        );
        assert_eq!(
            lnurl_decode("User@domain.com")?,
            (
                "domain.com".into(),
                "https://domain.com/.well-known/lnurlp/user".into(),
                Some("user@domain.com".into()),
            )
        );
        assert_eq!(
            lnurl_decode("ODELL@DOMAIN.COM")?,
            (
                "domain.com".into(),
                "https://domain.com/.well-known/lnurlp/odell".into(),
                Some("odell@domain.com".into()),
            )
        );
        assert!(ln_address_decode("invalid_ln_address").is_err());

        // Valid chars are a-z0-9-_.
        assert!(lnurl_decode("user.testy_test1@domain.com").is_ok());
        assert!(lnurl_decode("user+1@domain.com").is_err());

        Ok(())
    }

    #[sdk_macros::test_all]
    fn test_lnurl_lud_17_prefixes() -> Result<(), Box<dyn std::error::Error>> {
        // Covers cases in LUD-17: Protocol schemes and raw (non bech32-encoded) URLs
        // https://github.com/lnurl/luds/blob/luds/17.md

        // Variant-specific prefix replaces https for clearnet and http for onion

        // For onion addresses, the prefix maps to an equivalent HTTP URL
        assert_eq!(
            lnurl_decode("lnurlp://asfddf2dsf3f.onion")?,
            (
                "asfddf2dsf3f.onion".into(),
                "http://asfddf2dsf3f.onion".into(),
                None,
            )
        );
        assert_eq!(
            lnurl_decode("lnurlp://asfddf2dsf3flnurlp.onion")?,
            (
                "asfddf2dsf3flnurlp.onion".into(),
                "http://asfddf2dsf3flnurlp.onion".into(),
                None,
            )
        );
        assert_eq!(
            lnurl_decode("lnurlw://asfddf2dsf3f.onion")?,
            (
                "asfddf2dsf3f.onion".into(),
                "http://asfddf2dsf3f.onion".into(),
                None,
            )
        );
        assert_eq!(
            lnurl_decode("keyauth://asfddf2dsf3f.onion")?,
            (
                "asfddf2dsf3f.onion".into(),
                "http://asfddf2dsf3f.onion".into(),
                None,
            )
        );

        // For non-onion addresses, the prefix maps to an equivalent HTTPS URL
        assert_eq!(
            lnurl_decode("lnurlp://domain.com")?,
            ("domain.com".into(), "https://domain.com".into(), None)
        );
        assert_eq!(
            lnurl_decode("lnurlp://lnurlp.com")?,
            ("lnurlp.com".into(), "https://lnurlp.com".into(), None)
        );
        assert_eq!(
            lnurl_decode("lnurlw://domain.com")?,
            ("domain.com".into(), "https://domain.com".into(), None)
        );
        assert_eq!(
            lnurl_decode("lnurlw://lnurlw.com")?,
            ("lnurlw.com".into(), "https://lnurlw.com".into(), None)
        );
        assert_eq!(
            lnurl_decode("keyauth://domain.com")?,
            ("domain.com".into(), "https://domain.com".into(), None)
        );
        assert_eq!(
            lnurl_decode("keyauth://keyauth.com")?,
            ("keyauth.com".into(), "https://keyauth.com".into(), None)
        );

        // Same as above, but prefix: approach instead of prefix://
        assert_eq!(
            lnurl_decode("lnurlp:asfddf2dsf3f.onion")?,
            (
                "asfddf2dsf3f.onion".into(),
                "http://asfddf2dsf3f.onion".into(),
                None
            )
        );
        assert_eq!(
            lnurl_decode("lnurlw:asfddf2dsf3f.onion")?,
            (
                "asfddf2dsf3f.onion".into(),
                "http://asfddf2dsf3f.onion".into(),
                None
            )
        );
        assert_eq!(
            lnurl_decode("keyauth:asfddf2dsf3f.onion")?,
            (
                "asfddf2dsf3f.onion".into(),
                "http://asfddf2dsf3f.onion".into(),
                None
            )
        );

        assert_eq!(
            lnurl_decode("lnurlp:domain.com")?,
            ("domain.com".into(), "https://domain.com".into(), None)
        );
        assert_eq!(
            lnurl_decode("lnurlp:domain.com/lnurlp:lol")?,
            (
                "domain.com".into(),
                "https://domain.com/lnurlp:lol".into(),
                None
            )
        );
        assert_eq!(
            lnurl_decode("lnurlw:domain.com")?,
            ("domain.com".into(), "https://domain.com".into(), None)
        );
        assert_eq!(
            lnurl_decode("keyauth:domain.com")?,
            ("domain.com".into(), "https://domain.com".into(), None)
        );

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_pay_lud_17() -> Result<(), Box<dyn std::error::Error>> {
        let mock_rest_client = MockRestClient::new();
        let pay_path =
            "/lnurl-pay?session=db945b624265fc7f5a8d77f269f7589d789a771bdfd20e91a3cf6f50382a98d7";
        mock_lnurl_pay_endpoint(&mock_rest_client, None);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        let lnurl_pay_url = format!("lnurlp://localhost{pay_path}");
        if let InputType::LnUrlPay { data: pd, .. } =
            parse_with_rest_client(rest_client.as_ref(), &lnurl_pay_url, None).await?
        {
            assert_eq!(pd.callback, "https://localhost/lnurl-pay/callback/db945b624265fc7f5a8d77f269f7589d789a771bdfd20e91a3cf6f50382a98d7");
            assert_eq!(pd.max_sendable, 16000);
            assert_eq!(pd.min_sendable, 4000);
            assert_eq!(pd.comment_allowed, 0);
            assert_eq!(pd.domain, "localhost");

            assert_eq!(pd.metadata_vec()?.len(), 3);
            assert_eq!(
                pd.metadata_vec()?.first().ok_or("Key not found")?.key,
                "text/plain"
            );
            assert_eq!(
                pd.metadata_vec()?.first().ok_or("Key not found")?.value,
                "WRhtV"
            );
            assert_eq!(
                pd.metadata_vec()?.get(1).ok_or("Key not found")?.key,
                "text/long-desc"
            );
            assert_eq!(
                pd.metadata_vec()?.get(1).ok_or("Key not found")?.value,
                "MBTrTiLCFS"
            );
            assert_eq!(
                pd.metadata_vec()?.get(2).ok_or("Key not found")?.key,
                "image/png;base64"
            );
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_withdraw_lud_17() -> Result<(), Box<dyn std::error::Error>> {
        let mock_rest_client = MockRestClient::new();
        let withdraw_path = "/lnurl-withdraw?session=e464f841c44dbdd86cee4f09f4ccd3ced58d2e24f148730ec192748317b74538";
        mock_lnurl_withdraw_endpoint(&mock_rest_client, None);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        if let InputType::LnUrlWithdraw { data: wd } = parse_with_rest_client(
            rest_client.as_ref(),
            &format!("lnurlw://localhost{withdraw_path}"),
            None,
        )
        .await?
        {
            assert_eq!(wd.callback, "https://localhost/lnurl-withdraw/callback/e464f841c44dbdd86cee4f09f4ccd3ced58d2e24f148730ec192748317b74538");
            assert_eq!(
                wd.k1,
                "37b4c919f871c090830cc47b92a544a30097f03430bc39670b8ec0da89f01a81"
            );
            assert_eq!(wd.min_withdrawable, 3000);
            assert_eq!(wd.max_withdrawable, 12000);
            assert_eq!(wd.default_description, "sample withdraw");
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_auth_lud_17() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);
        let auth_path = "/lnurl-login?tag=login&k1=1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822";

        if let InputType::LnUrlAuth { data: ad } = parse_with_rest_client(
            rest_client.as_ref(),
            &format!("keyauth://localhost{auth_path}"),
            None,
        )
        .await?
        {
            assert_eq!(
                ad.k1,
                "1a855505699c3e01be41bddd32007bfcc5ff93505dec0cbca64b4b8ff590b822"
            );
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_pay_lud_17_error() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let pay_path = "/lnurl-pay?session=paylud17error";
        let expected_error_msg = "test pay error";
        mock_lnurl_pay_endpoint(&mock_rest_client, Some(expected_error_msg.to_string()));
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        if let InputType::LnUrlError { data: msg } = parse_with_rest_client(
            rest_client.as_ref(),
            &format!("lnurlp://localhost{pay_path}"),
            None,
        )
        .await?
        {
            assert_eq!(msg.reason, expected_error_msg);
            return Ok(());
        }

        Err(anyhow!("Unrecognized input type"))
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_lnurl_withdraw_lud_17_error() -> Result<()> {
        let mock_rest_client = MockRestClient::new();
        let withdraw_path = "/lnurl-withdraw?session=withdrawlud17error";
        let expected_error_msg = "test withdraw error";
        mock_lnurl_withdraw_endpoint(&mock_rest_client, Some(expected_error_msg.to_string()));
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        if let InputType::LnUrlError { data: msg } = parse_with_rest_client(
            rest_client.as_ref(),
            &format!("lnurlw://localhost{withdraw_path}"),
            None,
        )
        .await?
        {
            assert_eq!(msg.reason, expected_error_msg);
            return Ok(());
        }

        Err(anyhow!("Unrecognized input type"))
    }

    fn mock_external_parser(
        mock_rest_client: &MockRestClient,
        response_body: String,
        status_code: u16,
    ) {
        mock_rest_client.add_response(MockResponse::new(status_code, response_body));
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_external_parsing_lnurlp_first_response() -> Result<(), Box<dyn std::error::Error>>
    {
        let mock_rest_client = MockRestClient::new();
        let input = "123provider.domain32/1";
        let response = json!(
        {
            "callback": "callback_url",
            "minSendable": 57000,
            "maxSendable": 57000,
            "metadata": "[[\"text/plain\", \"External payment\"]]",
            "tag": "payRequest"
        })
        .to_string();
        mock_external_parser(&mock_rest_client, response, 200);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        let parsers = vec![ExternalInputParser {
            provider_id: "id".to_string(),
            input_regex: "(.*)(provider.domain)(.*)".to_string(),
            parser_url: "http://127.0.0.1:8080/<input>".to_string(),
        }];

        let input_type =
            parse_with_rest_client(rest_client.as_ref(), input, Some(&parsers)).await?;
        if let InputType::LnUrlPay { data, .. } = input_type {
            assert_eq!(data.callback, "callback_url");
            assert_eq!(data.max_sendable, 57000);
            assert_eq!(data.min_sendable, 57000);
            assert_eq!(data.comment_allowed, 0);

            assert_eq!(data.metadata_vec()?.len(), 1);
            assert_eq!(
                data.metadata_vec()?.first().ok_or("Key not found")?.key,
                "text/plain"
            );
            assert_eq!(
                data.metadata_vec()?.first().ok_or("Key not found")?.value,
                "External payment"
            );
        } else {
            panic!("Expected LnUrlPay, got {:?}", input_type);
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_external_parsing_bitcoin_address_and_bolt11(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mock_rest_client = MockRestClient::new();
        // Bitcoin parsing endpoint
        let bitcoin_input = "123bitcoin.address.provider32/1";
        let bitcoin_address = "1andreas3batLhQa2FawWjeyjCqyBzypd".to_string();
        mock_external_parser(&mock_rest_client, bitcoin_address.clone(), 200);

        // Bolt11 parsing endpoint
        let bolt11_input = "123bolt11.provider32/1";
        let bolt11 = "lnbc110n1p38q3gtpp5ypz09jrd8p993snjwnm68cph4ftwp22le34xd4r8ftspwshxhmnsdqqxqyjw5qcqpxsp5htlg8ydpywvsa7h3u4hdn77ehs4z4e844em0apjyvmqfkzqhhd2q9qgsqqqyssqszpxzxt9uuqzymr7zxcdccj5g69s8q7zzjs7sgxn9ejhnvdh6gqjcy22mss2yexunagm5r2gqczh8k24cwrqml3njskm548aruhpwssq9nvrvz".to_string();
        mock_external_parser(&mock_rest_client, bolt11.clone(), 200);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        // Set parsers
        let parsers = vec![
            ExternalInputParser {
                provider_id: "bitcoin".to_string(),
                input_regex: "(.*)(bitcoin.address.provider)(.*)".to_string(),
                parser_url: "http://127.0.0.1:8080/<input>".to_string(),
            },
            ExternalInputParser {
                provider_id: "bolt11".to_string(),
                input_regex: "(.*)(bolt11.provider)(.*)".to_string(),
                parser_url: "http://127.0.0.1:8080/<input>".to_string(),
            },
        ];

        // Parse and check results
        let input_type =
            parse_with_rest_client(rest_client.as_ref(), bitcoin_input, Some(&parsers)).await?;
        if let InputType::BitcoinAddress { address } = input_type {
            assert_eq!(address.address, bitcoin_address);
        } else {
            panic!("Expected BitcoinAddress, got {:?}", input_type);
        }

        let input_type =
            parse_with_rest_client(rest_client.as_ref(), bolt11_input, Some(&parsers)).await?;
        if let InputType::Bolt11 { invoice } = input_type {
            assert_eq!(invoice.bolt11, bolt11);
        } else {
            panic!("Expected Bolt11, got {:?}", input_type);
        }

        Ok(())
    }

    #[breez_sdk_macros::async_test_all]
    async fn test_external_parsing_error() -> Result<(), Box<dyn std::error::Error>> {
        let mock_rest_client = MockRestClient::new();
        let input = "123provider.domain.error32/1";
        let response = "Unrecognized input".to_string();
        mock_external_parser(&mock_rest_client, response, 400);
        let rest_client: Arc<dyn RestClient> = Arc::new(mock_rest_client);

        let parsers = vec![ExternalInputParser {
            provider_id: "id".to_string(),
            input_regex: "(.*)(provider.domain)(.*)".to_string(),
            parser_url: "http://127.0.0.1:8080/<input>".to_string(),
        }];

        let result = parse_with_rest_client(rest_client.as_ref(), input, Some(&parsers)).await;

        assert!(matches!(result, Err(e) if e.to_string() == "Unrecognized input type"));

        Ok(())
    }
}