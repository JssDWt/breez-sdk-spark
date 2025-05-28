use std::collections::HashMap;

use breez_sdk_common::input::{Bip21, InputType, PaymentMethod, PaymentMethodType, PaymentRequest};

use crate::{
    error::{
        ParseAndPickError, PickPaymentMethodError, PrepareReceivePaymentError,
        PrepareSendBitcoinError, PrepareSendLightningError, PrepareSendLiquidAddressError,
        PrepareSendLnurlPayError, ReceivePaymentError, SendBitcoinError, SendLightningError,
        SendLiquidAddressError, SendLnurlPayError,
    },
    model::{
        BitcoinPaymentMethod, LightningPaymentMethod, LightningPaymentRequest, LnurlPaymentMethod,
        MilliSatoshi, PickedInputType, PickedPaymentMethod, PrepareReceivePaymentRequest,
        PrepareReceivePaymentResponse, PrepareSendBitcoinRequest, PrepareSendBitcoinResponse,
        PrepareSendLightningRequest, PrepareSendLightningResponse, PrepareSendLiquidAddressRequest,
        PrepareSendLiquidAddressResponse, PrepareSendLnurlPayRequest, PrepareSendLnurlPayResponse,
        ReceivePaymentRequest, ReceivePaymentResponse, SendBitcoinRequest, SendBitcoinResponse,
        SendLightningRequest, SendLightningResponse, SendLiquidAddressRequest,
        SendLiquidAddressResponse, SendLnurlPayRequest, SendLnurlPayResponse,
    },
};

pub struct BreezServices {
    supported: Vec<PaymentMethodType>,
}

impl BreezServices {
    /// Parses the input string and picks a payment method based on the supported payment methods.
    pub async fn parse_and_pick(&self, input: &str) -> Result<PickedInputType, ParseAndPickError> {
        let input = breez_sdk_common::input::parse(input).await?;
        Ok(match input {
            InputType::LnurlAuth(lnurl_auth) => PickedInputType::LnurlAuth(lnurl_auth),
            InputType::PaymentRequest(req) => {
                let payment_method = self.pick_payment_method(req).await?;
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
    ) -> Result<PickedPaymentMethod, PickPaymentMethodError> {
        // TODO: Liquid should unpack the magic routing hint for example to send to a liquid address directly.
        Ok(match payment_request {
            PaymentRequest::Bip21(bip_21) => expand_bip_21(&bip_21, &self.supported)?,
            PaymentRequest::PaymentMethod(payment_method) => expand_payment_method(payment_method),
        })
    }

    pub async fn prepare_send_bitcoin(
        &self,
        _req: PrepareSendBitcoinRequest,
    ) -> Result<PrepareSendBitcoinResponse, PrepareSendBitcoinError> {
        todo!()
    }
    pub async fn prepare_send_lightning(
        &self,
        _req: PrepareSendLightningRequest,
    ) -> Result<PrepareSendLightningResponse, PrepareSendLightningError> {
        todo!()
    }
    pub async fn prepare_send_lnurl_pay(
        &self,
        _req: PrepareSendLnurlPayRequest,
    ) -> Result<PrepareSendLnurlPayResponse, PrepareSendLnurlPayError> {
        todo!()
    }
    pub async fn prepare_send_liquid_address(
        &self,
        _req: PrepareSendLiquidAddressRequest,
    ) -> Result<PrepareSendLiquidAddressResponse, PrepareSendLiquidAddressError> {
        todo!()
    }

    pub async fn prepare_receive_payment(
        &self,
        _req: PrepareReceivePaymentRequest,
    ) -> Result<PrepareReceivePaymentResponse, PrepareReceivePaymentError> {
        todo!()
    }

    pub async fn receive_payment(
        &self,
        _req: ReceivePaymentRequest,
    ) -> Result<ReceivePaymentResponse, ReceivePaymentError> {
        todo!()
    }

    pub async fn send_bitcoin(
        &self,
        _req: SendBitcoinRequest,
    ) -> Result<SendBitcoinResponse, SendBitcoinError> {
        todo!()
    }

    pub async fn send_lightning(
        &self,
        _req: SendLightningRequest,
    ) -> Result<SendLightningResponse, SendLightningError> {
        todo!()
    }

    pub async fn send_lnurl_pay(
        &self,
        _req: SendLnurlPayRequest,
    ) -> Result<SendLnurlPayResponse, SendLnurlPayError> {
        todo!()
    }

    pub async fn send_liquid_address(
        &self,
        _req: SendLiquidAddressRequest,
    ) -> Result<SendLiquidAddressResponse, SendLiquidAddressError> {
        todo!()
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
