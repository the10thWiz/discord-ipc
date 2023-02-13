#![warn(missing_docs)]
//! # Discord IPC Client

pub mod activity;
pub mod channel;
mod command;
pub mod discord;
mod ipc;
mod oauth;
mod payload;
mod platform;
pub mod voice;

pub use command::{EventResponse as Event, EventSubscribe};
use ipc::Framed;
pub use oauth::OauthScope;

use std::{
    io::{self, ErrorKind},
    path::PathBuf,
    time::Instant,
};

use activity::Activity;
use channel::PartialUser;
use command::{Command, EventResponse, GetChannel, Subscribe};
use discord::Snowflake;
use log::*;
use platform::PlatformSocket;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::ipc::OpCode;

/// An Error returned by the library
#[derive(Debug, Error)]
pub enum Error {
    /// An IO error in the underlying socket connection
    #[error("Underlying io error: {0}")]
    Io(#[from] std::io::Error),
    /// An error when parsing json from discord or the secret server
    #[error("Underlying json error: {0}")]
    Json(#[from] serde_json::Error),
    /// An error when requesting an authentication token from discord
    #[error("Underlying http error: {0}")]
    Http(#[from] reqwest::Error),
    /// An Invalid event was sent by Discord
    #[error("Invalid Event: {0}")]
    InvalidEvent(String),
    /// The connection has been closed by Discord
    #[error("Pipe has been closed")]
    PipeClosed,
    /// Discord sent an unexpected response
    #[error("Response Not Expected")]
    UnexpectedResponse,
    /// Discord sent an unexpected event
    #[error("Event Not Expected")]
    UnexpectedEvent,
    /// Discord sent an error in response to a command or subscribe request
    #[error("Discord Error: {0:?}")]
    Discord(command::Error),
}

/// Result alias for `Result<T, Error>`
pub type Result<T> = std::result::Result<T, Error>;

/// Connection Builder
///
/// Includes a socket type & connection strategy
#[async_trait::async_trait]
pub trait ConnectionBuilder {
    /// Associated Socket type generated when connected
    type Socket: Connection;
    /// Connect to a local discord client
    async fn connect() -> io::Result<Self::Socket>;
}

/// Convience trait for Socket types
pub trait Connection: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin {}

impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin> Connection for T {}

struct Secret {
    secret: SecretType,
    refresh_token: String,
    expires: Instant,
    save_refresh: Box<dyn TokenSaver>,
    scopes: Vec<OauthScope>,
}

enum SecretType {
    Local(String),
    Remote(String),
}

/// Client construct
pub struct Client<C> {
    framed: Framed<C>,
    user: PartialUser,
}

impl Client<()> {
    /// Create a new ClientBuilder with the specified client_id
    pub fn new(client_id: u64) -> ClientBuilder {
        ClientBuilder::new(client_id)
    }
}

impl<C: Connection> Client<C> {
    /// Update the User's current activity
    pub async fn set_activity(&mut self, activity: impl Into<Activity>) -> Result<Activity> {
        self.framed
            .send_command(Command::SetActivity {
                pid: std::process::id(),
                activity: activity.into(),
            })
            .await?;
        self.framed.response().await
    }

    /// Get the user's selected voice channel
    pub async fn get_selected_channel(&mut self) -> Result<Option<GetChannel>> {
        self.framed
            .send_command(Command::GetSelectedVoiceChannel {})
            .await?
            .response()
            .await
    }

    /// Subscribe to an event. Use `.event().await` to wait for events
    pub async fn subscribe(&mut self, event: EventSubscribe) -> Result<()> {
        self.framed.refresh_auth().await?;
        #[derive(Deserialize)]
        struct SubRes {}
        self.framed
            .send_message(OpCode::FRAME, Subscribe::sub(event))
            .await?
            .response::<SubRes>()
            .await
            .map(|_| ())
    }

    /// Unsubscribe from events previously subscribed to
    pub async fn unsubscribe(&mut self, event: EventSubscribe) -> Result<()> {
        self.framed.refresh_auth().await?;
        #[derive(Deserialize)]
        struct SubRes {}
        self.framed
            .send_message(OpCode::FRAME, Subscribe::unsub(event))
            .await?;
        self.framed.response::<SubRes>().await.map(|_| ())
    }

    /// Wait for a discord event to be sent
    pub async fn event(&mut self) -> Result<EventResponse> {
        self.framed.event().await
    }

    /// User information provided during connection
    pub fn user(&self) -> &PartialUser {
        &self.user
    }
}

/// Struct to save refresh tokens locally between executions
#[async_trait::async_trait]
pub trait TokenSaver {
    /// Save token to external location
    async fn save(&self, token: &str) -> io::Result<()>;
    /// Load token from external location
    async fn load(&self) -> io::Result<Option<String>>;
}

/// TokenSaver that uses a local file to save the token
pub struct FileSaver {
    /// Path to save the token to
    pub path: PathBuf,
}

#[async_trait::async_trait]
impl TokenSaver for FileSaver {
    async fn save(&self, token: &str) -> io::Result<()> {
        File::create(&self.path)
            .await?
            .write_all(token.as_bytes())
            .await
    }

    async fn load(&self) -> io::Result<Option<String>> {
        match File::open(&self.path).await {
            Ok(mut f) => {
                let mut s = String::new();
                f.read_to_string(&mut s).await?;
                Ok(Some(s))
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// TokenSaver that doesn't save
struct NoneSaver;

#[async_trait::async_trait]
impl TokenSaver for NoneSaver {
    async fn save(&self, _: &str) -> io::Result<()> {
        Ok(())
    }

    async fn load(&self) -> io::Result<Option<String>> {
        Ok(None)
    }
}

/// Builder for the `Client` struct
pub struct ClientBuilder {
    client_id: u64,
    scopes: Vec<OauthScope>,
    secret: Option<SecretType>,
    save_refresh: Box<dyn TokenSaver>,
}

impl ClientBuilder {
    fn new(client_id: u64) -> Self {
        ClientBuilder {
            client_id,
            secret: None,
            scopes: vec![OauthScope::Rpc],
            save_refresh: Box::new(NoneSaver),
        }
    }

    /// Insert a local secret. The value passed should be the Secret provided by Discord
    pub fn secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(SecretType::Local(secret.into()));
        self
    }

    /// Insert a remote secret. The value passed is used as a domain name to make the token request
    /// to. The server is expected to make a request to Discord on your app's behalf, with a client
    /// secret stored on the server.
    pub fn remote_secret(mut self, server: impl Into<String>) -> Self {
        self.secret = Some(SecretType::Remote(server.into()));
        self
    }

    /// Insert an OauthScope
    pub fn scope(mut self, scope: OauthScope) -> Self {
        self.scopes.push(scope);
        self
    }

    // /// Set refresh token if there is one already
    // pub fn refresh_token(mut self, token: Option<impl Into<String>>) -> Self {
    //     self.refresh_token = token.map(|t| t.into());
    //     self
    // }

    /// Insert TokenSaver
    pub fn save_token(mut self, token: impl TokenSaver + 'static) -> Self {
        self.save_refresh = Box::new(token);
        self
    }
}

#[derive(Debug, Serialize)]
struct HandshakeRequest {
    v: usize,
    client_id: Snowflake,
}

impl ClientBuilder {
    /// Connect to a running Discord client and authenticate
    pub async fn connect(self) -> Result<Client<<PlatformSocket as ConnectionBuilder>::Socket>> {
        let connection = PlatformSocket::connect().await?;
        let (framed, user) = Framed::connect(self, connection).await?;
        Ok(Client { framed, user })
    }
}
