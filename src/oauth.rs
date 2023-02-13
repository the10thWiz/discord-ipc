use serde::{Deserialize, Serialize};

use crate::discord::Snowflake;

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
