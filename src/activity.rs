//! Rich Presence Activities

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::discord::{Snowflake, UnixTimestamp};

/// User Activity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Activity {
    /// Activity name, not sent when setting activity
    #[serde(skip_serializing)]
    name: String,
    /// Activity type
    r#type: ActivityType,
    /// Informational URL
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    /// Creation time, not sent when setting
    #[serde(skip_serializing)]
    create_at: Option<UnixTimestamp>,
    /// Start/End times, commonly used for current match
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamps: Option<TimeStamps>,
    /// App ID that set the activity, not sent when setting
    #[serde(skip_serializing)]
    application_id: Option<Snowflake>,
    /// Activity details, commonly used for additional info
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
    /// Activity state, commonly used for current in-game activity
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    /// Associated Emoji
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<Emoji>,
    /// Party information
    #[serde(skip_serializing_if = "Option::is_none")]
    party: Option<Party>,
    /// Associated images
    #[serde(skip_serializing_if = "Option::is_none")]
    assets: Option<Assets>,
    /// Secrets for joining and spectating
    #[serde(skip_serializing_if = "Option::is_none")]
    secrets: Option<Secrets>,
    /// Unknown
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<bool>,
    /// Flags
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<ActivityFlags>,
    /// Unknown
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

/// Types of Activities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ActivityType {
    /// Playing a game
    Game = 0,
    /// Streaming, e.g. Twitch?
    Streaming = 1,
    /// Listening, e.g. Spotify
    Listening = 2,
    /// Unknown
    Watching = 3,
    /// Unknown
    Custom = 4,
    /// Unknown
    Competing = 5,
}

impl Default for ActivityType {
    fn default() -> Self {
        Self::Game
    }
}

/// Start/End times
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeStamps {
    /// Start time
    start: Option<UnixTimestamp>,
    /// End time
    end: Option<UnixTimestamp>,
}

/// Emoji descriptor
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Emoji {
    /// Emoji name
    name: String,
    /// Emoji ID
    id: Option<Snowflake>,
    /// Whether to animate
    animated: Option<bool>,
}

/// Party information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Party {
    /// Party ID, if applicable
    id: Option<String>,
    /// [min, max] party size
    size: [usize; 2],
}

/// Images assets for Rich Presence
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Assets {
    /// Large image URL
    large_image: Option<AssetImage>,
    /// Large image alt text
    large_text: Option<String>,
    /// Small image URL
    small_image: Option<AssetImage>,
    /// Small image alt text
    small_text: Option<String>,
}

// TODO: this has extra restrictions
/// Asset Image URL
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetImage(String);

/// Join & Spectate secrets
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Secrets {
    /// Secret to join current game
    join: Option<String>,
    /// Secret to spectate current game
    spectate: Option<String>,
    /// Match identifier
    r#match: Option<String>,
}

bitfield::bitfield! {
    /// Activity information
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct ActivityFlags(u32);
    impl Debug;
    /// Unknown
    instance, set_instance: 0;
    /// Can join user
    join, set_join: 1;
    /// Can spectate user
    spectate, set_spectate: 2;
    /// Can ask to join user
    join_request, set_join_request: 3;
    /// Unknown
    sync, set_sync: 4;
    /// Unknown
    play, set_play: 5;
    /// Unknown
    party_privacy_friends, set_party_privacy_friends: 6;
    /// Unknown
    party_privacy_voice_channel, set_party_privacy_voice_channel: 7;
    /// Unknown
    embedded, set_embedded: 8;
}

/// Unknown
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Button {
    /// Unknown
    label: String,
    /// Unknown
    url: String,
}
