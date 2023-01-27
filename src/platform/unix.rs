use std::{env, io, path::PathBuf};

use crate::ConnectionBuilder;
use tokio::net::UnixStream;

pub struct UnixConnection;

#[async_trait::async_trait]
impl ConnectionBuilder for UnixConnection {
    type Socket = UnixStream;

    async fn connect() -> io::Result<Self::Socket> {
        let tmp = env::var("XDG_RUNTIME_DIR")
            .or_else(|_| env::var("TMPDIR"))
            .or_else(|_| match env::temp_dir().to_str() {
                None => Err("Failed to convert temp_dir"),
                Some(tmp) => Ok(tmp.to_string()),
            })
            .unwrap_or("/tmp".to_string());
        let path = PathBuf::from(tmp).join("discord-ipc-0");
        UnixStream::connect(path).await
    }
}
