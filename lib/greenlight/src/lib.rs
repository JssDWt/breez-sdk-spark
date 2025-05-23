use breez_sdk_core::{BreezServices, PaymentMethodType};
use breez_sdk_internal::{BreezServicesImpl, utils::Arc};

pub struct ConnectRequest {}

pub enum ConnectError {}

pub async fn connect(req: ConnectRequest) -> Result<Arc<BreezServices<Greenlight>>, ConnectError> {
    todo!()
}

pub struct Greenlight {}

impl BreezServicesImpl for Greenlight {
    fn get_payment_methods(&self) -> Vec<PaymentMethodType> {
        todo!()
    }
}
