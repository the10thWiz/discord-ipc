extern crate discord_ipc;
use std::{
    fs::File,
    io::{self, Read, Write},
    time::Duration,
};

use discord_ipc::{Client, Event, EventSubscribe, OauthScope, Result};
use simplelog::{Config, TermLogger};

const CLIENT_ID: u64 = 0;
const CLIENT_SECRET: &str = "";

fn read_token() -> io::Result<String> {
    let mut file = File::open("refresh_token")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    if !buf.is_empty() {
        Ok(buf)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "No token found"))
    }
}

fn write_token(s: &str) -> io::Result<()> {
    let mut file = File::create("refresh_token")?;
    file.write_all(s.as_bytes())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    TermLogger::init(log::LevelFilter::Trace, Config::default()).unwrap();
    let mut client = Client::new(CLIENT_ID)
        .secret(CLIENT_SECRET)
        .scope(OauthScope::RpcVoiceRead)
        .refresh_token(read_token().ok())
        .save_token(write_token)
        .connect()
        .await?;
    client.set_activity("Activity Name").await?;
    client.subscribe(EventSubscribe::VoiceChannelSelect).await?;
    loop {
        match client.event().await? {
            Event::VoiceChannelSelect(s) => {
                client
                    .subscribe(EventSubscribe::SpeakingStart {
                        channel_id: s.channel_id,
                    })
                    .await?;
                client
                    .subscribe(EventSubscribe::SpeakingStop {
                        channel_id: s.channel_id,
                    })
                    .await?;
            }
            _ => (),
        }
    }
}
