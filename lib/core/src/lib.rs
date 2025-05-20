mod error;
mod models;

use std::{collections::HashMap, sync::Arc};

use breez_sdk_input::{
    Bip21, Bip353, InputType, Network, PaymentMethod, PaymentMethodType, PaymentRequest,
};
use error::{
    ParseAndPickError, PickPaymentMethodError, PrepareSendBitcoinAddressError,
    PrepareSendBolt11InvoiceError, PrepareSendBolt12InvoiceError, PrepareSendBolt12OfferError,
    PrepareSendLightningAddressError, PrepareSendLiquidAddressError, PrepareSendLnurlPayError,
};
use models::{
    Bip21Source, Bip353Source, PaymentMethodSource, PrepareSendBitcoinAddressRequest,
    PrepareSendBitcoinAddressResponse, PrepareSendBolt11InvoiceRequest,
    PrepareSendBolt11InvoiceResponse, PrepareSendBolt12InvoiceRequest,
    PrepareSendBolt12InvoiceResponse, PrepareSendBolt12OfferRequest,
    PrepareSendBolt12OfferResponse, PrepareSendLightningAddressRequest,
    PrepareSendLightningAddressResponse, PrepareSendLiquidAddressRequest,
    PrepareSendLiquidAddressResponse, PrepareSendLnurlPayRequest, PrepareSendLnurlPayResponse,
    SourcedInputType, SourcedPaymentMethod,
};

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

pub trait BreezServicesImpl: Send + Sync {
    /// Returns the payment methods supported by this implementation, ordered by preference.
    fn get_payment_methods(&self) -> Vec<PaymentMethodType>;
}

pub struct BreezServices<B> {
    network: Network,
    implementation: Arc<B>,
}

pub enum PaymentResult {}

impl<B> BreezServices<B>
where
    B: BreezServicesImpl,
{
    pub async fn parse_and_pick(&self, input: &str) -> Result<SourcedInputType, ParseAndPickError> {
        let input = breez_sdk_input::parse(input).await?;
        Ok(match input {
            InputType::LnurlAuth(lnurl_auth) => SourcedInputType::LnurlAuth(lnurl_auth),
            InputType::PaymentRequest(req) => {
                let payment_method = self.pick_payment_method(req).await?;
                SourcedInputType::PaymentMethod(payment_method)
            }
            InputType::ReceiveMethod(receive_method) => {
                SourcedInputType::ReceiveMethod(receive_method)
            }
            InputType::Url(url) => SourcedInputType::Url(url),
        })
    }

    /// Picks a payment method from the given payment request, based on the supported payment methods.
    /// Typically used after parsing a payment request with the general input parser.
    pub async fn pick_payment_method(
        &self,
        payment_request: PaymentRequest,
    ) -> Result<SourcedPaymentMethod, PickPaymentMethodError> {
        let supported = self.implementation.get_payment_methods();
        Ok(match payment_request {
            PaymentRequest::Bip21(bip_21) => self.expand_bip_21(bip_21, &supported).await?,
            PaymentRequest::Bip353(bip_353) => self.expand_bip_353(bip_353, &supported).await?,
            PaymentRequest::Plain(payment_method) => payment_method.into(),
        })
    }

    pub async fn prepare_send_bitcoin_address(
        &self,
        req: PrepareSendBitcoinAddressRequest,
    ) -> Result<PrepareSendBitcoinAddressResponse, PrepareSendBitcoinAddressError> {
        let unpacked = req.bitcoin_address.unpack();
        if unpacked.payment_method.network != self.network {
            return Err(PrepareSendBitcoinAddressError::InvalidNetwork);
        }

        todo!()
    }

    pub async fn prepare_send_bolt11_invoice(
        &self,
        bolt11_invoice: PrepareSendBolt11InvoiceRequest,
    ) -> Result<PrepareSendBolt11InvoiceResponse, PrepareSendBolt11InvoiceError> {
        todo!()
    }

    pub async fn prepare_send_bolt12_invoice(
        &self,
        bolt12_invoice: PrepareSendBolt12InvoiceRequest,
    ) -> Result<PrepareSendBolt12InvoiceResponse, PrepareSendBolt12InvoiceError> {
        todo!()
    }

    pub async fn prepare_send_bolt12_offer(
        &self,
        bolt12_offer: PrepareSendBolt12OfferRequest,
    ) -> Result<PrepareSendBolt12OfferResponse, PrepareSendBolt12OfferError> {
        todo!()
    }

    pub async fn prepare_send_lightning_address(
        &self,
        lightning_address: PrepareSendLightningAddressRequest,
    ) -> Result<PrepareSendLightningAddressResponse, PrepareSendLightningAddressError> {
        todo!()
    }

    pub async fn prepare_send_liquid_address(
        &self,
        liquid_address: PrepareSendLiquidAddressRequest,
    ) -> Result<PrepareSendLiquidAddressResponse, PrepareSendLiquidAddressError> {
        todo!()
    }

    pub async fn prepare_send_lnurl_pay(
        &self,
        lnurl_pay_request: PrepareSendLnurlPayRequest,
    ) -> Result<PrepareSendLnurlPayResponse, PrepareSendLnurlPayError> {
        todo!()
    }
}

