mod dns;
mod error;
mod input;
mod model;
mod utils;

use std::collections::HashMap;

use crate::input::{Bip21, InputType, PaymentMethod, PaymentMethodType, PaymentRequest};
use error::{ParseAndPickError, PickPaymentMethodError};

use model::{
    BitcoinPaymentMethod, LightningPaymentMethod, LightningPaymentRequest, LnurlPaymentMethod,
    MilliSatoshi, PickedInputType, PickedPaymentMethod,
};

pub struct BreezServicesCore {}

impl Default for BreezServicesCore {
    fn default() -> Self {
        Self::new()
    }
}

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
        let input = crate::input::parse(input).await?;
        Ok(match input {
            InputType::LnurlAuth(lnurl_auth) => PickedInputType::LnurlAuth(lnurl_auth),
            InputType::PaymentRequest(req) => {
                let payment_method = self.pick_payment_method(req, supported)?;
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
    pub fn pick_payment_method(
        &self,
        payment_request: PaymentRequest,
        supported: &[PaymentMethodType],
    ) -> Result<PickedPaymentMethod, PickPaymentMethodError> {
        // TODO: Liquid should unpack the magic routing hint for example to send to a liquid address directly.
        Ok(match payment_request {
            PaymentRequest::Bip21(bip_21) => expand_bip_21(&bip_21, supported)?,
            PaymentRequest::PaymentMethod(payment_method) => expand_payment_method(payment_method),
        })
    }
}

/// Picks a payment method from the given BIP21, based on the supported payment methods.
fn expand_bip_21(
    bip_21: &Bip21,
    supported: &[PaymentMethodType],
) -> Result<PickedPaymentMethod, PickPaymentMethodError> {
    let mut payment_methods = HashMap::new();
    for payment_method in &bip_21.payment_methods {
        payment_methods
            .entry(payment_method.get_type())
            .or_insert_with(|| payment_method.clone());
    }

    for supported_method in supported {
        let Some(payment_method) = payment_methods.remove(supported_method) else {
            continue;
        };

        return Ok(expand_payment_method(payment_method));
    }

    Err(PickPaymentMethodError::Unsupported)
}

fn expand_payment_method(payment_method: PaymentMethod) -> PickedPaymentMethod {
    match payment_method {
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
        PaymentMethod::LightningAddress(lightning_address) => {
            PickedPaymentMethod::LnurlPay(LnurlPaymentMethod::LightningAddress(lightning_address))
        }
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
    }
}

#[cfg(test)]
mod test_utils;
