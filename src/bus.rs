#![allow(dead_code)]
use crate::irc_parser::IrcMessage;
use anyhow::Result;
use std::{any::Any, collections::HashMap, fmt::Debug, sync::Arc};

use tokio::sync::{broadcast, mpsc, Notify, RwLock};

pub trait BusMessageTrait: Debug + Send + Sync + 'static {
    fn as_bus_message(self) -> BusMessageTypeEnum;
}
impl BusMessageTrait for String {
    fn as_bus_message(self) -> BusMessageTypeEnum {
        BusMessageTypeEnum::String(self)
    }
}

impl BusMessageTrait for IrcMessage {
    fn as_bus_message(self) -> BusMessageTypeEnum {
        BusMessageTypeEnum::IrcMessage(self)
    }
}

#[derive(Debug, Clone)]
pub enum BusMessageTypeEnum {
    String(String),
    IrcMessage(IrcMessage),
}

impl From<String> for BusMessageTypeEnum {
    fn from(s: String) -> Self {
        BusMessageTypeEnum::String(s)
    }
}

impl From<IrcMessage> for BusMessageTypeEnum {
    fn from(m: IrcMessage) -> Self {
        BusMessageTypeEnum::IrcMessage(m)
    }
}

// Implementing TryFrom<BusMessageTypeEnum> for IrcMessage
impl TryFrom<BusMessageTypeEnum> for IrcMessage {
    type Error = &'static str;

    fn try_from(value: BusMessageTypeEnum) -> Result<Self, Self::Error> {
        match value {
            BusMessageTypeEnum::IrcMessage(m) => Ok(m),
            _ => Err("BusMessageTypeEnum does not contain an IrcMessage"),
        }
    }
}
// Implementing TryFrom<BusMessageTypeEnum> for String
impl TryFrom<BusMessageTypeEnum> for String {
    type Error = &'static str;

    fn try_from(value: BusMessageTypeEnum) -> Result<Self, Self::Error> {
        match value {
            BusMessageTypeEnum::String(s) => Ok(s),
            _ => Err("BusMessageTypeEnum does not contain a String"),
        }
    }
}

#[derive(Debug)]
pub struct BusEntity {
    id: String,
    clients: broadcast::Sender<BusMessageTypeEnum>,
    producer_tx: mpsc::Sender<BusMessageTypeEnum>,
    producer_rx: Arc<RwLock<mpsc::Receiver<BusMessageTypeEnum>>>,
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

    pub async fn send_to_consumer(&self, message: BusMessageTypeEnum) -> Result<()> {
        // if let Ok(message) = serde_json::to_string(&message) {
        //     self.clients.send(message)?;
        //     return Ok(());
        self.clients.send(message)?;
        Ok(())
    }

    pub async fn recv_from_consumer(&self) -> Result<BusMessageTypeEnum> {
        let message = self.producer_rx.write().await.recv().await.unwrap();
        Ok(message)
    }

    // pub async fn recv_from_consumer_transform<MessageType: Serialize + for<'a> Deserialize<'a>>(
    //     &self
    // ) -> Result<MessageType> {
    //     let message = self.producer_rx.write().await.recv().await.unwrap();
    //     let message: MessageType = serde_json::from_str(&message)?;
    //     Ok(message)
    // }
}

#[derive(Debug, Clone)]
pub struct BusEntitySubscriber {
    rx: Arc<RwLock<broadcast::Receiver<BusMessageTypeEnum>>>,
    tx: Arc<mpsc::Sender<BusMessageTypeEnum>>,
}

impl BusEntitySubscriber {
    pub fn new(
        rx: broadcast::Receiver<BusMessageTypeEnum>,
        tx: mpsc::Sender<BusMessageTypeEnum>,
    ) -> Self {
        let rx = Arc::new(RwLock::new(rx));
        let tx = Arc::new(tx);
        BusEntitySubscriber { rx, tx }
    }

    pub async fn recv(&mut self) -> Result<BusMessageTypeEnum> {
        let message = self.rx.write().await.recv().await?;
        Ok(message)
    }

    // pub async fn recv_transform<MessageType: Serialize + for<'a> Deserialize<'a>>(
    //     &mut self
    // ) -> Result<MessageType> {
    //     let message = self.rx.write().await.recv().await?;
    //     let message: MessageType = serde_json::from_str(&message)?;
    //     Ok(message)
    // }

    pub async fn send(&self, message: BusMessageTypeEnum) -> Result<()> {
        // if let Ok(message) = serde_json::to_string(&message) {
        //     self.tx.send(message).await?;
        //     return Ok(());
        // }
        // anyhow::bail!("Failed to serialize message");
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
}
