#![allow(dead_code)]
use std::{ collections::VecDeque, sync::Arc };
use tokio::sync::{ Notify, RwLock };
use anyhow::Result;

#[derive(Debug)]
pub struct Queue {
    event: Arc<Notify>,
    queue: Arc<RwLock<VecDeque<String>>>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            event: Arc::new(Notify::new()),
            queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub async fn push_back(&self, item: impl AsRef<str>) {
        self.queue.write().await.push_back(item.as_ref().to_string());
        self.event.notify_waiters();
    }

    pub fn event(&self) -> Arc<Notify> {
        self.event.clone()
    }

    pub async fn pop_front(&self) -> Option<String> {
        self.queue.write().await.pop_front()
    }

    pub async fn display(&self) -> Vec<String> {
        self.queue.read().await.iter().cloned().collect()
    }

    pub async fn receive(&self) -> Option<String> {
        self.event().notified().await;
        self.pop_front().await
    }

    pub async fn receive_all(&self) -> Vec<String> {
        self.event().notified().await;
        self.queue
            .write().await
            .drain(..)
            .collect()
    }
}
pub struct Subscribers {
    subscribers: Arc<RwLock<Vec<Arc<Queue>>>>,
}

impl Subscribers {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn push(&self, subscriber: Arc<Queue>) {
        self.subscribers.write().await.push(subscriber);
    }

    pub async fn send(&self, message: impl AsRef<str> + Clone) {
        for subscriber in self.subscribers.write().await.iter() {
            subscriber.push_back(message.as_ref()).await;
        }
    }

    pub async fn subscribe(&self, subscriber: Arc<Queue>) {
        self.subscribers.write().await.push(subscriber);
    }
}

pub struct BusEntity {
    id: String,
    queue: Arc<Queue>,
    subscribers: Arc<Subscribers>,
}

impl BusEntity {
    pub fn new(id: impl AsRef<str>) -> Self {
        let id = id.as_ref().to_string();
        Self {
            id,
            queue: Arc::new(Queue::new()),
            subscribers: Arc::new(Subscribers::new()),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub async fn subscribe(&self, subscriber: Arc<Queue>) {
        self.subscribers.push(subscriber).await;
    }

    pub fn subscribers(&self) -> Arc<Subscribers> {
        self.subscribers.clone()
    }

    pub async fn subscribers_send(&self, message: impl AsRef<str>) {
        self.subscribers.send(message.as_ref()).await;
    }

    pub async fn queue_send(&self, message: impl AsRef<str>) {
        self.queue.push_back(message.as_ref()).await;
    }

    pub async fn queue_receive(&self) -> Option<String> {
        self.queue.receive().await
    }

    pub async fn queue_receive_all(&self) -> Vec<String> {
        self.queue.receive_all().await
    }
}

pub struct Bus {
    entities: Arc<RwLock<Vec<Arc<BusEntity>>>>,
    event: Arc<Notify>,
}

impl Bus {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            entities: Arc::new(RwLock::new(Vec::new())),
            event: Arc::new(Notify::new()),
        })
    }

    pub async fn register(&self, id: impl AsRef<str>) -> Arc<BusEntity> {
        let entity = Arc::new(BusEntity::new(id));
        self.entities.write().await.push(entity.clone());
        self.event.notify_waiters();
        entity
    }

    pub async fn get_entity(&self, id: impl AsRef<str>) -> Result<Arc<BusEntity>> {
        let id = id.as_ref();
        self.entities
            .read().await
            .iter()
            .find(|entity| entity.id() == id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Entity not found"))
    }

    pub async fn subscribe(&self, id: impl AsRef<str>, subscriber: Arc<BusEntity>) {
        let id = id.as_ref();
        loop {
            if let Ok(entity) = self.get_entity(&id).await {
                entity.subscribe(subscriber.queue()).await;
                break;
            } else {
                self.event.notified().await;
            }
        }
    }
}
