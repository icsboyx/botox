mod config;
use config::*;

mod bus;
use bus::*;
use irc_parser::IrcMessage;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
mod defs;
mod irc_parser;
mod test;
mod tts;
mod twitch;

use std::{path::Path, process::exit, sync::Arc, time::Duration};
// use tokio::{ io::{ AsyncBufReadExt, BufReader }, time::{ sleep, Duration } };

#[tokio::main]
async fn main() {
    // let bus = Bus::new();
    let config_file = Path::new("config.toml");
    let config: Config;
    if let Ok(c) = load_config(config_file).await {
        config = c;
    } else {
        eprintln!("Failed to load config file");
        exit(1);
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Test {
        a: String,
        b: i32,
    }
    let bus = Arc::new(Bus::new());

    tokio::spawn(twitch::start(bus.clone(), config.server, config.user));
    tokio::spawn(monitor_task(bus.clone()));
    tokio::spawn(tts::start(bus.clone()));

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}

pub async fn monitor_task(bus: Arc<Bus>) {
    let context = "monitor_task";
    let mut twitch_subscriber = bus.subscribe_to_entity("twitch").await.unwrap();
    println!("[{}] got twitch entity", context);

    loop {
        let msg = twitch_subscriber.recv::<IrcMessage>().await.unwrap();
        {
            println!("[{}] {:?}", context, msg);
        }
    }
}
