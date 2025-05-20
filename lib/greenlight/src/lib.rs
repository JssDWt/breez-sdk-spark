use std::sync::Arc;

use breez_sdk_core::BreezServices;

pub struct ConnectRequest {}

pub enum ConnectError {}

pub async fn connect(req: ConnectRequest) -> Result<Arc<BreezServices>, ConnectError> {
    todo!()
}
