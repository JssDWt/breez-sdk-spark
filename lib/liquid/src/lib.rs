use breez_sdk_core::{error::*, model::*};
use breez_sdk_input::PaymentRequest;
use breez_sdk_internal::utils::Arc;
use breez_sdk_macros::async_trait;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Amount {
    Bitcoin(u64),

    Asset {
        asset_id: String,
        amount: u64,
    },
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FeeBreakdown {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaymentDetails {}

pub struct ConnectRequest {}

pub enum ConnectError {}

pub async fn connect(req: ConnectRequest) -> Result<Arc<Liquid>, ConnectError> {
    todo!()
}

pub struct Liquid {}

#[async_trait]
impl BreezServices<Amount, PaymentDetails, FeeBreakdown> for Liquid {
    async fn parse_and_pick(&self, input: &str) -> Result<SourcedInputType<Amount>, ParseAndPickError> {
        todo!()
    }
    async fn pick_payment_method(
        &self,
        payment_request: PaymentRequest,
    ) -> Result<SourcedPaymentMethod<Amount>, PickPaymentMethodError> {
        todo!()
    }
    async fn prepare_send_bitcoin(
        &self,
        req: PrepareSendBitcoinRequest,
    ) -> Result<PrepareSendBitcoinResponse<Amount, FeeBreakdown>, PrepareSendBitcoinError> {
        todo!()
    }
    async fn prepare_send_lightning(
        &self,
        req: PrepareSendLightningRequest<Amount>,
    ) -> Result<PrepareSendLightningResponse<Amount, FeeBreakdown>, PrepareSendLightningError> {
        todo!()
    }
    async fn prepare_send_lnurl_pay(
        &self,
        req: PrepareSendLnurlPayRequest<Amount>,
    ) -> Result<PrepareSendLnurlPayResponse<Amount, FeeBreakdown>, PrepareSendLnurlPayError> {
        todo!()
    }
    async fn prepare_send_liquid_address(
        &self,
        req: PrepareSendLiquidAddressRequest<Amount>,
    ) -> Result<PrepareSendLiquidAddressResponse<Amount, FeeBreakdown>, PrepareSendLiquidAddressError> {
        todo!()
    }
    async fn prepare_receive_payment(
        &self,
        req: PrepareReceivePaymentRequest<Amount>,
    ) -> Result<PrepareReceivePaymentResponse<Amount>, PrepareReceivePaymentError> {
        todo!()
    }
    async fn receive_payment(
        &self,
        req: ReceivePaymentRequest<Amount>,
    ) -> Result<ReceivePaymentResponse, ReceivePaymentError> {
        todo!()
    }
    async fn send_bitcoin(
        &self,
        req: SendBitcoinRequest<Amount, FeeBreakdown>,
    ) -> Result<SendBitcoinResponse<Amount, PaymentDetails, FeeBreakdown>, SendBitcoinError> {
        todo!()
    }
    async fn send_lightning(
        &self,
        req: SendLightningRequest<Amount, FeeBreakdown>,
    ) -> Result<SendLightningResponse<Amount, PaymentDetails, FeeBreakdown>, SendLightningError> {
        todo!()
    }
    async fn send_lnurl_pay(
        &self,
        req: SendLnurlPayRequest<Amount, FeeBreakdown>,
    ) -> Result<SendLnurlPayResponse<Amount, PaymentDetails, FeeBreakdown>, SendLnurlPayError> {
        todo!()
    }
    async fn send_liquid_address(
        &self,
        req: SendLiquidAddressRequest<Amount, FeeBreakdown>,
    ) -> Result<SendLiquidAddressResponse<Amount, PaymentDetails, FeeBreakdown>, SendLiquidAddressError> {
        todo!()
    }
}
