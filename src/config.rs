use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[derive(Debug, Deserialize, Serialize)]
pub struct UserConfig {
    pub nick: String,
    pub token: String,
    pub channel: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub address: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub user: UserConfig,
    pub server: ServerConfig,
}

pub async fn load_config(file: &Path) -> Result<Config> {
    let mut config_file = tokio::fs::File::open(file).await?;
    let mut config_strings = String::new();
    config_file.read_to_string(&mut config_strings).await?;
    let config = toml::from_str(&config_strings)?;
    println!("{:?}", config);
    Ok(config)
}
