use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::{RwLock, broadcast};
use tracing::{debug, info, trace};
use uuid::Uuid;

use crate::model::{SdkEvent, SdkEventListener};

pub struct EventManager {
    listeners: RwLock<HashMap<String, Box<dyn SdkEventListener>>>,
    notifier: broadcast::Sender<SdkEvent>,
    is_paused: AtomicBool,
}

impl EventManager {
    pub fn new() -> Self {
        let (notifier, _) = broadcast::channel::<SdkEvent>(100);

        Self {
            listeners: Default::default(),
            notifier,
            is_paused: AtomicBool::new(false),
        }
    }

    pub async fn add(&self, listener: Box<dyn SdkEventListener>) -> String {
        let id = Uuid::new_v4().to_string();
        (*self.listeners.write().await).insert(id.clone(), listener);
        debug!("Added event listener with id: {id}");
        id
    }

    pub async fn remove(&self, id: String) {
        debug!("Removing event listener with id: {id}");
        (*self.listeners.write().await).remove(&id);
    }

    pub async fn notify(&self, e: SdkEvent) {
        if self.is_paused.load(Ordering::SeqCst) {
            debug!("Event notifications are paused, not emitting event: {e:?}");
            return;
        }

        debug!("Emitting event: {e:?}");
        let _ = self.notifier.send(e.clone());

        for (id, listener) in (*self.listeners.read().await).iter() {
            trace!("Emitting event to listener: {id}");
            listener.on_event(e.clone());
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SdkEvent> {
        self.notifier.subscribe()
    }

    pub fn pause_notifications(&self) {
        info!("Pausing event notifications");
        self.is_paused.store(true, Ordering::SeqCst);
    }

    pub fn resume_notifications(&self) {
        info!("Resuming event notifications");
        self.is_paused.store(false, Ordering::SeqCst);
    }
}