impl<B> BreezServices<B>
where
    B: BreezServicesImpl,
{
    /// Picks a payment method from the given BIP21, based on the supported payment methods.
    async fn expand_bip_21(
        &self,
        bip_21: Bip21,
        supported: &[PaymentMethodType],
    ) -> Result<SourcedPaymentMethod, PickPaymentMethodError> {
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
                    SourcedPaymentMethod::BitcoinAddress(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: bitcoin_address.clone(),
                    }))
                }
                PaymentMethod::Bolt11Invoice(bolt11_invoice) => {
                    SourcedPaymentMethod::Bolt11Invoice(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: bolt11_invoice.clone(),
                    }))
                }
                PaymentMethod::Bolt12Invoice(bolt12_invoice) => {
                    SourcedPaymentMethod::Bolt12Invoice(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: bolt12_invoice.clone(),
                    }))
                }
                PaymentMethod::Bolt12Offer(bolt12_offer) => {
                    SourcedPaymentMethod::Bolt12Offer(PaymentMethodSource::Bip21(Bip21Source {
                        bip_21_uri: bip_21.bip_21,
                        payment_method: bolt12_offer.clone(),
                    }))
                }
                PaymentMethod::LightningAddress(lightning_address) => {
                    SourcedPaymentMethod::LightningAddress(PaymentMethodSource::Bip21(
                        Bip21Source {
                            bip_21_uri: bip_21.bip_21,
                            payment_method: lightning_address.clone(),
                        },
                    ))
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
                        payment_method: lnurl_pay_request.clone(),
                    }))
                }
                PaymentMethod::SilentPaymentAddress(silent_payment_address) => {
                    SourcedPaymentMethod::SilentPaymentAddress(PaymentMethodSource::Bip21(
                        Bip21Source {
                            bip_21_uri: bip_21.bip_21,
                            payment_method: silent_payment_address.clone(),
                        },
                    ))
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
    ) -> Result<SourcedPaymentMethod, PickPaymentMethodError> {
        let mut payment_methods = HashMap::new();
        for payment_method in &bip_353.bip_21.payment_methods {
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
                    SourcedPaymentMethod::BitcoinAddress(PaymentMethodSource::Bip353(
                        Bip353Source {
                            bip_353_uri: bip_353.address,
                            bip_21: Bip21Source {
                                bip_21_uri: bip_353.bip_21.bip_21,
                                payment_method: bitcoin_address.clone(),
                            },
                        },
                    ))
                }
                PaymentMethod::Bolt11Invoice(bolt11_invoice) => {
                    SourcedPaymentMethod::Bolt11Invoice(PaymentMethodSource::Bip353(Bip353Source {
                        bip_353_uri: bip_353.address,
                        bip_21: Bip21Source {
                            bip_21_uri: bip_353.bip_21.bip_21,
                            payment_method: bolt11_invoice.clone(),
                        },
                    }))
                }
                PaymentMethod::Bolt12Invoice(bolt12_invoice) => {
                    SourcedPaymentMethod::Bolt12Invoice(PaymentMethodSource::Bip353(Bip353Source {
                        bip_353_uri: bip_353.address,
                        bip_21: Bip21Source {
                            bip_21_uri: bip_353.bip_21.bip_21,
                            payment_method: bolt12_invoice.clone(),
                        },
                    }))
                }
                PaymentMethod::Bolt12Offer(bolt12_offer) => {
                    SourcedPaymentMethod::Bolt12Offer(PaymentMethodSource::Bip353(Bip353Source {
                        bip_353_uri: bip_353.address,
                        bip_21: Bip21Source {
                            bip_21_uri: bip_353.bip_21.bip_21,
                            payment_method: bolt12_offer.clone(),
                        },
                    }))
                }
                PaymentMethod::LightningAddress(lightning_address) => {
                    SourcedPaymentMethod::LightningAddress(PaymentMethodSource::Bip353(
                        Bip353Source {
                            bip_353_uri: bip_353.address,
                            bip_21: Bip21Source {
                                bip_21_uri: bip_353.bip_21.bip_21,
                                payment_method: lightning_address.clone(),
                            },
                        },
                    ))
                }
                PaymentMethod::LiquidAddress(liquid_address) => {
                    SourcedPaymentMethod::LiquidAddress(PaymentMethodSource::Bip353(Bip353Source {
                        bip_353_uri: bip_353.address,
                        bip_21: Bip21Source {
                            bip_21_uri: bip_353.bip_21.bip_21,
                            payment_method: liquid_address.clone(),
                        },
                    }))
                }
                PaymentMethod::LnurlPay(lnurl_pay_request) => {
                    SourcedPaymentMethod::LnurlPay(PaymentMethodSource::Bip353(Bip353Source {
                        bip_353_uri: bip_353.address,
                        bip_21: Bip21Source {
                            bip_21_uri: bip_353.bip_21.bip_21,
                            payment_method: lnurl_pay_request.clone(),
                        },
                    }))
                }
                PaymentMethod::SilentPaymentAddress(silent_payment_address) => {
                    SourcedPaymentMethod::SilentPaymentAddress(PaymentMethodSource::Bip353(
                        Bip353Source {
                            bip_353_uri: bip_353.address,
                            bip_21: Bip21Source {
                                bip_21_uri: bip_353.bip_21.bip_21,
                                payment_method: silent_payment_address.clone(),
                            },
                        },
                    ))
                }
            });
        }

        Err(PickPaymentMethodError::Unsupported)
    }
}
