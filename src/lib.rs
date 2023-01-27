pub mod activity;
pub mod channel;
mod command;
pub mod discord;
mod oauth;
mod payload;
mod platform;
pub mod voice;

use std::{collections::VecDeque, io};

use activity::Activity;
use channel::PartialUser;
use command::{Command, CommandWrapper, EventResponse, EventSubscribe, Subscribe};
use discord::Snowflake;
use log::*;
use oauth::OauthScope;
use payload::OutPayload;
use platform::PlatformSocket;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Underlying io error")]
    Io(#[from] std::io::Error),
    #[error("Underlying json error")]
    Json(#[from] serde_json::Error),
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

pub trait Secret {
    fn secret(&self) -> &str;
}

impl Secret for &str {
    fn secret(&self) -> &str {
        self
    }
}

pub struct Client<S, C> {
    client_id: u64,
    secret: S,
    connection: C,
    buffer: Vec<u8>,
    event_queue: VecDeque<EventResponse>,
    user: PartialUser,
}

impl Client<(), ()> {
    pub fn new(client_id: u64) -> ClientBuilder<()> {
        ClientBuilder {
            client_id,
            secret: (),
            scopes: vec![],
        }
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

impl<S, C: Connection> Client<S, C> {
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
        self.send_message(
            OpCode::FRAME,
            CommandWrapper::new(Command::SetActivity {
                pid: std::process::id(),
                activity: activity.into(),
            }),
        )
        .await?;
        self.response().await
    }

    pub async fn subscribe(&mut self, event: EventSubscribe) -> Result<()> {
        #[derive(Deserialize)]
        struct SubRes {}
        self.send_message(OpCode::FRAME, Subscribe::sub(event))
            .await?;
        self.response::<SubRes>().await.map(|_| ())
    }

    pub fn user(&self) -> &PartialUser {
        &self.user
    }
}

pub struct ClientBuilder<S> {
    client_id: u64,
    secret: S,
    scopes: Vec<OauthScope>,
}

impl ClientBuilder<()> {
    pub fn secret<S: Secret>(self, secret: S) -> ClientBuilder<S> {
        ClientBuilder {
            client_id: self.client_id,
            secret,
            scopes: self.scopes,
        }
    }
}

impl<S: Secret> ClientBuilder<S> {
    pub fn scope(mut self, scope: OauthScope) -> Self {
        self.scopes.push(scope);
        self
    }
}

#[derive(Debug, Serialize)]
struct HandshakeRequest {
    v: usize,
    client_id: Snowflake,
}

impl ClientBuilder<()> {
    pub async fn connect(
        self,
    ) -> Result<Client<(), <PlatformSocket as ConnectionBuilder>::Socket>> {
        let connection = PlatformSocket::connect().await?;
        let mut client = Client {
            client_id: self.client_id,
            secret: self.secret,
            connection,
            buffer: vec![],
            event_queue: Default::default(),
            user: PartialUser {
                username: String::new(),
                discriminator: String::new(),
                id: Snowflake(0),
                avatar: String::new(),
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
            // TODO: if Secret is defined we need to authenticate
            Ok(client)
        } else {
            Err(Error::UnexpectedEvent)
        }
    }
}

impl<S: Secret> ClientBuilder<S> {
    pub async fn connect(self) -> Result<Client<S, <PlatformSocket as ConnectionBuilder>::Socket>> {
        let connection = PlatformSocket::connect().await?;
        let mut client = Client {
            client_id: self.client_id,
            secret: self.secret,
            connection,
            buffer: vec![],
            event_queue: Default::default(),
            user: PartialUser {
                username: String::new(),
                discriminator: String::new(),
                id: Snowflake(0),
                avatar: String::new(),
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
            // TODO: if Secret is defined we need to authenticate
            Ok(client)
        } else {
            Err(Error::UnexpectedEvent)
        }
    }
}
