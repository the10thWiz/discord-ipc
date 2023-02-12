use std::fmt::Display;

use reqwest::Client;
use rocket::{
    form::Form,
    launch, post,
    response::Responder,
    routes,
    serde::{de::value::StrDeserializer, json::Json, Deserialize, Serialize},
    FromForm, State,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Error {
    message: String,
}

impl rocket::serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self {
            message: format!("{msg}"),
        }
    }
}

impl std::error::Error for Error {}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Responder)]
enum ApiResult<T> {
    Ok(Json<T>),
    #[response(status = 400)]
    BadRequest(Json<Error>),
    #[response(status = 500)]
    DiscordError(Json<Error>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum OauthScope {
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
#[serde(crate = "rocket::serde")]
pub struct TokenRes {
    pub(crate) access_token: String,
    pub(crate) token_type: String,
    pub(crate) expires_in: u64,
    pub(crate) refresh_token: String,
    pub(crate) scope: String,
}

#[derive(Debug, FromForm)]
struct TokenRequest<'a> {
    code: &'a str,
    client_id: &'a str,
}

#[derive(Debug, FromForm)]
struct TokenRefresh<'a> {
    refresh_token: &'a str,
    client_id: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(crate = "rocket::serde")]
pub enum GrantType {
    AuthorizationCode,
    RefreshToken,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct DiscordTokenRequest<'a> {
    grant_type: GrantType,
    code: &'a str,
    client_id: &'a str,
    client_secret: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct DiscordTokenRefresh<'a> {
    grant_type: GrantType,
    refresh_token: &'a str,
    client_id: &'a str,
    client_secret: &'a str,
}

#[post("/token", data = "<data>")]
async fn token(
    client: &State<Client>,
    secrets: &State<ClientSecrets<'static>>,
    data: Form<TokenRequest<'_>>,
) -> ApiResult<TokenRes> {
    if data.client_id != secrets.client_id {
        return ApiResult::BadRequest(Json(Error {
            message: format!("Invalid Client ID"),
        }));
    }
    let res: TokenRes = match client
        .post("https://discord.com/api/oauth2/token")
        .form(&DiscordTokenRequest {
            grant_type: GrantType::AuthorizationCode,
            code: data.code,
            client_secret: secrets.client_secret,
            client_id: secrets.client_id,
        })
        .send()
        .await
    {
        Ok(res) => match res.json().await {
            Ok(res) => res,
            Err(e) => {
                return ApiResult::DiscordError(Json(Error {
                    message: format!("Json error: {e:?}"),
                }))
            }
        },
        Err(e) => {
            return ApiResult::DiscordError(Json(Error {
                message: format!("Discord error: {e:?}"),
            }))
        }
    };
    for scope in res.scope.split_whitespace() {
        match OauthScope::deserialize(StrDeserializer::<Error>::new(scope)) {
            Ok(s) => {
                if !secrets.scopes.contains(&s) {
                    return ApiResult::BadRequest(Json(Error {
                        message: format!("Disallowed scope: {s:?}"),
                    }));
                }
            }
            Err(e) => return ApiResult::DiscordError(Json(e)),
        }
    }
    ApiResult::Ok(Json(res))
}

#[post("/refresh", data = "<data>")]
async fn refresh(
    client: &State<Client>,
    secrets: &State<ClientSecrets<'static>>,
    data: Form<TokenRefresh<'_>>,
) -> ApiResult<TokenRes> {
    if data.client_id != secrets.client_id {
        return ApiResult::BadRequest(Json(Error {
            message: format!("Invalid Client ID"),
        }));
    }
    let res: TokenRes = match client
        .post("https://discord.com/api/oauth2/token")
        .form(&DiscordTokenRefresh {
            grant_type: GrantType::RefreshToken,
            refresh_token: data.refresh_token,
            client_secret: secrets.client_secret,
            client_id: secrets.client_id,
        })
        .send()
        .await
    {
        Ok(res) => match res.json().await {
            Ok(res) => res,
            Err(e) => {
                return ApiResult::DiscordError(Json(Error {
                    message: format!("Json error: {e:?}"),
                }))
            }
        },
        Err(e) => {
            return ApiResult::DiscordError(Json(Error {
                message: format!("Discord error: {e:?}"),
            }))
        }
    };
    for scope in res.scope.split_whitespace() {
        match OauthScope::deserialize(StrDeserializer::<Error>::new(scope)) {
            Ok(s) => {
                if !secrets.scopes.contains(&s) {
                    return ApiResult::BadRequest(Json(Error {
                        message: format!("Disallowed scope: {s:?}"),
                    }));
                }
            }
            Err(e) => return ApiResult::DiscordError(Json(e)),
        }
    }
    ApiResult::Ok(Json(res))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ClientSecrets<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    scopes: Vec<OauthScope>,
}

#[launch]
fn rocket() -> _ {
    let secrects: ClientSecrets<'static> =
        rocket::serde::json::from_str(include_str!("../secrets.json")).unwrap();
    rocket::build()
        .manage(Client::new())
        .manage(secrects)
        .mount("/", routes![token, refresh])
}
