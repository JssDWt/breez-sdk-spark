pub mod error;
pub mod lnurl;
pub mod model;

use std::collections::HashMap;

use breez_sdk_input::{
    Bip21, Bip353, Bolt11Invoice, Bolt12Invoice, Bolt12Offer, InputType, PaymentMethod,
    PaymentMethodType, PaymentRequest,
};
use breez_sdk_internal::utils::Arc;
use error::{ParseAndPickError, PickPaymentMethodError};

use model::*;

struct UnpackedPaymentMethod<P> {
    pub payment_method: P,
    pub bip_21_uri: Option<String>,
    pub bip_353_address: Option<String>,
}

impl<P> PaymentMethodSource<P> {
    fn unpack(self) -> UnpackedPaymentMethod<P> {
        match self {
            PaymentMethodSource::Bip21(bip21_source) => UnpackedPaymentMethod {
                payment_method: bip21_source.payment_method,
                bip_21_uri: Some(bip21_source.bip_21_uri),
                bip_353_address: None,
            },
            PaymentMethodSource::Bip353(bip353_source) => UnpackedPaymentMethod {
                payment_method: bip353_source.bip_21.payment_method,
                bip_21_uri: Some(bip353_source.bip_21.bip_21_uri),
                bip_353_address: Some(bip353_source.bip_353_uri),
            },
            PaymentMethodSource::Plain(payment_method) => UnpackedPaymentMethod {
                payment_method,
                bip_21_uri: None,
                bip_353_address: None,
            },
        }
    }
}

pub enum AmountError {}
pub trait AmountMapper<A> {
    fn map_sat(&self, amount: u64) -> A;
    fn map_msat(&self, amount: u64) -> A;
}

pub struct BreezServicesCore<A> {
    amount_mapper: Arc<dyn AmountMapper<A>>,
}

impl<A> BreezServicesCore<A> {
    pub fn new(amount_mapper: Arc<dyn AmountMapper<A>>) -> Self {
        BreezServicesCore { amount_mapper }
    }

    /// Parses the input string and picks a payment method based on the supported payment methods.
    pub async fn parse_and_pick(
        &self,
        input: &str,
        supported: &[PaymentMethodType],
    ) -> Result<SourcedInputType<A>, ParseAndPickError> {
        let input = breez_sdk_input::parse(input).await?;
        Ok(match input {
            InputType::LnurlAuth(lnurl_auth) => SourcedInputType::LnurlAuth(lnurl_auth),
            InputType::PaymentRequest(req) => {
                let payment_method = self.pick_payment_method(req, supported).await?;
                SourcedInputType::PaymentMethod(payment_method)
            }
            InputType::ReceiveRequest(receive_request) => {
                SourcedInputType::ReceiveRequest(receive_request)
            }
            InputType::Url(url) => SourcedInputType::Url(url),
        })
    }

    /// Picks a payment method from the given payment request, based on the supported payment methods.
    /// Typically used after parsing a payment request with the general input parser.
    pub async fn pick_payment_method(
        &self,
        payment_request: PaymentRequest,
        supported: &[PaymentMethodType],
    ) -> Result<SourcedPaymentMethod<A>, PickPaymentMethodError> {
        // TODO: Liquid should unpack the magic routing hint for example to send to a liquid address directly.
        Ok(match payment_request {
            PaymentRequest::Bip21(bip_21) => self.expand_bip_21(bip_21, &supported).await?,
            PaymentRequest::Bip353(bip_353) => self.expand_bip_353(bip_353, &supported).await?,
            PaymentRequest::Plain(payment_method) => match payment_method {
                PaymentMethod::BitcoinAddress(bitcoin_address) => {
                    SourcedPaymentMethod::Bitcoin(PaymentMethodSource::Plain(
                        BitcoinPaymentMethod::BitcoinAddress(bitcoin_address),
                    ))
                }
                PaymentMethod::Bolt11Invoice(bolt11_invoice) => {
                    SourcedPaymentMethod::Lightning(PaymentMethodSource::Plain(
                        self.bolt11_invoice_to_lightning_payment_request(bolt11_invoice),
                    ))
                }
                PaymentMethod::Bolt12Invoice(bolt12_invoice) => {
                    SourcedPaymentMethod::Lightning(PaymentMethodSource::Plain(
                        self.bolt12_invoice_to_lightning_payment_request(bolt12_invoice),
                    ))
                }
                PaymentMethod::Bolt12Offer(bolt12_offer) => {
                    SourcedPaymentMethod::Lightning(PaymentMethodSource::Plain(
                        self.bolt12_offer_to_lightning_payment_request(bolt12_offer),
                    ))
                }
                PaymentMethod::LightningAddress(lightning_address) => {
                    SourcedPaymentMethod::LnurlPay(PaymentMethodSource::Plain(
                        LnurlPaymentMethod::LightningAddress(lightning_address),
                    ))
                }
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
            },
        })
    }

