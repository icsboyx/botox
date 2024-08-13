use crate::*;
use anyhow::{ Context, Result };
use defs::AsArcMutex;
use futures::{ future::ok, pin_mut };
use irc_parser::IrcMessage;
use msedge_tts::{
    tts::{ client::{ connect, connect_async }, SpeechConfig },
    voice::get_voices_list,
};
use rodio::{ Decoder, OutputStream, Sink, Source };
use std::{ io::BufReader, pin::pin };

pub async fn start(bus: Arc<Bus>) -> Result<()> {
    let context = "tts";
    let mut twitch_subscriber = bus.subscribe_to_entity("twitch").await.unwrap();
    let voices = get_voices_list().unwrap();
    let voice = voices
        .iter()
        .find(|v| v.name == "Microsoft Server Speech Text to Speech Voice (it-IT, GiuseppeNeural)")
        .unwrap();

    loop {
        tokio::select! {
            Ok(msg) = twitch_subscriber.recv::<IrcMessage>() => {
                let bot_voice_config = SpeechConfig::from(voice);
                let text = msg.payload;
                if msg.context.command == "PRIVMSG" {
                    println!("[{}][TX] {}", context, text);
                    tokio::task::spawn_blocking(move || create_audio(bot_voice_config, text)).await??;
                }

            }
        }
    }

    // loop {
    //     let bot_voice_config = SpeechConfig::from(voice);
    //     let msg = twitch_subscriber.recv::<IrcMessage>().await.unwrap();
    //     if msg.context.command == "PRIVMSG" {
    //         println!("Bot voice config: {:?}", bot_voice_config);
    //         let text = msg.payload;
    //         println!("[{}][TX] {}", context, text);
    //         tokio::spawn(create_audio(bot_voice_config, text));
    //     }
    // }
}

pub fn create_audio(voice_config: SpeechConfig, text: String) -> Result<()> {
    let (_audio_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut stream = connect().unwrap();
    let audio_payload = stream.synthesize(&text, &voice_config).unwrap();
    use std::io::Cursor;
    let audio_buffer = BufReader::new(Cursor::new(audio_payload.audio_bytes));
    // let source = Decoder::new(audio_buffer).unwrap();
    let sink = stream_handle.play_once(audio_buffer)?;
    sink.sleep_until_end();
    // let sink = Sink::try_new(&stream_handle).unwrap();
    // sink.append(source);
    // sink.sleep_until_end();
    Ok(())
}
