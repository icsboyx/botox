use std::{collections::HashMap, sync::Arc};

use crate::*;

#[derive(Debug, Default)]
enum AudioType {
    None,
    #[default]
    Normal,
    Echo,
    Chorus,
}

type FunctionType = Box<dyn (Fn(&Message) -> String) + Send>;

struct BotCommand {
    activator: String,
    action: FunctionType,
    audio_type: AudioType,
}

impl BotCommand {
    fn new(activator: impl AsRef<str>, action: FunctionType) -> Self {
        Self {
            activator: activator.as_ref().to_string(),
            action,
            audio_type: AudioType::None,
        }
    }

    fn action(&self, irc_msg: &Message) -> String {
        (self.action)(irc_msg)
    }

    fn with_audio_type(mut self, audio_type: AudioType) -> Self {
        self.audio_type = audio_type;
        self
    }
}
struct BotCommands {
    hashmap: HashMap<String, BotCommand>,
}

impl BotCommands {
    fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
        }
    }

    fn add_command(&mut self, command: impl AsRef<str>, bot_command: BotCommand) {
        self.hashmap
            .insert(command.as_ref().to_string(), bot_command);
    }

    fn get_command(&self, command: &str) -> Option<&BotCommand> {
        self.hashmap.get(command)
    }
}

pub async fn start(bus: Arc<Bus>) {
    let my_subscriber = bus.register("commands").await;
    bus.subscribe("twitch", my_subscriber.clone()).await;
    let mut commands = BotCommands::new();
    let hello_command = BotCommand::new("!hello", Box::new(bot_cmd_hello));
    let echo_command = BotCommand::new("!echo", Box::new(bot_cmd_echo));
    commands.add_command("!hello", hello_command);
    commands.add_command("!echo", echo_command);

    loop {
        tokio::select! {
            messages = my_subscriber.queue_receive_all() => {
                for msg in messages {
                    let irc_msg = parse_message(&msg);
                    #[cfg(debug_assertions)]
                    println!("[COMMANDS][RX] {:?}", irc_msg);


                    match irc_msg.context.command.as_str() {
                        "PRIVMSG" => {
                            if irc_msg.payload.starts_with("!") {
                                let command = irc_msg.payload.split_whitespace().next().unwrap();
                                if let Some(response) = commands.get_command(command) {
                                    let response = response.action(&irc_msg);
                                    bus.get_entity("twitch").await.unwrap().queue_send(response).await;
                                }
                            }

                        }
                        "PING" => {
                            bus.get_entity("twitch").await.unwrap().queue_send("PONG :tmi.twitch.tv").await;
                        }
                        _ => {}
                    }


                }


            }
        }
    }
}

fn bot_cmd_hello(irc_msg: &Message) -> String {
    let destination = irc_msg.context.destination.clone();
    let _payload = irc_msg.payload.clone();
    let sender = irc_msg.context.sender.clone();
    format!("PRIVMSG {} :Hello to you @{}!", destination, sender)
}

fn bot_cmd_echo(irc_msg: &Message) -> String {
    let destination = irc_msg.context.destination.clone();
    let payload = irc_msg.payload.clone();
    let sender = irc_msg.context.sender.clone();
    format!("PRIVMSG {} :{} @{}!", destination, payload, sender)
}
