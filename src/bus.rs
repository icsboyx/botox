#![allow(dead_code)]
use std::{clone, collections::HashMap, sync::Arc};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, Notify, RwLock};

#[derive(Debug, Clone)]
pub struct BusEntitySubscriber {
    rx: Arc<RwLock<broadcast::Receiver<String>>>,
    tx: Arc<mpsc::Sender<String>>,
}

impl BusEntitySubscriber {
    pub fn new(rx: broadcast::Receiver<String>, tx: mpsc::Sender<String>) -> Self {
        let rx = Arc::new(RwLock::new(rx));
        let tx = Arc::new(tx);
        BusEntitySubscriber { rx, tx }
    }

    pub async fn recv(&mut self) -> Result<String> {
        let message = self.rx.write().await.recv().await?;
        Ok(message)
    }

    pub async fn recv_transform<MessageType: Serialize + for<'a> Deserialize<'a>>(
        &mut self,
    ) -> Result<MessageType> {
        let message = self.rx.write().await.recv().await?;
        let message: MessageType = serde_json::from_str(&message)?;
        Ok(message)
    }

    pub async fn send(&self, message: impl Serialize + for<'a> Deserialize<'_>) -> Result<()> {
        if let Ok(message) = serde_json::to_string(&message) {
            self.tx.send(message).await?;
            return Ok(());
        }
        anyhow::bail!("Failed to serialize message");
    }
}

#[derive(Debug)]
pub struct BusEntity {
    id: String,
    clients: broadcast::Sender<String>,
    producer_tx: mpsc::Sender<String>,
    producer_rx: Arc<RwLock<mpsc::Receiver<String>>>,
}

impl BusEntity {
    pub fn new(id: impl AsRef<str>) -> Self {
        let (receiver_tx, _receiver_rx) = tokio::sync::broadcast::channel(1024);
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        BusEntity {
            id: id.as_ref().to_string(),
            clients: receiver_tx,
            producer_tx: tx,
            producer_rx: Arc::new(RwLock::new(rx)),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn send_to_consumer(
        &self,
        message: impl Serialize + for<'a> Deserialize<'_>,
    ) -> Result<()> {
        if let Ok(message) = serde_json::to_string(&message) {
            self.clients.send(message)?;
            return Ok(());
        }
        anyhow::bail!("Failed to serialize message");
    }

    pub async fn recv_from_producer(&self) -> Result<String> {
        let message = self.clients.subscribe().recv().await?;
        Ok(message)
    }

    pub async fn recv_from_producer_transform<MessageType: Serialize + for<'a> Deserialize<'a>>(
        &self,
    ) -> Result<MessageType> {
        let message = self.clients.subscribe().recv().await?;
        let message: MessageType = serde_json::from_str(&message)?;
        Ok(message)
    }

    pub async fn send_to_producer(
        &self,
        message: impl Serialize + for<'a> Deserialize<'_>,
    ) -> Result<()> {
        if let Ok(message) = serde_json::to_string(&message) {
            self.producer_tx.send(message).await?;
            return Ok(());
        }
        anyhow::bail!("Failed to serialize message");
    }

    pub async fn recv_from_consumer(&self) -> Result<String> {
        let message = self.producer_rx.write().await.recv().await.unwrap();
        Ok(message)
    }

    pub async fn recv_from_consumer_transform<MessageType: Serialize + for<'a> Deserialize<'a>>(
        &self,
    ) -> Result<MessageType> {
        let message = self.producer_rx.write().await.recv().await.unwrap();
        let message: MessageType = serde_json::from_str(&message)?;
        Ok(message)
    }
}

pub struct Bus {
    entities: Arc<RwLock<HashMap<String, Arc<BusEntity>>>>,
    notify: Notify,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            entities: Arc::new(RwLock::new(HashMap::new())),
            notify: Notify::new(),
        }
    }

    pub async fn add_new_entity(&self, id: impl AsRef<str>) -> Arc<BusEntity> {
        let entity = BusEntity::new(&id);
        let entity = Arc::new(entity);
        self.entities
            .write()
            .await
            .insert(id.as_ref().to_string(), entity.clone());
        self.notify.notify_waiters();
        entity
    }

    pub async fn add_entity(&mut self, entity: BusEntity) {
        self.entities
            .write()
            .await
            .insert(entity.id().to_string(), Arc::new(entity));
        self.notify.notify_waiters();
    }

    pub async fn get_entity(&self, id: &str) -> Option<Arc<BusEntity>> {
        self.entities.read().await.get(id).cloned()
    }

    pub async fn await_entity(&self, id: &str) -> Option<Arc<BusEntity>> {
        loop {
            if let Some(entity) = self.get_entity(id).await {
                return Some(entity);
            }
            self.notify.notified().await;
        }
    }

    pub async fn subscribe_to_entity(&self, id: &str) -> Option<BusEntitySubscriber> {
        if let Some(entity) = self.await_entity(id).await {
            let subscriber = {
                BusEntitySubscriber::new(entity.clients.subscribe(), entity.producer_tx.clone())
            };
            return Some(subscriber);
        }
        None
    }
}
