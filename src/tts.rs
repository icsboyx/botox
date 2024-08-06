use crate::*;

use irc_parser::IrcMessage;
use msedge_tts::{
    tts::{
        client::{connect_async, MSEdgeTTSClientAsync},
        SpeechConfig,
    },
    voice::get_voices_list,
};
use tokio::sync::RwLock;

pub async fn start(bus: Arc<Bus>) {
    let context = "tts";
    let mut twitch_subscriber = bus.subscribe_to_entity("twitch").await.unwrap();
    let mut twitch_subscriber1 = bus.subscribe_to_entity("twitch").await.unwrap();

    let voices = get_voices_list().unwrap();
    let voice = voices
        .iter()
        .find(|v| v.name == "Microsoft Server Speech Text to Speech Voice (it-IT, GiuseppeNeural)")
        .unwrap();
    println!("Voice: {:?}", voice);
    let bot_voice_config = SpeechConfig::from(voice);
    println!("Bot voice config: {:?}", bot_voice_config);

    let stream = Arc::new(RwLock::new(connect_async().await.unwrap()));

    loop {
        tokio::select! {
            Ok(msg) = twitch_subscriber.recv::<IrcMessage>() => {
                println!("[{}] {:?}", context, msg);
                let msg = IrcMessage::try_from(msg).unwrap();
                if msg.context.command == "PRIVMSG" {
                    let text = msg.payload.clone();
                    create_audio( &bot_voice_config, text, stream.clone()).await;
                }
            }
            Ok(msg) = twitch_subscriber1.recv::<IrcMessage>() => {
                println!("[{}] {:?}", context, msg);
                let msg = IrcMessage::try_from(msg).unwrap();
                if msg.context.command == "PRIVMSG" {
                    let text = msg.payload.clone();
                    create_audio( &bot_voice_config, text, stream.clone()).await;
                }
            }

        }
    }
}

async fn create_audio(
    voice_config: &SpeechConfig,
    text: String,
    stream: Arc<RwLock<MSEdgeTTSClientAsync<async_std::net::TcpStream>>>,
) {
    let audio = stream
        .write()
        .await
        .synthesize(&text, voice_config)
        .await
        .unwrap();
    println!("Audio: {:?}", audio.audio_bytes.len());
}
