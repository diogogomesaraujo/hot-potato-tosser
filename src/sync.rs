use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Flag(pub Arc<RwLock<bool>>);

impl Flag {
    pub fn new(value: bool) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    pub async fn read(&self) -> bool {
        *self.0.read().await
    }

    pub async fn write(&self, value: bool) {
        *self.0.write().await = value;
    }

    pub fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
