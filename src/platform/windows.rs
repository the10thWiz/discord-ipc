use crate::ConnectionBuilder;
use std::io;
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

pub struct NamedPipeSocket {}

#[async_trait::async_trait]
impl ConnectionBuilder for NamedPipeSocket {
    type Socket = NamedPipeClient;

    async fn connect() -> io::Result<Self::Socket> {
        ClientOptions::new().open(r"\\.\pipe\discord-ipc-0")
    }
}
