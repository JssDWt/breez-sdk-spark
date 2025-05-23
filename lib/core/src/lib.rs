pub mod error;
pub mod lnurl;
pub mod model;

use std::collections::HashMap;

use breez_sdk_input::{Bip21, InputType, PaymentMethod, PaymentMethodType, PaymentRequest};
use error::{ParseAndPickError, PickPaymentMethodError};

use model::*;

pub struct BreezServicesCore {}

impl BreezServicesCore {
    pub fn new() -> Self {
        Self {}
    }

    /// Parses the input string and picks a payment method based on the supported payment methods.
    pub async fn parse_and_pick(
        &self,
        input: &str,
        supported: &[PaymentMethodType],
    ) -> Result<PickedInputType, ParseAndPickError> {
        let input = breez_sdk_input::parse(input).await?;
        Ok(match input {
            InputType::LnurlAuth(lnurl_auth) => PickedInputType::LnurlAuth(lnurl_auth),
            InputType::PaymentRequest(req) => {
                let payment_method = self.pick_payment_method(req, supported).await?;
                PickedInputType::PaymentMethod(payment_method)
            }
            InputType::ReceiveRequest(receive_request) => {
                PickedInputType::ReceiveRequest(receive_request)
            }
            InputType::Url(url) => PickedInputType::Url(url),
        })
    }

    /// Picks a payment method from the given payment request, based on the supported payment methods.
    /// Typically used after parsing a payment request with the general input parser.
    pub async fn pick_payment_method(
        &self,
        payment_request: PaymentRequest,
        supported: &[PaymentMethodType],
    ) -> Result<PickedPaymentMethod, PickPaymentMethodError> {
        // TODO: Liquid should unpack the magic routing hint for example to send to a liquid address directly.
        Ok(match payment_request {
            PaymentRequest::Bip21(bip_21) => self.expand_bip_21(bip_21, &supported).await?,
            PaymentRequest::PaymentMethod(payment_method) => {
                self.expand_payment_method(payment_method).await?
            }
        })
    }

    async fn expand_payment_method(
        &self,
        payment_method: PaymentMethod,
    ) -> Result<PickedPaymentMethod, PickPaymentMethodError> {
        Ok(match payment_method {
            PaymentMethod::BitcoinAddress(bitcoin_address) => {
                PickedPaymentMethod::Bitcoin(BitcoinPaymentMethod::BitcoinAddress(bitcoin_address))
            }
            PaymentMethod::Bolt11Invoice(bolt11_invoice) => {
                PickedPaymentMethod::Lightning(LightningPaymentRequest {
                    max_amount: MilliSatoshi(bolt11_invoice.amount_msat.unwrap_or(u64::MAX)), // TODO: Set max amount to sane value.
                    min_amount: MilliSatoshi(bolt11_invoice.amount_msat.unwrap_or(0)), // TODO: Set min amount to minimum payable amount.
                    method: LightningPaymentMethod::Bolt11Invoice(bolt11_invoice.invoice),
                })
            }
            PaymentMethod::Bolt12Invoice(bolt12_invoice) => {
                PickedPaymentMethod::Lightning(LightningPaymentRequest {
                    max_amount: MilliSatoshi(bolt12_invoice.amount_msat),
                    min_amount: MilliSatoshi(bolt12_invoice.amount_msat),
                    method: LightningPaymentMethod::Bolt12Invoice(bolt12_invoice.invoice),
                })
            }
            PaymentMethod::Bolt12Offer(bolt12_offer) => {
                PickedPaymentMethod::Lightning(LightningPaymentRequest {
                    max_amount: MilliSatoshi(u64::MAX), // TODO: Set max amount to sane value.
                    min_amount: MilliSatoshi(0), // TODO: Set min amount to minimum payable amount.
                    method: LightningPaymentMethod::Bolt12Offer(bolt12_offer.offer),
                })
            }
            PaymentMethod::LightningAddress(lightning_address) => PickedPaymentMethod::LnurlPay(
                LnurlPaymentMethod::LightningAddress(lightning_address),
            ),
            PaymentMethod::LiquidAddress(liquid_address) => {
                PickedPaymentMethod::LiquidAddress(liquid_address)
            }
            PaymentMethod::LnurlPay(lnurl_pay_request) => {
                PickedPaymentMethod::LnurlPay(LnurlPaymentMethod::LnurlPay(lnurl_pay_request))
            }
            PaymentMethod::SilentPaymentAddress(silent_payment_address) => {
                PickedPaymentMethod::Bitcoin(BitcoinPaymentMethod::SilentPaymentAddress(
                    silent_payment_address,
                ))
            }
        })
    }

    /// Picks a payment method from the given BIP21, based on the supported payment methods.
    async fn expand_bip_21(
        &self,
        bip_21: Bip21,
        supported: &[PaymentMethodType],
    ) -> Result<PickedPaymentMethod, PickPaymentMethodError> {
        let mut payment_methods = HashMap::new();
        for payment_method in &bip_21.payment_methods {
            if !payment_methods.contains_key(&payment_method.get_type()) {
                payment_methods.insert(payment_method.get_type(), payment_method.clone());
            }
        }

        for supported_method in supported {
            let payment_method = match payment_methods.remove(supported_method) {
                Some(payment_method) => payment_method,
                None => continue,
            };

            return self.expand_payment_method(payment_method).await;
        }

        Err(PickPaymentMethodError::Unsupported)
    }
}
