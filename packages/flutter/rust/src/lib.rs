mod frb_generated;

use extend::ext;

use breez_sdk_spark::{AddEventListenerResponse, BreezSdk, SdkEvent, SdkEventListener};
use frb_generated::StreamSink;

pub struct BindingEventListener {
    pub listener: StreamSink<SdkEvent>,
}

impl SdkEventListener for BindingEventListener {
    fn on_event(&self, e: SdkEvent) {
        let _ = self.listener.add(e);
    }
}

#[ext]
#[breez_sdk_macros::async_trait]
pub impl BreezSdk {
    async fn frb_override_add_event_listener(&self, listener: StreamSink<SdkEvent>) -> AddEventListenerResponse {
        self.add_event_listener(Box::new(BindingEventListener { listener })).await
    }
}

