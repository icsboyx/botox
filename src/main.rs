mod config;
use async_std::{ sync::RwLock, task };
use config::*;
mod bus;
use bus::*;
use irc_parser::{ Context, IrcMessage };
use serde::{ Deserialize, Serialize };
use tokio::time::sleep;
mod defs;
mod irc_parser;
mod tts;
mod twitch;

use std::{ path::Path, process::exit, sync::Arc, time::Duration };
// use tokio::{ io::{ AsyncBufReadExt, BufReader }, time::{ sleep, Duration } };

#[tokio::main]
async fn main() {
    // let bus = Bus::new();

    let mut spawned = Vec::new();
    let config_file = Path::new("config.toml");
    let config: Config;
    if let Ok(c) = load_config(config_file).await {
        config = c;
    } else {
        eprintln!("Failed to load config file");
        exit(1);
    }
    let bus = Arc::new(Bus::new());

    spawned.push(tokio::spawn(twitch::start(bus.clone(), config.server, config.user)));
    spawned.push(tokio::spawn(tts::start(bus.clone())));

    for handle in spawned {
        handle.await.unwrap().unwrap();
    }
}
