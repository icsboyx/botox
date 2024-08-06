mod config;
use chrono::{format::DelayedFormat, NaiveWeek};
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
use futures::pin_mut;

use std::{path::Path, pin::Pin, process::exit, sync::Arc, time::Duration};
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
    tokio::spawn(timestamp_generator(bus.clone(), 5));
    tokio::spawn(monitor_task(bus.clone()));
    // tokio::spawn(tts::start(bus.clone()));

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}

pub async fn monitor_task(bus: Arc<Bus>) {
    let mut twitch_subscriber = bus.subscribe_to_entity("twitch").await.unwrap();
    let mut timestamp_subscriber = bus
        .subscribe_to_entity("timestamp_generator")
        .await
        .unwrap();

    let twitch_subscriber_task = async move {
        loop {
            let msg = twitch_subscriber.recv::<IrcMessage>().await.unwrap();
            println!("[monitor_task][twitch] {:?}", msg);
        }
    };

    let timestamp_subscriber_task = async move {
        loop {
            let msg = timestamp_subscriber.recv::<String>().await.unwrap();
            println!("[monitor_task][timestamp_generator] {:?}", msg);
        }
    };

    tokio::select! {
        _ = twitch_subscriber_task => {}
        _ = timestamp_subscriber_task => {}
    }
}

pub async fn timestamp_generator(bus: Arc<bus::Bus>, delay: u64) {
    let my_subscriber = bus.add_new_entity("timestamp_generator").await;
    println!("[timestamp_generator] Started");
    loop {
        sleep(Duration::from_secs(delay)).await;
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
        println!("[timestamp_generator][Sending] {:?}", timestamp.clone());
        my_subscriber.send(timestamp.clone()).await.unwrap();
    }
}
