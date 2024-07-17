#![allow(dead_code)]
use std::{ collections::VecDeque, sync::Arc };

use tokio::sync::{ Notify, RwLock };

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

    pub async fn push_back(&self, item: String) {
        self.queue.write().await.push_back(item);
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

    pub async fn send(&self, message: String) {
        for subscriber in self.subscribers.write().await.iter() {
            subscriber.push_back(message.clone()).await;
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
    pub fn new(id: String) -> Self {
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

    pub fn subscribers(&self) -> Arc<Subscribers> {
        self.subscribers.clone()
    }
    pub async fn send(&self, message: String) {
        self.subscribers.send(message).await;
    }
    pub async fn subscribe(&self, subscriber: Arc<Queue>) {
        self.subscribers.push(subscriber).await;
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
        let id = id.as_ref().to_owned();
        let entity = Arc::new(BusEntity::new(id));
        self.entities.write().await.push(entity.clone());
        self.event.notify_waiters();
        entity
    }

    pub async fn get_entity(&self, id: impl AsRef<str>) -> Option<Arc<BusEntity>> {
        let id = id.as_ref();
        self.entities
            .read().await
            .iter()
            .find(|entity| entity.id == id)
            .cloned()
    }

    pub async fn subscribe(&self, id: impl AsRef<str>, subscriber: Arc<BusEntity>) {
        loop {
            if let Some(entity) = self.get_entity(&id).await {
                println!("[{}] Subscribing to {}", subscriber.id, entity.id());
                entity.subscribe(subscriber.queue()).await;
                break;
            } else {
                self.event.notified().await;
            }
        }
    }
}
