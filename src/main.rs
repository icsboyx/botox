mod config;
use config::*;

mod bus;
use bus::*;
use serde::{Deserialize, Serialize};

mod irc_parser;
use irc_parser::*;

// mod commands;
// use commands::*;

mod defs;

mod twitch;

use std::{collections::HashMap, path::Path, process::exit, sync::Arc};
use tokio::{
    io::{unix::AsyncFd, AsyncBufReadExt, BufReader},
    time::{sleep, Duration},
};

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

    tokio::spawn(test_sub(bus.clone()));
    tokio::spawn(twitch::start(bus.clone(), config.server, config.user));
    tokio::spawn(user_input(bus.clone()));
    // tokio::spawn(commands::start(bus.clone()));

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}

async fn test_sub(bus: Arc<Bus>) {
    println!("waiting for twitch entity");
    let mut twitch_sunscriber = bus.subscribe_to_entity("twitch").await.unwrap();
    println!("got twitch entity");
    loop {
        let msg = twitch_sunscriber
            .recv_transform::<IrcMessage>()
            .await
            .unwrap();
        println!("[test_sub][row] {:?}", msg);
    }
}

async fn user_input(bus: Arc<Bus>) {
    let mut task_handlers = Vec::new();

    let twitch_sunscriber = bus.subscribe_to_entity("twitch").await.unwrap();
    let mut clone_t = twitch_sunscriber.clone();

    let read_user_input = async move {
        let mut std_out = BufReader::new(tokio::io::stdin());
        let mut line = String::new();
        std_out.read_line(&mut line).await.unwrap();
        println!("[user_input] {:?}", line);
        let irc_message = IrcMessage::new(
            HashMap::new(),
            Context::new("test", "PRIVMSG", "#icsboyx"),
            line,
        );
        twitch_sunscriber.send(irc_message).await.unwrap();
    };

    let read_twitch = async move {
        loop {
            let msg = clone_t.recv_transform::<IrcMessage>().await.unwrap();
            println!("[user_input][row] {:?}", msg);
        }
    };

    task_handlers.push(tokio::spawn(read_user_input));
    task_handlers.push(tokio::spawn(read_twitch));

    for task in task_handlers {
        task.await.unwrap();
    }
}
