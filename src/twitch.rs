use std::sync::Arc;

use crate::*;
use anyhow::Result;
use futures::{ pin_mut, SinkExt, StreamExt };
use tokio_tungstenite::tungstenite::Message;

trait WsMessageHandler {
    fn as_ws_text(&self) -> Message;
}

impl<T: std::fmt::Display> WsMessageHandler for T {
    fn as_ws_text(&self) -> Message {
        println!("[TWITCH][TX] {}", self);
        Message::text(self.to_string())
    }
}

pub async fn start(
    bus: Arc<Bus>,
    server_config: ServerConfig,
    user_config: UserConfig
) -> Result<()> {
    let my_subscriber = bus.register("twitch").await;

    let (ws_stream, _response) = tokio_tungstenite::connect_async(server_config.address).await?;
    let (mut write, mut read) = ws_stream.split();

    _ = write.send(format!("PASS oauth:{}", user_config.token).as_ws_text()).await?;
    _ = write.send(format!("NICK {}", user_config.nick).as_ws_text()).await?;
    _ = write.send(format!("JOIN #{}", user_config.channel).as_ws_text()).await?;

    let ping_interval = tokio::time::interval(Duration::from_secs(180));
    let _my_queue = my_subscriber.queue().event();

    pin_mut!(ping_interval);

    loop {
        tokio::select! {
                _ = ping_interval.tick() => {
                    write.send("PING  :tmi.twitch.tv".as_ws_text()).await?;
                }

            Some(line) = read.next() => {
                if let Ok(line) = line {
                    let lines = line.to_text().unwrap().trim_end_matches("\r\n").split("\r\n");
                    for line in lines {
                        let payload = line;
                        my_subscriber.subscribers_send(payload.to_string()).await;
                        println!("[TWITCH][RX] {}", payload);

                    }
                }
            }

            Some(msg) = my_subscriber.queue_receive() => {
                write.send(msg.as_ws_text()).await?;
            }
        }
    }
}
