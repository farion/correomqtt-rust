use std::{
    fmt,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

pub trait ScriptCancellationHandle: Send + Sync {
    fn cancel(&self);
}

#[derive(Clone, Default)]
pub struct ScriptCancellationToken {
    inner: Arc<CancellationInner>,
}

#[derive(Default)]
struct CancellationInner {
    cancelled: AtomicBool,
    handles: Mutex<Vec<Arc<dyn ScriptCancellationHandle>>>,
}

impl ScriptCancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }

    pub fn cancel(&self) {
        self.inner.cancelled.store(true, Ordering::SeqCst);
        self.cancel_owned_operations();
    }

    pub fn register_handle(&self, handle: Arc<dyn ScriptCancellationHandle>) {
        if self.is_cancelled() {
            handle.cancel();
        }

        self.inner
            .handles
            .lock()
            .expect("script cancellation handles lock poisoned")
            .push(handle);
    }

    pub fn cancel_owned_operations(&self) {
        let handles = self
            .inner
            .handles
            .lock()
            .expect("script cancellation handles lock poisoned")
            .clone();

        for handle in handles {
            handle.cancel();
        }
    }
}

impl fmt::Debug for ScriptCancellationToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScriptCancellationToken")
            .field("cancelled", &self.is_cancelled())
            .finish_non_exhaustive()
    }
}
