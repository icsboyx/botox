#![allow(dead_code)]
use std::sync::Arc;

use tokio::sync::Mutex;

pub type ArcMutex<T> = Arc<Mutex<T>>;
pub trait AsArcMutex {
    fn as_arcmut(self) -> ArcMutex<Self>;
}

impl<T> AsArcMutex for T {
    fn as_arcmut(self) -> ArcMutex<Self> {
        Arc::new(Mutex::new(self))
    }
}
