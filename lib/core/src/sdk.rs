use std::collections::HashMap;

use breez_sdk_common::{
    ensure_sdk,
    fiat::FiatAPI,
    input::{
        Bip21, BitcoinAddress, Bolt11Invoice, Bolt12Invoice, Bolt12InvoiceRequest, Bolt12Offer,
        InputType, LiquidAddress, PaymentMethodType, PaymentRequestSource, RawInputType,
        RawPaymentMethod, SilentPaymentAddress,
    },
    lnurl::auth::perform_lnurl_auth,
    rest::RestClient,
    utils::Arc,
};
use tokio::sync::watch;
use tracing::info;

use crate::{
    Config, ConnectRequest, GetInfoResponse, Network, PrepareSendPaymentError,
    PrepareSendPaymentRequest, PrepareSendPaymentResponse, ReceiveMethod, SendPaymentError,
    SendPaymentRequest, SendPaymentResponse,
    buy::BuyBitcoinApi,
    error::{
        AcceptPaymentProposedFeesError, BuyBitcoinError, ConnectError, FetchFiatCurrenciesError,
        FetchFiatRatesError, FetchOnchainLimitsError, FetchPaymentProposedFeesError,
        FetchRecommendedFeesError, GetInfoError, GetPaymentError, InitializeLoggingError,
        ListPaymentsError, ListRefundablesError, LnurlAuthError, ParseAndPickError,
        PickPaymentMethodError, PrepareBuyBitcoinError, PrepareReceivePaymentError,
        PrepareRefundError, ReceivePaymentError, RefundError, RegisterWebhookError,
        SignMessageError, StopError, UnregisterWebhookError, VerifyMessageError,
    },
    event::EventManager,
    lnurl::LnurlAuthSigner,
    model::{
        AcceptPaymentProposedFeesRequest, AcceptPaymentProposedFeesResponse,
        AddEventListenerResponse, BuyBitcoinRequest, BuyBitcoinResponse,
        FetchFiatCurrenciesResponse, FetchFiatRatesResponse, FetchOnchainLimitsResponse,
        FetchPaymentProposedFeesRequest, FetchPaymentProposedFeesResponse,
        FetchRecommendedFeesResponse, InitializeLoggingRequest, InitializeLoggingResponse,
        ListPaymentsRequest, ListPaymentsResponse, ListRefundablesResponse, LnurlAuthRequest,
        LnurlAuthResponse, Payment, PrepareBuyBitcoinRequest, PrepareBuyBitcoinResponse,
        PrepareReceivePaymentRequest, PrepareReceivePaymentResponse, PrepareRefundRequest,
        PrepareRefundResponse, ReceivePaymentRequest, ReceivePaymentResponse, RefundRequest,
        RefundResponse, RegisterWebhookRequest, RegisterWebhookResponse,
        RemoveEventListenerRequest, SdkEventListener, SignMessageRequest, SignMessageResponse,
        UnregisterWebhookRequest, UnregisterWebhookResponse, VerifyMessageRequest,
        VerifyMessageResponse,
    },
};

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct BreezSdk {
    buy_bitcoin_api: Arc<dyn BuyBitcoinApi>,
    config: Config,
    event_manager: EventManager,
    fiat_api: Arc<dyn FiatAPI>,
    lnurl_auth_signer: Arc<LnurlAuthSigner>,
    rest_client: Arc<dyn RestClient>,
    shutdown_sender: watch::Sender<()>,
    supported: Vec<PaymentMethodType>,
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
pub async fn connect(_req: ConnectRequest) -> Result<BreezSdk, ConnectError> {
    todo!()
}

impl BreezSdk {
    pub async fn initialize_logging(
        _req: InitializeLoggingRequest,
    ) -> Result<InitializeLoggingResponse, InitializeLoggingError> {
        todo!()
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
impl BreezSdk {
    pub async fn accept_payment_proposed_fees(
        &self,
        _req: AcceptPaymentProposedFeesRequest,
    ) -> Result<AcceptPaymentProposedFeesResponse, AcceptPaymentProposedFeesError> {
        todo!()
    }

    pub async fn add_event_listener(
        &self,
        listener: Box<dyn SdkEventListener>,
    ) -> AddEventListenerResponse {
        let listener_id = self.event_manager.add(listener).await;
        AddEventListenerResponse { listener_id }
    }

    // pub async fn backup(&self, _req: BackupRequest) -> Result<BackupResponse, BackupError> {
    //     todo!()
    // }

    pub async fn buy_bitcoin(
        &self,
        req: BuyBitcoinRequest,
    ) -> Result<BuyBitcoinResponse, BuyBitcoinError> {
        let amount_sat = req.prepared.req.amount_sat;
        let amount_msat = amount_sat * 1000;
        self.validate_buy_bitcoin(amount_sat)?;
        let receive_result = self
            .receive_payment(ReceivePaymentRequest {
                prepared: PrepareReceivePaymentResponse {
                    req: PrepareReceivePaymentRequest {
                        amount_msat,
                        message: Some("Bitcoin transfer".to_string()),
                        receive_method: ReceiveMethod::BitcoinAddress,
                    },
                    fee_msat: req.prepared.fee_msat,
                    min_payer_amount_msat: amount_msat,
                    max_payer_amount_msat: amount_msat,
                },
                description: None,
                use_description_hash: None,
            })
            .await?;

        // TODO: The payment request is not a bitcoin address maybe?
        let url = self
            .buy_bitcoin_api
            .buy_bitcoin(
                req.prepared.req.provider,
                receive_result.payment_request,
                amount_sat,
                req.redirect_url,
            )
            .await?;
        Ok(BuyBitcoinResponse { url })
    }

    pub async fn fetch_fiat_currencies(
        &self,
    ) -> Result<FetchFiatCurrenciesResponse, FetchFiatCurrenciesError> {
        let currencies = self.fiat_api.fetch_fiat_currencies().await?;
        Ok(FetchFiatCurrenciesResponse { currencies })
    }

    pub async fn fetch_fiat_rates(&self) -> Result<FetchFiatRatesResponse, FetchFiatRatesError> {
        let rates = self.fiat_api.fetch_fiat_rates().await?;
        Ok(FetchFiatRatesResponse { rates })
    }

    pub async fn fetch_onchain_limits(
        &self,
    ) -> Result<FetchOnchainLimitsResponse, FetchOnchainLimitsError> {
        todo!()
    }

    pub async fn fetch_payment_proposed_fees(
        &self,
        _req: FetchPaymentProposedFeesRequest,
    ) -> Result<FetchPaymentProposedFeesResponse, FetchPaymentProposedFeesError> {
        todo!()
    }

    pub async fn fetch_recommended_fees(
        &self,
    ) -> Result<FetchRecommendedFeesResponse, FetchRecommendedFeesError> {
        todo!()
    }

    pub async fn get_info(&self) -> Result<GetInfoResponse, GetInfoError> {
        todo!()
    }

    pub async fn get_payment(&self, _payment_id: &str) -> Result<Payment, GetPaymentError> {
        todo!()
    }

    pub async fn list_payments(
        &self,
        _req: ListPaymentsRequest,
    ) -> Result<ListPaymentsResponse, ListPaymentsError> {
        todo!()
    }

    pub async fn list_refundables(&self) -> Result<ListRefundablesResponse, ListRefundablesError> {
        todo!()
    }

    pub async fn lnurl_auth(
        &self,
        req: LnurlAuthRequest,
    ) -> Result<LnurlAuthResponse, LnurlAuthError> {
        let callback_status = perform_lnurl_auth(
            self.rest_client.as_ref(),
            &req.data,
            self.lnurl_auth_signer.as_ref(),
        )
        .await?;
        Ok(LnurlAuthResponse { callback_status })
    }

    /// Parses the input string and picks a payment method based on the supported payment methods.
    pub async fn parse(&self, input: &str) -> Result<InputType, ParseAndPickError> {
        let input = breez_sdk_common::input::parse(input).await?;
        Ok(match input {
            RawInputType::Bip21(bip21) => expand_bip_21(bip21, None, &self.supported)?,
            RawInputType::Bip353(bip353) => {
                expand_bip_21(bip353.bip_21, Some(bip353.address), &self.supported)?
            }
            RawInputType::Bolt12InvoiceRequest(details) => {
                InputType::Bolt12InvoiceRequest(Bolt12InvoiceRequest {
                    details,
                    source: PaymentRequestSource::default(),
                })
            }
            RawInputType::LnurlAuth(data) => InputType::LnurlAuth(data),
            RawInputType::LnurlWithdraw(data) => InputType::LnurlWithdraw(data),
            RawInputType::PaymentMethod(data) => {
                expand_payment_method(data, PaymentRequestSource::default())
            }
            RawInputType::Url(data) => InputType::Url(data),
        })
    }

    pub async fn prepare_buy_bitcoin(
        &self,
        req: PrepareBuyBitcoinRequest,
    ) -> Result<PrepareBuyBitcoinResponse, PrepareBuyBitcoinError> {
        let amount_sat = req.amount_sat;
        self.validate_buy_bitcoin(amount_sat)?;

        let prepared = self
            .prepare_receive_payment(PrepareReceivePaymentRequest {
                amount_msat: amount_sat * 1000,
                message: Some("Bitcoin transfer".to_string()),
                receive_method: ReceiveMethod::BitcoinAddress,
            })
            .await?;

        Ok(PrepareBuyBitcoinResponse {
            req,
            fee_msat: prepared.fee_msat,
        })
    }

    pub async fn prepare_send_payment(
        &self,
        _req: PrepareSendPaymentRequest,
    ) -> Result<PrepareSendPaymentResponse, PrepareSendPaymentError> {
        todo!()
    }

    pub async fn prepare_receive_payment(
        &self,
        _req: PrepareReceivePaymentRequest,
    ) -> Result<PrepareReceivePaymentResponse, PrepareReceivePaymentError> {
        todo!()
    }

    pub async fn prepare_refund(
        &self,
        _req: PrepareRefundRequest,
    ) -> Result<PrepareRefundResponse, PrepareRefundError> {
        todo!()
    }

    pub async fn receive_payment(
        &self,
        _req: ReceivePaymentRequest,
    ) -> Result<ReceivePaymentResponse, ReceivePaymentError> {
        todo!()
    }

    pub async fn refund(&self, _req: RefundRequest) -> Result<RefundResponse, RefundError> {
        todo!()
    }

    pub async fn register_wekhook(
        &self,
        _req: RegisterWebhookRequest,
    ) -> Result<RegisterWebhookResponse, RegisterWebhookError> {
        todo!()
    }

    pub async fn remove_event_listener(&self, req: RemoveEventListenerRequest) -> () {
        self.event_manager.remove(req.listener_id).await
    }

    // pub async fn rescan(&self, _req: RescanRequest) -> Result<RescanResponse, RescanError> {
    //     todo!()
    // }

    // pub async fn restore(&self, _req: RestoreRequest) -> Result<RestoreResponse, RestoreError> {
    //     todo!()
    // }

    pub async fn send_payment(
        &self,
        _req: SendPaymentRequest,
    ) -> Result<SendPaymentResponse, SendPaymentError> {
        todo!()
    }

    /// Sign given message with the private key. Returns a zbase encoded signature.
    pub async fn sign_message(
        &self,
        _req: &SignMessageRequest,
    ) -> Result<SignMessageResponse, SignMessageError> {
        todo!()
    }

    /// Stops the SDK's background tasks
    ///
    /// This method stops the background tasks started by the `start()` method.
    /// It should be called before your application terminates to ensure proper cleanup.
    /// When this function returns successfully, the SDK is no longer running and all background tasks have been stopped.
    ///
    /// # Returns
    ///
    /// Result containing either success or a `StopError` if the background task couldn't be stopped
    pub async fn stop(&self) -> Result<(), StopError> {
        self.shutdown_sender
            .send(())
            .map_err(|_| StopError::SendSignalFailed)?;
        self.shutdown_sender.closed().await;
        info!("Breez SDK stopped successfully");
        Ok(())
    }

    // pub async fn sync(&self) -> Result<SyncResponse, SyncError> {
    //     todo!()
    // }

    pub async fn unregister_webhook(
        &self,
        _req: UnregisterWebhookRequest,
    ) -> Result<UnregisterWebhookResponse, UnregisterWebhookError> {
        todo!()
    }

    /// Verifies whether given message was signed by the given pubkey and the signature (zbase encoded) is valid.
    pub async fn verify_message(
        &self,
        _req: &VerifyMessageRequest,
    ) -> Result<VerifyMessageResponse, VerifyMessageError> {
        todo!()
    }
}

impl BreezSdk {
    fn validate_buy_bitcoin(&self, amount_sat: u64) -> Result<(), PrepareBuyBitcoinError> {
        ensure_sdk!(
            self.config.network == Network::Mainnet,
            PrepareBuyBitcoinError::InvalidNetwork
        );
        // The Moonpay API defines BTC amounts as having precision = 5, so only 5 decimals are considered
        ensure_sdk!(
            amount_sat % 1_000 == 0,
            PrepareBuyBitcoinError::InvalidAmount(
                "Can only buy sat amounts that are multiples of 1000".to_string()
            )
        );
        Ok(())
    }
}

/// Picks a payment method from the given BIP21, based on the supported payment methods.
fn expand_bip_21(
    bip_21: Bip21,
    bip_353_address: Option<String>,
    supported: &[PaymentMethodType],
) -> Result<InputType, PickPaymentMethodError> {
    let source = PaymentRequestSource {
        bip_21_uri: Some(bip_21.uri),
        bip_353_address,
    };
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

        return Ok(expand_payment_method(payment_method, source));
    }

    Err(PickPaymentMethodError::Unsupported)
}

fn expand_payment_method(
    payment_method: RawPaymentMethod,
    source: PaymentRequestSource,
) -> InputType {
    match payment_method {
        RawPaymentMethod::BitcoinAddress(details) => {
            InputType::BitcoinAddress(BitcoinAddress { details, source })
        }
        RawPaymentMethod::Bolt11Invoice(details) => {
            InputType::Bolt11Invoice(Bolt11Invoice { details, source })
        }
        RawPaymentMethod::Bolt12Invoice(details) => {
            InputType::Bolt12Invoice(Bolt12Invoice { details, source })
        }
        RawPaymentMethod::Bolt12Offer(details) => {
            InputType::Bolt12Offer(Bolt12Offer { details, source })
        }
        RawPaymentMethod::LightningAddress(address) => InputType::LightningAddress(address),
        RawPaymentMethod::LiquidAddress(details) => {
            InputType::LiquidAddress(LiquidAddress { details, source })
        }
        RawPaymentMethod::LnurlPay(data) => InputType::LnurlPay(data),
        RawPaymentMethod::SilentPaymentAddress(details) => {
            InputType::SilentPaymentAddress(SilentPaymentAddress { details, source })
        }
    }
}
