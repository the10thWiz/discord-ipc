extern crate discord_ipc;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

use discord_ipc::{
    command::VoiceChannelSelect, Client, Event, EventSubscribe, FileSaver, OauthScope, Result,
};
use simplelog::{Config, TermLogger};

// const CLIENT_ID: u64 = 0;
// const CLIENT_SECRET: &str = "";
const CLIENT_ID: u64 = 1067583828543148164;
const CLIENT_SECRET: &str = "t_G0ECIQevD-V5Xqs4jHx_GFm7FNhcGJ";

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    TermLogger::init(log::LevelFilter::Trace, Config::default()).unwrap();
    let mut client = Client::new(CLIENT_ID)
        .secret(CLIENT_SECRET)
        .scope(OauthScope::RpcVoiceRead)
        .refresh_token(read_token().ok())
        .save_token(FileSaver {
            path: PathBuf::from("refresh_token"),
        })
        .connect()
        .await?;
    client.set_activity("Activity Name").await?;
    client.subscribe(EventSubscribe::VoiceChannelSelect).await?;
    loop {
        match client.event().await? {
            Event::VoiceChannelSelect(VoiceChannelSelect {
                channel_id: Some(channel_id),
                ..
            }) => {
                client
                    .subscribe(EventSubscribe::SpeakingStart { channel_id })
                    .await?;
                client
                    .subscribe(EventSubscribe::SpeakingStop { channel_id })
                    .await?;
            }
            _ => (),
        }
    }
}
