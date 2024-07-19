mod config;
use config::*;

mod bus;
use bus::*;
use tokio_tungstenite::tungstenite::http::uri::PathAndQuery;

mod defs;

mod twitch;

use std::{ path::Path, process::exit, sync::Arc };
use tokio::{ io::AsyncBufReadExt, time::{ sleep, Duration } };

#[tokio::main]
async fn main() {
    let bus = Bus::new();
    let config_file = Path::new("config.toml");
    let config: Config;
    if let Ok(c) = load_config(config_file).await {
        config = c;
    } else {
        eprintln!("Failed to load config file");
        exit(1);
    }
    tokio::spawn(twitch::start(bus.clone(), config.server, config.user));
    tokio::spawn(task_user_input(bus.clone()));

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}

async fn task_user_input(bus: Arc<Bus>) {
    // read user input
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let my_subscriber = bus.register("user").await;
    bus.subscribe("twitch", my_subscriber.clone()).await;
    let twitch_queue = bus.get_entity("twitch").await.unwrap();

    let my_subscriber_arc = my_subscriber.clone();

    let read_user_input = async move {
        loop {
            let mut line = String::new();
            stdin.read_line(&mut line).await.unwrap();
            my_subscriber.as_ref().queue_send(&line).await;
            twitch_queue.queue_send(&line).await;
        }
    };

    tokio::select! {
        _ = read_user_input => {}
    }
}
