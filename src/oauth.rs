use std::{
    io::{self, ErrorKind},
    path::PathBuf,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{command::RPCServerConf, discord::Snowflake, Result};

/// Oauth Scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OauthScope {
    /// Read current activity
    #[serde(rename = "activities.read")]
    ActivitiesRead,
    /// Set current activity
    #[serde(rename = "activities.write")]
    ActivitiesWrite,
    /// Unknown
    #[serde(rename = "applications.builds.read")]
    ApplicationsBuildsRead,
    /// Unknown
    #[serde(rename = "applications.builds.upload")]
    ApplicationsBuildsUpload,
    /// Unknown
    #[serde(rename = "applications.commands")]
    ApplicationsCommands,
    /// Unknown
    #[serde(rename = "applications.commands.update")]
    ApplicationsCommandsUpdate,
    /// Unknown
    #[serde(rename = "applications.commands.permissions.update")]
    ApplicationsCommandsPermissionsUpdate,
    /// Unknown
    #[serde(rename = "applications.entitlements")]
    ApplicationsEntitlements,
    /// Unknown
    #[serde(rename = "applications.store.update")]
    ApplicationsStoreUpdate,
    /// Bot User
    #[serde(rename = "bot")]
    Bot,
    /// Unknown
    #[serde(rename = "connections")]
    Connections,
    /// Read User DMs
    #[serde(rename = "dm_channels.read")]
    DmChannelsRead,
    /// Get User's email
    #[serde(rename = "email")]
    Email,
    /// Unknown
    #[serde(rename = "gdm.join")]
    GdmJoin,
    /// List Guilds joined by user
    #[serde(rename = "guilds")]
    Guilds,
    /// Join guild on behalf of a user
    #[serde(rename = "guilds.join")]
    GuildsJoin,
    /// List members of a guild
    #[serde(rename = "guilds.members.read")]
    GuildsMembersRead,
    /// Access to `/user/@me`
    #[serde(rename = "identify")]
    Identify,
    /// Read messaged
    #[serde(rename = "messages.read")]
    MessagesRead,
    /// List Friends
    #[serde(rename = "relationships.read")]
    RelationshipsRead,
    /// Unknown
    #[serde(rename = "role_connections.write")]
    RoleConnectionsWrite,
    /// Access RPC - included by default
    #[serde(rename = "rpc")]
    Rpc,
    /// Set RPC activity
    #[serde(rename = "rpc.activities.write")]
    RpcActivitiesWrite,
    /// Get RPC activity
    #[serde(rename = "rpc.notifications.read")]
    RpcNotificationsRead,
    /// Get Voice activity
    #[serde(rename = "rpc.voice.read")]
    RpcVoiceRead,
    /// Set Voice settings
    #[serde(rename = "rpc.voice.write")]
    RpcVoiceWrite,
    /// Voice permissions
    #[serde(rename = "voice")]
    Voice,
    /// Unknown
    #[serde(rename = "webhook.incoming")]
    WebhookIncoming,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Application {
    description: String,
    icon: Option<String>,
    id: Snowflake,
    #[serde(default)]
    rpc_origins: Vec<String>,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct TokenRefresh<'a> {
    pub(crate) grant_type: GrantType,
    pub(crate) refresh_token: &'a str,
    pub(crate) client_id: Snowflake,
    pub(crate) client_secret: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct TokenReq<'a> {
    pub(crate) grant_type: GrantType,
    pub(crate) code: &'a str,
    pub(crate) client_id: Snowflake,
    pub(crate) client_secret: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantType {
    AuthorizationCode,
    RefreshToken,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct TokenRefreshServer<'a> {
    pub(crate) refresh_token: &'a str,
    pub(crate) client_id: Snowflake,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct TokenReqServer<'a> {
    pub(crate) code: &'a str,
    pub(crate) client_id: Snowflake,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenRes {
    pub(crate) access_token: String,
    pub(crate) token_type: String,
    pub(crate) expires_in: u64,
    pub(crate) refresh_token: String,
    pub(crate) scope: String,
}

pub(crate) struct Secret {
    secret: SecretType,
    refresh_token: String,
    expires: Instant,
    save_refresh: Box<dyn TokenSaver>,
    pub scopes: Vec<OauthScope>,
}

impl Secret {
    pub async fn new(
        secret: SecretType,
        expires: Instant,
        save_refresh: Box<dyn TokenSaver>,
        scopes: Vec<OauthScope>,
    ) -> io::Result<Self> {
        Ok(Self {
            secret,
            expires,
            refresh_token: save_refresh.load().await?.unwrap_or_default(),
            save_refresh,
            scopes,
        })
    }

    /// Convert a code into an access token via either a local secret or a remote server
    pub async fn authorization_token(
        &mut self,
        client_id: u64,
        config: &RPCServerConf,
        token: &str,
    ) -> Result<String> {
        let http = reqwest::Client::new();
        let res: TokenRes = match &self.secret {
            SecretType::Local(secret) => {
                http.post(format!("https:{}/oauth2/token", config.api_endpoint))
                    .form(&TokenReq {
                        grant_type: GrantType::AuthorizationCode,
                        code: token,
                        client_id: Snowflake(client_id),
                        client_secret: secret.as_str(),
                    })
                    .send()
                    .await?
                    .json()
                    .await?
            }
            SecretType::Remote(server) => {
                http.post(format!("https:{}/refresh", server))
                    .form(&TokenReqServer {
                        code: token,
                        client_id: Snowflake(client_id),
                    })
                    .send()
                    .await?
                    .json()
                    .await?
            }
        };
        let _ = self.save_refresh.save(&res.refresh_token).await;
        self.refresh_token = res.refresh_token;
        self.expires = Instant::now() + Duration::from_secs(res.expires_in - 10);
        Ok(res.access_token)
    }

    pub async fn refresh_token<'s>(
        &'s mut self,
        client_id: u64,
        config: &RPCServerConf,
    ) -> Result<Option<&'s str>> {
        if self.expires < Instant::now() {
            let http = reqwest::Client::new();
            let res: TokenRes = match &self.secret {
                SecretType::Local(secret) => {
                    http.post(format!("https:{}/oauth2/token", config.api_endpoint))
                        .form(&TokenRefresh {
                            grant_type: GrantType::RefreshToken,
                            refresh_token: &self.refresh_token,
                            client_id: Snowflake(client_id),
                            client_secret: secret.as_str(),
                        })
                        .send()
                        .await?
                        .json()
                        .await?
                }
                SecretType::Remote(server) => {
                    http.post(format!("https:{}/refresh", server))
                        .form(&TokenRefreshServer {
                            refresh_token: &self.refresh_token,
                            client_id: Snowflake(client_id),
                        })
                        .send()
                        .await?
                        .json()
                        .await?
                }
            };
            let _ = self.save_refresh.save(&res.refresh_token).await;
            self.refresh_token = res.refresh_token;
            self.expires = Instant::now() + Duration::from_secs(res.expires_in - 10);
            Ok(Some(&self.refresh_token))
        } else {
            Ok(None)
        }
    }

    // pub(crate) async fn refresh_auth(&mut self) -> Result<()> {
    //     if let Some(auth) = self.auth.as_mut() {
    //         if auth.expires < Instant::now() {
    //             let http = reqwest::Client::new();
    //             let res: TokenRes = match &auth.secret {
    //                 SecretType::Local(secret) => {
    //                     http.post(format!("https:{}/oauth2/token", self.config.api_endpoint))
    //                         .form(&TokenRefresh {
    //                             grant_type: GrantType::RefreshToken,
    //                             refresh_token: &auth.refresh_token,
    //                             client_id: Snowflake(self.client_id),
    //                             client_secret: secret.as_str(),
    //                         })
    //                         .send()
    //                         .await?
    //                         .json()
    //                         .await?
    //                 }
    //                 SecretType::Remote(server) => {
    //                     http.post(format!("https:{}/refresh", server))
    //                         .form(&TokenRefreshServer {
    //                             refresh_token: &auth.refresh_token,
    //                             client_id: Snowflake(self.client_id),
    //                         })
    //                         .send()
    //                         .await?
    //                         .json()
    //                         .await?
    //                 }
    //             };
    //             let _ = auth.save_refresh.save(&res.refresh_token).await;
    //             auth.refresh_token = res.refresh_token;
    //             auth.expires = Instant::now() + Duration::from_secs(res.expires_in - 10);
    //             self.send_message(
    //                 OpCode::FRAME,
    //                 CommandWrapper::new(Command::Authenticate {
    //                     access_token: res.access_token,
    //                 }),
    //             )
    //             .await?;
    //             loop {
    //                 match self.recv::<Authenticate>().await? {
    //                     OutPayload::Error(e) => return Err(Error::Discord(e)),
    //                     OutPayload::Ready(_) => return Err(Error::UnexpectedEvent),
    //                     OutPayload::Event(e) => self.event_queue.push_back(e),
    //                     OutPayload::CommandResponse(_) => break,
    //                 }
    //             }
    //         }
    //     }
    //     Ok(())
    // }
}

pub(crate) enum SecretType {
    Local(String),
    Remote(String),
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
pub struct NoneSaver;

#[async_trait::async_trait]
impl TokenSaver for NoneSaver {
    async fn save(&self, _: &str) -> io::Result<()> {
        Ok(())
    }

    async fn load(&self) -> io::Result<Option<String>> {
        Ok(None)
    }
}
