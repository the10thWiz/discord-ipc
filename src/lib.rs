pub mod activity;
pub mod channel;
mod command;
pub mod discord;
mod oauth;
mod payload;
mod platform;
pub mod voice;

pub use command::{EventResponse as Event, EventSubscribe};
pub use oauth::OauthScope;

use std::{
    collections::VecDeque,
    io,
    time::{Duration, Instant},
};

use activity::Activity;
use channel::PartialUser;
use command::{
    Authenticate, Authorize, Command, CommandWrapper, EventResponse, RPCServerConf, Subscribe,
};
use discord::Snowflake;
use log::*;
use oauth::{GrantType, TokenReq, TokenRes};
use payload::OutPayload;
use platform::PlatformSocket;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::oauth::TokenRefresh;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Underlying io error")]
    Io(#[from] std::io::Error),
    #[error("Underlying json error")]
    Json(#[from] serde_json::Error),
    #[error("Underlying http error")]
    Http(#[from] reqwest::Error),
    #[error("Invalid Event: {0}")]
    InvalidEvent(String),
    #[error("Pipe has been closed")]
    PipeClosed,
    #[error("Response Not Expected")]
    UnexpectedResponse,
    #[error("Event Not Expected")]
    UnexpectedEvent,
    #[error("Discord Error: {0:?}")]
    Discord(command::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[async_trait::async_trait]
pub trait ConnectionBuilder {
    type Socket: Connection;
    async fn connect() -> io::Result<Self::Socket>;
}

pub trait Connection: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin {}

impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin> Connection for T {}

struct Secret {
    secret: String,
    refresh_token: String,
    expires: Instant,
    save_refresh: Box<dyn Fn(&str) -> io::Result<()>>,
    scopes: Vec<OauthScope>,
}

pub struct Client<C> {
    client_id: u64,
    connection: C,
    buffer: Vec<u8>,
    event_queue: VecDeque<EventResponse>,
    user: PartialUser,

    auth: Option<Secret>,
    config: RPCServerConf,
}

impl Client<()> {
    pub fn new(client_id: u64) -> ClientBuilder {
        ClientBuilder::new(client_id)
    }
}

#[repr(u32)]
enum OpCode {
    HANDSHAKE = 0,
    FRAME = 1,
    CLOSE = 2,
    PING = 3,
    PONG = 4,
}

impl<C: Connection> Client<C> {
    async fn send_message<M: Serialize>(&mut self, opcode: OpCode, message: M) -> Result<()> {
        const WIDTH: usize = std::mem::size_of::<u32>();
        self.buffer.clear();
        // Opcode
        self.buffer
            .extend_from_slice(&u32::to_le_bytes(opcode as u32));
        // Placeholder ([WIDTH..WIDTH*2]) for len
        self.buffer.extend_from_slice(&[0u8; WIDTH]);
        serde_json::to_writer(&mut self.buffer, &message)?;
        // len = len - (Opcode + Len)
        let len = self.buffer.len() - WIDTH * 2;
        // Insert len
        self.buffer[WIDTH..WIDTH * 2].copy_from_slice(&u32::to_le_bytes(len as u32));
        trace!(
            "-> {:?}",
            std::str::from_utf8(&self.buffer[WIDTH * 2..]).unwrap_or("")
        );
        self.connection.write_all(&self.buffer).await?;
        Ok(())
    }

    async fn send_command(&mut self, command: Command) -> Result<()> {
        self.refresh_auth().await?;
        self.send_message(OpCode::FRAME, CommandWrapper::new(command))
            .await
    }

    async fn recv<M: DeserializeOwned>(&mut self) -> Result<OutPayload<M>> {
        let mut buf = [0u8; 4];
        self.connection.read_exact(&mut buf).await?;
        let ty = u32::from_le_bytes(buf);
        self.connection.read_exact(&mut buf).await?;
        let len = u32::from_le_bytes(buf);
        if ty == OpCode::FRAME as u32 {
            let mut buf = vec![0u8; len as usize];
            self.connection.read_exact(&mut buf).await?;
            trace!("<- {:?}", std::str::from_utf8(&buf).unwrap_or(""));
            Ok(payload::parse_response(&buf)?)
        } else if ty == OpCode::CLOSE as u32 {
            let mut buf = vec![0u8; len as usize];
            self.connection.read_exact(&mut buf).await?;
            trace!("<- {:?}", std::str::from_utf8(&buf).unwrap_or(""));
            Err(Error::PipeClosed)
        } else if ty == OpCode::PING as u32 {
            todo!("Ping")
        } else if ty == OpCode::PONG as u32 {
            todo!("Pong")
        } else {
            Err(Error::PipeClosed)
        }
    }

    async fn response<M: DeserializeOwned>(&mut self) -> Result<M> {
        loop {
            match self.recv().await? {
                OutPayload::Event(EventResponse::Error(e)) => return Err(Error::Discord(e)),
                OutPayload::Event(e) => self.event_queue.push_back(e),
                OutPayload::CommandResponse(m) => return Ok(m),
            }
        }
    }

    pub async fn event(&mut self) -> Result<EventResponse> {
        self.refresh_auth().await?;
        if let Some(e) = self.event_queue.pop_front() {
            Ok(e)
        } else {
            match self.recv::<()>().await? {
                OutPayload::Event(EventResponse::Error(e)) => return Err(Error::Discord(e)),
                OutPayload::Event(e) => Ok(e),
                OutPayload::CommandResponse(m) => Err(Error::UnexpectedResponse),
            }
        }
    }

    pub async fn set_activity(&mut self, activity: impl Into<Activity>) -> Result<Activity> {
        self.send_command(Command::SetActivity {
            pid: std::process::id(),
            activity: activity.into(),
        })
        .await?;
        self.response().await
    }

    pub async fn subscribe(&mut self, event: EventSubscribe) -> Result<()> {
        self.refresh_auth().await?;
        #[derive(Deserialize)]
        struct SubRes {}
        self.send_message(OpCode::FRAME, Subscribe::sub(event))
            .await?;
        self.response::<SubRes>().await.map(|_| ())
    }

    pub fn user(&self) -> &PartialUser {
        &self.user
    }

    async fn authenticate(&mut self) -> Result<()> {
        if let Some(mut auth) = self.auth.take() {
            self.send_message(
                OpCode::FRAME,
                CommandWrapper::new(Command::Authorize {
                    scopes: auth.scopes.clone(),
                    client_id: Snowflake(self.client_id),
                    rpc_token: None,
                }),
            )
            .await?;
            let token: Authorize = self.response().await?;
            let http = reqwest::Client::new();
            let res: TokenRes = http
                .post(format!("https:{}/oauth2/token", self.config.api_endpoint))
                .form(&TokenReq {
                    grant_type: GrantType::AuthorizationCode,
                    code: &token.code,
                    client_id: Snowflake(self.client_id),
                    client_secret: &auth.secret,
                })
                .send()
                .await?
                .json()
                .await?;
            let _ = (auth.save_refresh)(&res.refresh_token);
            auth.refresh_token = res.refresh_token;
            auth.expires = Instant::now() + Duration::from_secs(res.expires_in - 10);
            self.send_message(
                OpCode::FRAME,
                CommandWrapper::new(Command::Authenticate {
                    access_token: res.access_token,
                }),
            )
            .await?;
            let _: Authenticate = self.response().await?;
            self.auth = Some(auth);
        }
        Ok(())
    }

    async fn refresh_auth(&mut self) -> Result<()> {
        if let Some(auth) = self.auth.as_mut() {
            if auth.expires < Instant::now() {
                let http = reqwest::Client::new();
                let res: TokenRes = http
                    .post(format!("https:{}/oauth2/token", self.config.api_endpoint))
                    .form(&TokenRefresh {
                        grant_type: GrantType::RefreshToken,
                        refresh_token: &auth.refresh_token,
                        client_id: Snowflake(self.client_id),
                        client_secret: &auth.secret,
                    })
                    .send()
                    .await?
                    .json()
                    .await?;
                auth.refresh_token = res.refresh_token;
                auth.expires = Instant::now() + Duration::from_secs(res.expires_in - 10);
                self.send_message(
                    OpCode::FRAME,
                    CommandWrapper::new(Command::Authenticate {
                        access_token: res.access_token,
                    }),
                )
                .await?;
                loop {
                    match self.recv::<Authenticate>().await? {
                        OutPayload::Event(EventResponse::Error(e)) => {
                            return Err(Error::Discord(e))
                        }
                        OutPayload::Event(e) => self.event_queue.push_back(e),
                        OutPayload::CommandResponse(_) => break,
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct ClientBuilder {
    client_id: u64,
    scopes: Vec<OauthScope>,
    secret: Option<String>,
    refresh_token: Option<String>,
    save_refresh: Box<dyn Fn(&str) -> io::Result<()>>,
}

impl ClientBuilder {
    fn new(client_id: u64) -> Self {
        ClientBuilder {
            client_id,
            secret: None,
            scopes: vec![OauthScope::Rpc],
            refresh_token: None,
            save_refresh: Box::new(|_| Ok(())),
        }
    }

    pub fn secret(mut self, secret: impl Into<String>) -> Self {
        self.secret = Some(secret.into());
        self
    }

    pub fn scope(mut self, scope: OauthScope) -> Self {
        self.scopes.push(scope);
        self
    }

    /// Set refresh token if there is one already
    pub fn refresh_token(mut self, token: Option<impl Into<String>>) -> Self {
        self.refresh_token = token.map(|t| t.into());
        self
    }

    /// Set refresh token if there is one already
    pub fn save_token(mut self, token: impl Fn(&str) -> io::Result<()> + 'static) -> Self {
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
    pub async fn connect(self) -> Result<Client<<PlatformSocket as ConnectionBuilder>::Socket>> {
        let connection = PlatformSocket::connect().await?;
        let mut client = Client {
            client_id: self.client_id,
            connection,
            buffer: vec![],
            event_queue: Default::default(),
            user: PartialUser {
                username: String::new(),
                discriminator: String::new(),
                id: Snowflake(0),
                avatar: String::new(),
            },
            auth: None,
            config: RPCServerConf {
                cdn_host: String::new(),
                api_endpoint: String::new(),
                environment: String::new(),
            },
        };
        client
            .send_message(
                OpCode::HANDSHAKE,
                HandshakeRequest {
                    v: 1,
                    client_id: Snowflake(self.client_id),
                },
            )
            .await?;
        if let EventResponse::Ready(a) = client.event().await? {
            client.user = a.user;
            client.config = a.config;
            if let Some(secret_val) = self.secret {
                let has_refresh = self.refresh_token.is_some();
                client.auth = Some(Secret {
                    refresh_token: self.refresh_token.unwrap_or_default(),
                    expires: Instant::now() - Duration::from_secs(1),
                    secret: secret_val,
                    save_refresh: self.save_refresh,
                    scopes: self.scopes,
                });
                if has_refresh {
                    if client.refresh_auth().await.is_err() {
                        client.authenticate().await?;
                    }
                } else {
                    client.authenticate().await?;
                }
            }

            Ok(client)
        } else {
            Err(Error::UnexpectedEvent)
        }
    }
}
