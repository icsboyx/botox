#![allow(dead_code)]
use crate::irc_parser::IrcMessage;
use anyhow::Result;

use bincode;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::sync::{broadcast, mpsc, Notify, RwLock};

#[derive(Debug, Clone)]
pub enum BusMessageTypeEnum {
    String(String),
    IrcMessage(IrcMessage),
}

#[derive(Debug)]
pub struct BusEntity {
    id: String,
    clients: broadcast::Sender<Vec<u8>>,
    producer_tx: mpsc::Sender<Vec<u8>>,
    producer_rx: Arc<RwLock<mpsc::Receiver<Vec<u8>>>>,
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

    pub async fn send(&self, message: impl serde::ser::Serialize) -> Result<()> {
        let message = bincode::serialize(&message)?;
        self.clients.send(message)?;
        Ok(())
    }

    pub async fn recv<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        if let Some(message) = self.producer_rx.write().await.recv().await {
            let message: T = bincode::deserialize(&message[..])?;
            return Ok(message);
        }
        anyhow::bail!("Failed to receive message")
    }
}

#[derive(Debug, Clone)]
pub struct BusEntitySubscriber {
    rx: Arc<RwLock<broadcast::Receiver<Vec<u8>>>>,
    tx: Arc<mpsc::Sender<Vec<u8>>>,
}

impl BusEntitySubscriber {
    pub fn new(rx: broadcast::Receiver<Vec<u8>>, tx: mpsc::Sender<Vec<u8>>) -> Self {
        let rx = Arc::new(RwLock::new(rx));
        let tx = Arc::new(tx);
        BusEntitySubscriber { rx, tx }
    }

    pub async fn recv<T: serde::de::DeserializeOwned>(&mut self) -> Result<T> {
        let message = self.rx.write().await.recv().await?;
        let message: T = bincode::deserialize(&message[..])?;
        Ok(message)
    }
    pub async fn send(&self, message: impl serde::ser::Serialize) -> Result<()> {
        let message = bincode::serialize(&message)?;
        self.tx.send(message).await?;
        Ok(())
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

    pub async fn remove_entity(&mut self, id: &str) {
        self.entities.write().await.remove(id);
        self.notify.notify_waiters();
    }

    pub async fn get_all_entities<'a>(&'a self) -> Vec<Arc<BusEntity>> {
        self.entities.read().await.values().cloned().collect()
    }
}