    /// Picks a payment method from the given BIP21, based on the supported payment methods.
    async fn expand_bip_21(
        &self,
        bip_21: Bip21,
        supported: &[PaymentMethodType],
    ) -> Result<SourcedPaymentMethod<A>, PickPaymentMethodError> {
        let mut payment_methods = HashMap::new();
        for payment_method in &bip_21.payment_methods {
            if !payment_methods.contains_key(&payment_method.get_type()) {
                payment_methods.insert(payment_method.get_type(), payment_method.clone());
            }
        }

        for supported_method in supported {
            let payment_method = match payment_methods.get(supported_method) {
                Some(payment_method) => payment_method,
                None => continue,
            };

            return Ok(match payment_method {
                PaymentMethod::BitcoinAddress(bitcoin_address) => {
                    SourcedPaymentMethod::Bitcoin(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: BitcoinPaymentMethod::BitcoinAddress(
                            bitcoin_address.clone(),
                        ),
                    }))
                }
                PaymentMethod::Bolt11Invoice(bolt11_invoice) => {
                    SourcedPaymentMethod::Lightning(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: self
                            .bolt11_invoice_to_lightning_payment_request(bolt11_invoice.clone()),
                    }))
                }
                PaymentMethod::Bolt12Invoice(bolt12_invoice) => {
                    SourcedPaymentMethod::Lightning(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: self
                            .bolt12_invoice_to_lightning_payment_request(bolt12_invoice.clone()),
                    }))
                }
                PaymentMethod::Bolt12Offer(bolt12_offer) => {
                    SourcedPaymentMethod::Lightning(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: self
                            .bolt12_offer_to_lightning_payment_request(bolt12_offer.clone()),
                    }))
                }
                PaymentMethod::LightningAddress(lightning_address) => {
                    SourcedPaymentMethod::LnurlPay(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: LnurlPaymentMethod::LightningAddress(
                            lightning_address.clone(),
                        ),
                    }))
                }
                PaymentMethod::LiquidAddress(liquid_address) => {
                    SourcedPaymentMethod::LiquidAddress(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: liquid_address.clone(),
                    }))
                }
                PaymentMethod::LnurlPay(lnurl_pay_request) => {
                    SourcedPaymentMethod::LnurlPay(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: LnurlPaymentMethod::LnurlPay(lnurl_pay_request.clone()),
                    }))
                }
                PaymentMethod::SilentPaymentAddress(silent_payment_address) => {
                    SourcedPaymentMethod::Bitcoin(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: BitcoinPaymentMethod::SilentPaymentAddress(
                            silent_payment_address.clone(),
                        ),
                    }))
                }
            });
        }

        Err(PickPaymentMethodError::Unsupported)
    }

    /// Picks a payment method from the given BIP21, based on the supported payment methods.
    async fn expand_bip_353(
        &self,
        bip_353: Bip353,
        supported: &[PaymentMethodType],
    ) -> Result<SourcedPaymentMethod<A>, PickPaymentMethodError> {
        let bip_21 = self.expand_bip_21(bip_353.bip_21, supported).await?;

        Ok(match bip_21 {
            SourcedPaymentMethod::Bitcoin(PaymentMethodSource::Bip21(bip_21)) => {
                SourcedPaymentMethod::Bitcoin(PaymentMethodSource::Bip353(Bip353Source {
                    bip_353_uri: bip_353.address,
                    bip_21,
                }))
            }
            SourcedPaymentMethod::Lightning(PaymentMethodSource::Bip21(bip_21)) => {
                SourcedPaymentMethod::Lightning(PaymentMethodSource::Bip353(Bip353Source {
                    bip_353_uri: bip_353.address,
                    bip_21,
                }))
            }
            SourcedPaymentMethod::LnurlPay(PaymentMethodSource::Bip21(bip_21)) => {
                SourcedPaymentMethod::LnurlPay(PaymentMethodSource::Bip353(Bip353Source {
                    bip_353_uri: bip_353.address,
                    bip_21,
                }))
            }
            SourcedPaymentMethod::LiquidAddress(PaymentMethodSource::Bip21(bip_21)) => {
                SourcedPaymentMethod::LiquidAddress(PaymentMethodSource::Bip353(Bip353Source {
                    bip_353_uri: bip_353.address,
                    bip_21,
                }))
            }
            SourcedPaymentMethod::Bitcoin(_) => return Err(PickPaymentMethodError::Unsupported),
            SourcedPaymentMethod::Lightning(_) => return Err(PickPaymentMethodError::Unsupported),
            SourcedPaymentMethod::LnurlPay(_) => return Err(PickPaymentMethodError::Unsupported),
            SourcedPaymentMethod::LiquidAddress(_) => {
                return Err(PickPaymentMethodError::Unsupported);
            }
        })
    }

    fn bolt11_invoice_to_lightning_payment_request(
        &self,
        bolt11_invoice: Bolt11Invoice,
    ) -> LightningPaymentRequest<A> {
        LightningPaymentRequest {
            min_amount: self
                .amount_mapper
                .map_msat(bolt11_invoice.amount_msat.unwrap_or(0)), // TODO: Set min amount to minimum payable amount.
            max_amount: self
                .amount_mapper
                .map_msat(bolt11_invoice.amount_msat.unwrap_or(u64::MAX)), // TODO: Set max amount to balance.
            method: LightningPaymentMethod::Bolt11Invoice(bolt11_invoice),
        }
    }

    fn bolt12_invoice_to_lightning_payment_request(
        &self,
        bolt12_invoice: Bolt12Invoice,
    ) -> LightningPaymentRequest<A> {
        LightningPaymentRequest {
            min_amount: self.amount_mapper.map_msat(bolt12_invoice.amount_msat),
            max_amount: self.amount_mapper.map_msat(bolt12_invoice.amount_msat),
            method: LightningPaymentMethod::Bolt12Invoice(bolt12_invoice),
        }
    }

    fn bolt12_offer_to_lightning_payment_request(
        &self,
        bolt12_offer: Bolt12Offer,
    ) -> LightningPaymentRequest<A> {
        LightningPaymentRequest {
            min_amount: self.amount_mapper.map_msat(0), // TODO: Set min amount to minimum payable amount.
            max_amount: self.amount_mapper.map_msat(u64::MAX), // TODO: Set max amount to balance.
            method: LightningPaymentMethod::Bolt12Offer(bolt12_offer),
        }
    }
}
