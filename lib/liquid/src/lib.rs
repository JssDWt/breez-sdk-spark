use breez_sdk_core::{BreezServices, PaymentMethodType};
use breez_sdk_internal::{BreezServicesImpl, utils::Arc};

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
