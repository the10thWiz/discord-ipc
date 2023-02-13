use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use log::*;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    channel::PartialUser,
    command::{Authenticate, Authorize, Command, CommandWrapper, EventResponse, RPCServerConf},
    discord::Snowflake,
    oauth::{GrantType, TokenRefresh, TokenRefreshServer, TokenReq, TokenReqServer, TokenRes},
    payload::{self, OutPayload},
    ClientBuilder, Connection, Error, Result, Secret, SecretType,
};

pub struct Framed<C> {
    client_id: u64,
    connection: C,
    buffer: Vec<u8>,
    event_queue: VecDeque<EventResponse>,

    auth: Option<Secret>,
    pub config: RPCServerConf,
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
    pub async fn connect(config: ClientBuilder, connection: C) -> Result<(Self, PartialUser)> {
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
        if let EventResponse::Ready(a) = client.event().await? {
            client.config = a.config;
            if let Some(secret_val) = config.secret {
                let refresh_token = config.save_refresh.load().await.ok().and_then(|f| f);
                let has_refresh = refresh_token.is_some();
                client.auth = Some(Secret {
                    refresh_token: refresh_token.unwrap_or_default(),
                    expires: Instant::now() - Duration::from_secs(1),
                    secret: secret_val,
                    save_refresh: config.save_refresh,
                    scopes: config.scopes,
                });
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

    pub async fn send_message<M: Serialize>(
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

    pub async fn send_command(&mut self, command: Command) -> Result<&mut Self> {
        self.refresh_auth().await?;
        self.send_message(OpCode::FRAME, CommandWrapper::new(command))
            .await?;
        Ok(self)
    }

    pub async fn recv<M: DeserializeOwned>(&mut self) -> Result<OutPayload<M>> {
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

    pub async fn response<M: DeserializeOwned>(&mut self) -> Result<M> {
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
                OutPayload::CommandResponse(_m) => Err(Error::UnexpectedResponse),
            }
        }
    }

    pub async fn authenticate(&mut self) -> Result<()> {
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
            let res: TokenRes = match &auth.secret {
                SecretType::Local(secret) => {
                    http.post(format!("https:{}/oauth2/token", self.config.api_endpoint))
                        .form(&TokenReq {
                            grant_type: GrantType::AuthorizationCode,
                            code: &token.code,
                            client_id: Snowflake(self.client_id),
                            client_secret: secret,
                        })
                        .send()
                        .await?
                        .json()
                        .await?
                }
                SecretType::Remote(server) => {
                    http.post(format!("https:{}/refresh", server))
                        .form(&TokenReqServer {
                            code: &token.code,
                            client_id: Snowflake(self.client_id),
                        })
                        .send()
                        .await?
                        .json()
                        .await?
                }
            };
            let _ = auth.save_refresh.save(&res.refresh_token).await;
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

    pub async fn refresh_auth(&mut self) -> Result<()> {
        if let Some(auth) = self.auth.as_mut() {
            if auth.expires < Instant::now() {
                let http = reqwest::Client::new();
                let res: TokenRes = match &auth.secret {
                    SecretType::Local(secret) => {
                        http.post(format!("https:{}/oauth2/token", self.config.api_endpoint))
                            .form(&TokenRefresh {
                                grant_type: GrantType::RefreshToken,
                                refresh_token: &auth.refresh_token,
                                client_id: Snowflake(self.client_id),
                                client_secret: secret,
                            })
                            .send()
                            .await?
                            .json()
                            .await?
                    }
                    SecretType::Remote(server) => {
                        http.post(format!("https:{}/refresh", server))
                            .form(&TokenRefreshServer {
                                refresh_token: &auth.refresh_token,
                                client_id: Snowflake(self.client_id),
                            })
                            .send()
                            .await?
                            .json()
                            .await?
                    }
                };
                let _ = auth.save_refresh.save(&res.refresh_token).await;
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

#[derive(Debug, Serialize)]
struct HandshakeRequest {
    v: usize,
    client_id: Snowflake,
}

// impl ClientBuilder {
//     pub async fn connect(self) -> Result<Client<<PlatformSocket as ConnectionBuilder>::Socket>> {
//         let connection = PlatformSocket::connect().await?;
//         let mut client = Client {
//             client_id: self.client_id,
//             connection,
//             buffer: vec![],
//             event_queue: Default::default(),
//             user: PartialUser {
//                 username: String::new(),
//                 discriminator: String::new(),
//                 id: Snowflake(0),
//                 avatar: None,
//             },
//             auth: None,
//             config: RPCServerConf {
//                 cdn_host: String::new(),
//                 api_endpoint: String::new(),
//                 environment: String::new(),
//             },
//         };
//         client
//             .send_message(
//                 OpCode::HANDSHAKE,
//                 HandshakeRequest {
//                     v: 1,
//                     client_id: Snowflake(self.client_id),
//                 },
//             )
//             .await?;
//         if let EventResponse::Ready(a) = client.event().await? {
//             client.user = a.user;
//             client.config = a.config;
//             if let Some(secret_val) = self.secret {
//                 let has_refresh = self.refresh_token.is_some();
//                 client.auth = Some(Secret {
//                     refresh_token: self.refresh_token.unwrap_or_default(),
//                     expires: Instant::now() - Duration::from_secs(1),
//                     secret: secret_val,
//                     save_refresh: self.save_refresh,
//                     scopes: self.scopes,
//                 });
//                 if has_refresh {
//                     if client.refresh_auth().await.is_err() {
//                         client.authenticate().await?;
//                     }
//                 } else {
//                     client.authenticate().await?;
//                 }
//             }
//
//             Ok(client)
//         } else {
//             Err(Error::UnexpectedEvent)
//         }
//     }
// }
