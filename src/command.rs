use chrono::prelude::*;
use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
    activity::Activity,
    channel::{ChannelType, Message, PartialChannel, PartialGuild, PartialUser},
    discord::Snowflake,
    oauth::{Application, OauthScope},
    voice::{
        CertifiedDevice, InputSettings, ModeSettings, OutputSettings, Pan, VoiceState, Volume,
    },
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Subscribe {
    #[serde(flatten)]
    args: EventSubscribe,
    cmd: &'static str,
    nonce: String,
}

impl Subscribe {
    pub fn sub(args: EventSubscribe) -> Self {
        Self {
            args,
            cmd: "SUBSCRIBE",
            nonce: format!("{}", DateTime::<Local>::default()),
        }
    }

    pub fn unsub(args: EventSubscribe) -> Self {
        Self {
            args,
            cmd: "UNSUBSCRIBE",
            nonce: format!("{}", DateTime::<Local>::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct CommandWrapper {
    #[serde(flatten)]
    args: Command,
    nonce: String,
}

impl CommandWrapper {
    pub fn new(args: Command) -> Self {
        Self {
            args,
            nonce: format!("{}", DateTime::<Local>::default()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "cmd", content = "args")]
pub enum Command {
    Authorize {
        scopes: Vec<OauthScope>,
        client_id: Snowflake,
        #[serde(skip_serializing_if = "Option::is_none")]
        rpc_token: Option<String>,
    },
    Authenticate {
        access_token: String,
    },

    GetGuilds {},
    GetGuild {
        guild_id: Snowflake,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
    },
    GetChannels {
        guild_id: Snowflake,
    },
    GetChannel {
        channel_id: Snowflake,
    },
    SetUserVoiceSettings {
        user_id: Snowflake,
        #[serde(skip_serializing_if = "Option::is_none")]
        pan: Option<Pan>,
        #[serde(skip_serializing_if = "Option::is_none")]
        volume: Option<Volume>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mute: Option<bool>,
    },
    SelectVoiceChannel {
        channel_id: Snowflake,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        force: Option<bool>,
    },
    GetSelectedVoiceChannel {},
    SelectTextChannel {
        channel_id: Snowflake,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u64>,
    },
    GetVoiceSettings {},
    SetVoiceSettings {
        #[serde(skip_serializing_if = "Option::is_none")]
        input: Option<InputSettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<OutputSettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mode: Option<ModeSettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        automatic_gain_control: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        echo_cancellation: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        noise_suppression: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        qos: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        silence_warning: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        deaf: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mute: Option<bool>,
    },
    SetCertifiedDevices {
        devices: Vec<CertifiedDevice>,
    },
    SetActivity {
        pid: u32,
        activity: Activity,
    },
    SendActivityJoinInvite {
        user_id: Snowflake,
    },
    CloseActivityRequest {
        user_id: Snowflake,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Authorize {
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Authenticate {
    user: PartialUser,
    scopes: Vec<OauthScope>,
    expires: DateTime<Local>,
    application: Application,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetGuilds {
    guilds: Vec<PartialGuild>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetGuild {
    id: Snowflake,
    name: String,
    icon_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetChannels {
    channels: Vec<PartialChannel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetUserVoiceSettings {
    user_id: Snowflake,
    #[serde(skip_serializing_if = "Option::is_none")]
    pan: Option<Pan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volume: Option<Volume>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mute: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetVoiceSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<InputSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<OutputSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<ModeSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    automatic_gain_control: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    echo_cancellation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    noise_suppression: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qos: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    silence_warning: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deaf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mute: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetVoiceSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<OutputSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<ModeSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automatic_gain_control: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo_cancellation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_suppression: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qos: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silence_warning: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deaf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetChannel {
    pub id: Snowflake,
    pub guild_id: Snowflake,
    pub name: String,
    pub r#type: ChannelType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u64>,
    pub user_limit: u64,
    pub position: u64,
    #[serde(default)]
    pub voice_states: Vec<VoiceState>,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CommandPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    cmd: Option<Command>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<EventSubscribe>,
}

// #[derive(Debug, Clone, PartialEq)]
// pub enum CommandResponse {
//     Command(Response),
//     Event(EventResponse),
// }

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Empty {}

impl<'de> Deserialize<'de> for Empty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(Self {})
    }
}

impl<'de> Visitor<'de> for Empty {
    type Value = Self;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "An empty type, e.g. an empty map, etc")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(self)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if let Some(k) = map.next_key::<&str>()? {
            Err(<A::Error as serde::de::Error>::custom(format!(
                "Unexpected key: {k}"
            )))
        } else {
            Ok(self)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "evt", content = "args")]
pub enum EventSubscribe {
    GuildStatus { guild_id: Snowflake },
    GuildCreate,
    ChannelCreate,
    VoiceChannelSelect,
    VoiceStateCreate { channel_id: Snowflake },
    VoiceStateUpdate { channel_id: Snowflake },
    VoiceStateDelete { channel_id: Snowflake },
    VoiceSettingsUpdate,
    VoiceConnectionStatus,
    SpeakingStart { channel_id: Snowflake },
    SpeakingStop { channel_id: Snowflake },
    MessageCreate { channel_id: Snowflake },
    MessageUpdate { channel_id: Snowflake },
    MessageDelete { channel_id: Snowflake },
    NotificationCreate,
    ActivityJoin,
    ActivitySpectate,
    ActivityJoinRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RPCServerConf {
    pub cdn_host: String,
    pub api_endpoint: String,
    pub environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ready {
    pub v: usize,
    pub config: RPCServerConf,
    pub user: PartialUser,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Error {
    pub code: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GuildStatus {
    pub guild: PartialGuild,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GuildCreate {
    pub id: Snowflake,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelCreate {
    pub id: Snowflake,
    pub name: String,
    pub r#type: ChannelType,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceChannelSelect {
    pub channel_id: Option<Snowflake>,
    pub guild_id: Option<Snowflake>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VoiceSettingsUpdate {
    pub input: Option<InputSettings>,
    pub output: Option<OutputSettings>,
    pub mode: Option<ModeSettings>,
    pub automatic_gain_control: Option<bool>,
    pub echo_cancellation: Option<bool>,
    pub noise_suppression: Option<bool>,
    pub qos: Option<bool>,
    pub silence_warning: Option<bool>,
    pub deaf: Option<bool>,
    pub mute: Option<bool>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceConnectionStatus {
    pub state: VoiceState,
    pub hostname: String,
    pub pings: Vec<u64>,
    pub average_ping: u64,
    pub last_ping: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpeakingUpdate {
    pub user_id: Snowflake,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageNotification {
    pub channel_id: Snowflake,
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationCreate {
    pub channel_id: Snowflake,
    pub message: Message,
    pub icon_url: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Secret {
    secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActivityJoinRequest {
    pub user: PartialUser,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventResponse {
    Ready(Ready),
    Error(Error),
    GuildStatus(GuildStatus),
    GuildCreate(GuildCreate),
    ChannelCreate(ChannelCreate),
    VoiceChannelSelect(VoiceChannelSelect),
    VoiceStateCreate(VoiceState),
    VoiceStateUpdate(VoiceState),
    VoiceStateDelete(VoiceState),
    VoiceSettingsUpdate(VoiceSettingsUpdate),
    VoiceConnectionStatus(VoiceConnectionStatus),
    SpeakingStart(SpeakingUpdate),
    SpeakingStop(SpeakingUpdate),
    MessageCreate(MessageNotification),
    MessageUpdate(MessageNotification),
    MessageDelete(MessageNotification),
    NotificationCreate(NotificationCreate),
    ActivityJoin(Secret),
    ActivitySpectate(Secret),
    ActivityJoinRequest(ActivityJoinRequest),
}
