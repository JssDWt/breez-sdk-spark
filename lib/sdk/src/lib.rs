use std::sync::Arc;

pub enum ConnectRequest {
    Greenlight(breez_sdk_greenlight::ConnectRequest),
    Liquid(breez_sdk_liquid::ConnectRequest),
}

pub enum ConnectError {}

pub async fn connect(req: ConnectRequest) -> Result<Arc<BreezServices>, ConnectError> {
    match req {
        ConnectRequest::Greenlight(req) => breez_sdk_greenlight::connect(req).await,
        ConnectRequest::Liquid(req) => breez_sdk_liquid::connect(req).await,
    }
}
