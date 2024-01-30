use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use log::*;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    channel::PartialUser,
    command::{
        Authenticate, Authorize, Command, CommandWrapper, Empty, EventResponse, RPCServerConf,
    },
    discord::Snowflake,
    oauth::{
        GrantType, Secret, TokenRefresh, TokenRefreshServer, TokenReq, TokenReqServer, TokenRes,
    },
    payload::{self, OutPayload},
    ClientBuilder, Connection, Error, Result, SecretType,
};

pub struct Framed<C> {
    client_id: u64,
    connection: C,
    buffer: Vec<u8>,
    event_queue: VecDeque<EventResponse>,

    auth: Option<Secret>,
    pub(crate) config: RPCServerConf,
}

#[repr(u32)]
pub enum OpCode {
    HANDSHAKE = 0,
    FRAME = 1,
    CLOSE = 2,
    PING = 3,
    PONG = 4,
}

impl<C: Connection> Framed<C> {
    pub(crate) async fn connect(
        config: ClientBuilder,
        connection: C,
    ) -> Result<(Self, PartialUser)> {
        let mut client = Self {
            client_id: config.client_id,
            connection,
            buffer: vec![],
            event_queue: VecDeque::new(),

            auth: None,
            config: RPCServerConf {
                cdn_host: "".into(),
                api_endpoint: "".into(),
                environment: "".into(),
            },
        };
        client
            .send_message(
                OpCode::HANDSHAKE,
                HandshakeRequest {
                    v: 1,
                    client_id: Snowflake(client.client_id),
                },
            )
            .await?;
        if let OutPayload::Ready(a) = client.recv::<Empty>().await? {
            client.config = a.config;
            if let Some(secret_val) = config.secret {
                let refresh_token = config.save_refresh.load().await.ok().and_then(|f| f);
                let has_refresh = refresh_token.is_some();
                client.auth = Some(
                    Secret::new(
                        secret_val,
                        Instant::now() - Duration::from_secs(1),
                        config.save_refresh,
                        config.scopes,
                    )
                    .await?,
                );
                if has_refresh {
                    if client.refresh_auth().await.is_err() {
                        client.authenticate().await?;
                    }
                } else {
                    client.authenticate().await?;
                }
            }

            Ok((client, a.user))
        } else {
            Err(Error::UnexpectedEvent)
        }
    }

    pub(crate) async fn send_message<M: Serialize>(
        &mut self,
        opcode: OpCode,
        message: M,
    ) -> Result<&mut Self> {
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
        Ok(self)
    }

    pub(crate) async fn send_command(&mut self, command: Command) -> Result<&mut Self> {
        self.refresh_auth().await?;
        self.send_message(OpCode::FRAME, CommandWrapper::new(command))
            .await?;
        Ok(self)
    }

    pub(crate) async fn recv<M: DeserializeOwned>(&mut self) -> Result<OutPayload<M>> {
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

    pub(crate) async fn response<M: DeserializeOwned>(&mut self) -> Result<M> {
        loop {
            match self.recv().await? {
                OutPayload::Error(e) => return Err(Error::Discord(e)),
                OutPayload::Ready(_) => return Err(Error::UnexpectedEvent),
                OutPayload::Event(e) => self.event_queue.push_back(e),
                OutPayload::CommandResponse(m) => return Ok(m),
            }
        }
    }

    pub(crate) async fn event(&mut self) -> Result<EventResponse> {
        self.refresh_auth().await?;
        if let Some(e) = self.event_queue.pop_front() {
            Ok(e)
        } else {
            match self.recv::<()>().await? {
                OutPayload::Error(e) => return Err(Error::Discord(e)),
                OutPayload::Ready(_) => return Err(Error::UnexpectedEvent),
                OutPayload::Event(e) => Ok(e),
                OutPayload::CommandResponse(_m) => Err(Error::UnexpectedResponse),
            }
        }
    }

    pub(crate) async fn authenticate(&mut self) -> Result<()> {
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
            let access_token = auth
                .authorization_token(self.client_id, &self.config, &token.code)
                .await?;
            self.send_message(
                OpCode::FRAME,
                CommandWrapper::new(Command::Authenticate { access_token }),
            )
            .await?;
            let _: Authenticate = self.response().await?;
            self.auth = Some(auth);
        }
        Ok(())
    }

    pub(crate) async fn refresh_auth(&mut self) -> Result<()> {
        if let Some(auth) = self.auth.as_mut() {
            if let Some(access_token) = auth
                .refresh_token(self.client_id, &self.config)
                .await?
                .map(|s| s.to_string())
            {
                self.send_message(
                    OpCode::FRAME,
                    CommandWrapper::new(Command::Authenticate { access_token }),
                )
                .await?;
                loop {
                    match self.recv::<Authenticate>().await? {
                        OutPayload::Error(e) => return Err(Error::Discord(e)),
                        OutPayload::Ready(_) => return Err(Error::UnexpectedEvent),
                        OutPayload::Event(e) => self.event_queue.push_back(e),
                        OutPayload::CommandResponse(_) => break,
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct HandshakeRequest {
    v: usize,
    client_id: Snowflake,
}
