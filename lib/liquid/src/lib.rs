use std::sync::Arc;

use breez_sdk_core::{BreezServices, BreezServicesImpl, PaymentMethodType};

pub struct ConnectRequest {}

pub enum ConnectError {}

pub async fn connect(req: ConnectRequest) -> Result<Arc<BreezServices<Liquid>>, ConnectError> {
    todo!()
}

pub struct Liquid {}

impl BreezServicesImpl for Liquid {
    fn get_payment_methods(&self) -> Vec<PaymentMethodType> {
        todo!()
    }
}
