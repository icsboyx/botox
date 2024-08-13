use std::sync::Arc;

use crate::*;
use anyhow::Result;
use futures::{ pin_mut, SinkExt, StreamExt };
use irc_parser::IrcMessage;
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
    let my_subscriber = bus.add_new_entity("twitch").await;

    let (ws_stream, _response) = tokio_tungstenite::connect_async(server_config.address).await?;
    let (mut write, mut read) = ws_stream.split();

    _ = write.send(format!("PASS oauth:{}", user_config.token).as_ws_text()).await?;
    _ = write.send(format!("NICK {}", user_config.nick).as_ws_text()).await?;
    _ = write.send(format!("JOIN #{}", user_config.channel).as_ws_text()).await?;
    _ = write.send("CAP REQ :twitch.tv/tags".as_ws_text()).await?;

    let ping_interval = tokio::time::interval(Duration::from_secs(180));

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
                        let irc_message = irc_parser::parse_message(&payload.to_string());
                        println!("[TWITCH][RX] {:?}", irc_message);
                        my_subscriber.send(irc_message).await?;
                    }
                }
            }

            Ok(msg) = my_subscriber.recv::<IrcMessage>()=> {
                println!("[TWITCH][TX] {:?}", msg);
            }
        }
    }
}
