use breez_sdk_core::{error::*, model::*};
use breez_sdk_input::PaymentRequest;
use breez_sdk_internal::utils::Arc;
use breez_sdk_macros::async_trait;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Millisatoshi(u64);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FeeBreakdown {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaymentDetails {}

pub struct ConnectRequest {}

pub enum ConnectError {}

pub async fn connect(req: ConnectRequest) -> Result<Arc<Greenlight>, ConnectError> {
    todo!()
}

pub struct Greenlight {}

#[async_trait]
impl BreezServices<Millisatoshi, PaymentDetails, FeeBreakdown> for Greenlight {
    async fn parse_and_pick(&self, input: &str) -> Result<SourcedInputType<Millisatoshi>, ParseAndPickError> {
        todo!()
    }
    async fn pick_payment_method(
        &self,
        payment_request: PaymentRequest,
    ) -> Result<SourcedPaymentMethod<Millisatoshi>, PickPaymentMethodError> {
        todo!()
    }
    async fn prepare_send_bitcoin(
        &self,
        req: PrepareSendBitcoinRequest,
    ) -> Result<PrepareSendBitcoinResponse<Millisatoshi, FeeBreakdown>, PrepareSendBitcoinError> {
        todo!()
    }
    async fn prepare_send_lightning(
        &self,
        req: PrepareSendLightningRequest<Millisatoshi>,
    ) -> Result<PrepareSendLightningResponse<Millisatoshi, FeeBreakdown>, PrepareSendLightningError> {
        todo!()
    }
    async fn prepare_send_lnurl_pay(
        &self,
        req: PrepareSendLnurlPayRequest<Millisatoshi>,
    ) -> Result<PrepareSendLnurlPayResponse<Millisatoshi, FeeBreakdown>, PrepareSendLnurlPayError> {
        todo!()
    }
    async fn prepare_send_liquid_address(
        &self,
        req: PrepareSendLiquidAddressRequest<Millisatoshi>,
    ) -> Result<PrepareSendLiquidAddressResponse<Millisatoshi, FeeBreakdown>, PrepareSendLiquidAddressError> {
        todo!()
    }
    async fn prepare_receive_payment(
        &self,
        req: PrepareReceivePaymentRequest<Millisatoshi>,
    ) -> Result<PrepareReceivePaymentResponse<Millisatoshi>, PrepareReceivePaymentError> {
        todo!()
    }
    async fn receive_payment(
        &self,
        req: ReceivePaymentRequest<Millisatoshi>,
    ) -> Result<ReceivePaymentResponse, ReceivePaymentError> {
        todo!()
    }
    async fn send_bitcoin(
        &self,
        req: SendBitcoinRequest<Millisatoshi, FeeBreakdown>,
    ) -> Result<SendBitcoinResponse<Millisatoshi, PaymentDetails, FeeBreakdown>, SendBitcoinError> {
        todo!()
    }
    async fn send_lightning(
        &self,
        req: SendLightningRequest<Millisatoshi, FeeBreakdown>,
    ) -> Result<SendLightningResponse<Millisatoshi, PaymentDetails, FeeBreakdown>, SendLightningError> {
        todo!()
    }
    async fn send_lnurl_pay(
        &self,
        req: SendLnurlPayRequest<Millisatoshi, FeeBreakdown>,
    ) -> Result<SendLnurlPayResponse<Millisatoshi, PaymentDetails, FeeBreakdown>, SendLnurlPayError> {
        todo!()
    }
    async fn send_liquid_address(
        &self,
        req: SendLiquidAddressRequest<Millisatoshi, FeeBreakdown>,
    ) -> Result<SendLiquidAddressResponse<Millisatoshi, PaymentDetails, FeeBreakdown>, SendLiquidAddressError> {
        todo!()
    }
}
