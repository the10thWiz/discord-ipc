extern crate discord_ipc;
use std::time::Duration;

use discord_ipc::{Client, Result};
use simplelog::{Config, TermLogger};

const CLIENT_ID: u64 = 1067583828543148164;
const CLIENT_SECRET: &str = "t_G0ECIQevD-V5Xqs4jHx_GFm7FNhcGJ";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    TermLogger::init(log::LevelFilter::Trace, Config::default()).unwrap();
    let mut client = Client::new(CLIENT_ID)
        .secret(CLIENT_SECRET)
        .connect()
        .await?;
    client.set_activity("Activity Name").await?;
    tokio::time::sleep(Duration::from_secs(10)).await;
    Ok(())
}
