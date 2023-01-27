use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::discord::{Snowflake, UnixTimestamp};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Activity {
    #[serde(skip_serializing)]
    name: String,
    r#type: ActivityType,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(skip_serializing)]
    create_at: Option<UnixTimestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamps: Option<TimeStamps>,
    #[serde(skip_serializing)]
    application_id: Option<Snowflake>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<Emoji>,
    #[serde(skip_serializing_if = "Option::is_none")]
    party: Option<Party>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assets: Option<Assets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secrets: Option<Secrets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<ActivityFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    buttons: Option<Vec<Button>>,
}

impl From<&str> for Activity {
    fn from(s: &str) -> Self {
        Self {
            state: Some(s.to_string()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ActivityType {
    Game = 0,
    Streaming = 1,
    Listening = 2,
    Watching = 3,
    Custom = 4,
    Competing = 5,
}

impl Default for ActivityType {
    fn default() -> Self {
        Self::Game
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeStamps {
    start: Option<UnixTimestamp>,
    end: Option<UnixTimestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Emoji {
    name: String,
    id: Option<Snowflake>,
    animated: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Party {
    id: Option<String>,
    size: [usize; 2],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Assets {
    large_image: Option<AssetImage>,
    large_text: Option<String>,
    small_image: Option<AssetImage>,
    small_text: Option<String>,
}

// TODO: this has extra restrictions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetImage(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Secrets {
    join: Option<String>,
    spectate: Option<String>,
    r#match: Option<String>,
}

bitfield::bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct ActivityFlags(u32);
    impl Debug;
    instance, set_instance: 0;
    join, set_join: 1;
    spectate, set_spectate: 2;
    join_request, set_join_request: 3;
    sync, set_sync: 4;
    play, set_play: 5;
    party_privacy_friends, set_party_privacy_friends: 6;
    party_privacy_voice_channel, set_party_privacy_voice_channel: 7;
    embedded, set_embedded: 8;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Button {
    label: String,
    url: String,
}
