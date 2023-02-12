use serde::{Deserialize, Serialize};

use crate::discord::Snowflake;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OauthScope {
    // TODO:
    #[serde(rename = "activities.read")]
    ActivitiesRead,
    #[serde(rename = "activities.write")]
    ActivitiesWrite,
    #[serde(rename = "applications.builds.read")]
    ApplicationsBuildsRead,
    #[serde(rename = "applications.builds.upload")]
    ApplicationsBuildsUpload,
    #[serde(rename = "applications.commands")]
    ApplicationsCommands,
    #[serde(rename = "applications.commands.update")]
    ApplicationsCommandsUpdate,
    #[serde(rename = "applications.commands.permissions.update")]
    ApplicationsCommandsPermissionsUpdate,
    #[serde(rename = "applications.entitlements")]
    ApplicationsEntitlements,
    #[serde(rename = "applications.store.update")]
    ApplicationsStoreUpdate,
    #[serde(rename = "bot")]
    Bot,
    #[serde(rename = "connections")]
    Connections,
    #[serde(rename = "dm_channels.read")]
    DmChannelsRead,
    #[serde(rename = "email")]
    Email,
    #[serde(rename = "gdm.join")]
    GdmJoin,
    #[serde(rename = "guilds")]
    Guilds,
    #[serde(rename = "guilds.join")]
    GuildsJoin,
    #[serde(rename = "guilds.members.read")]
    GuildsMembersRead,
    #[serde(rename = "identify")]
    Identify,
    #[serde(rename = "messages.read")]
    MessagesRead,
    #[serde(rename = "relationships.read")]
    RelationshipsRead,
    #[serde(rename = "role_connections.write")]
    RoleConnectionsWrite,
    #[serde(rename = "rpc")]
    Rpc,
    #[serde(rename = "rpc.activities.write")]
    RpcActivitiesWrite,
    #[serde(rename = "rpc.notifications.read")]
    RpcNotificationsRead,
    #[serde(rename = "rpc.voice.read")]
    RpcVoiceRead,
    #[serde(rename = "rpc.voice.write")]
    RpcVoiceWrite,
    #[serde(rename = "voice")]
    Voice,
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
