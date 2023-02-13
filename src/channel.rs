//! Discord Channel descriptions

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{activity::Emoji, discord::Snowflake};

/// Partial Discord server identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartialGuild {
    /// Guild ID
    pub id: Snowflake,
    /// Guild name
    pub name: String,
}

/// Discord server member
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GuildMember {
    /// User info
    pub user: Option<User>,
    /// User nickname
    pub nick: Option<String>,
    /// Avatar hash
    pub avatar: Option<String>,
    /// List of roles
    pub roles: Vec<Snowflake>,
    /// When the user joined
    pub joined_at: DateTime<Local>,
    /// When the user added premium. May be related to Nitro
    pub premium_since: Option<DateTime<Local>>,
    /// Is the user deafened
    pub deaf: bool,
    /// Is the user muted
    pub mute: bool,
    /// Has the user completed joining
    pub pending: Option<bool>,
    /// Does the user have any special permissions
    pub permissions: Option<String>,
    /// End of a Timeout
    pub communication_disabled_until: Option<DateTime<Local>>,
}

/// Partial User info
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartialUser {
    /// Username
    pub username: String,
    /// Discriminator, the last four numbers
    pub discriminator: String,
    /// User ID
    pub id: Snowflake,
    /// Avatar Hash
    pub avatar: Option<String>,
}

impl PartialUser {
    /// Discriminator numbers
    pub fn nums(&self) -> usize {
        self.discriminator.parse().unwrap()
    }
}

/// Full User info
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: Snowflake,
    /// User name
    pub username: String,
    /// Discriminator, the four numbers
    pub discriminator: String,
    /// Avatar Hash
    pub avatar: Option<String>,
    /// Is the user a bot
    pub bot: Option<bool>,
    /// Is the user a system account
    pub system: Option<bool>,
    /// Does the user have MFA enabled
    pub mfa_enabled: Option<bool>,
    /// Banner Hash
    pub banner: Option<String>,
    /// Accent color. Unknown
    pub accent_color: Option<u64>,
    /// Unknown
    pub locale: Option<u64>,
    /// Has the user verified their email
    pub verified: Option<bool>,
    /// User email address (if public)
    pub email: Option<String>,
    /// User flags. Unknown
    pub flags: Option<u64>,
    /// Nitro type. Unknown
    pub premium_type: Option<u64>,
    /// User Public flags. Unknown
    pub public_flags: Option<u64>,
}

/// Partial Channel info
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartialChannel {
    /// Channel ID
    pub id: Snowflake,
    /// Channel name
    pub name: String,
    /// Channel type
    pub r#type: ChannelType,
}

/// Channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ChannelType {
    /// Typical Text Channel
    GuildText = 0,
    /// Direct Message Channel
    DM = 1,
    /// Typical Voice Channel
    GuildVoice = 2,
    /// Group Direct Message Channel
    GroupDm = 3,
    /// Server Heading
    GuildCategory = 4,
    /// Server Announcement Channel
    GuildAnnouncement = 5,
    /// Announcement Thread, Unknown
    AnnouncementThread = 10,
    /// Public Thread
    PublicThread = 11,
    /// Private Thread, only visible to participants
    PrivateThread = 12,
    /// Unknown
    GuildStageVoice = 13,
    /// Unknown
    GuildDirectory = 14,
    /// Unknown
    GuildForum = 15,
}

/// Message Info
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub id: Snowflake,
    /// Channel the message was sent in
    pub channel_id: Snowflake,
    /// Author of the message
    pub author: User,
    /// Contents of the message
    pub content: String,
    /// Time sent
    pub timestamp: DateTime<Local>,
    /// Time edited (if edited)
    pub edited_timestamp: Option<DateTime<Local>>,
    /// TTS enabled
    pub tts: bool,
    /// Contains an active `@everyone`
    pub mentions_everyone: bool,
    /// List of users mentioned
    pub mentions: Vec<User>,
    /// List of roles mentioned
    pub mention_roles: Vec<Snowflake>,
    /// List of channels mentioned
    pub mention_channels: Option<Vec<ChannelMention>>,
    /// List of reactions
    pub reactions: Vec<Reaction>,
    /// Unique ID
    pub nonce: Nonce,
}

/// Unique Identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Nonce {
    /// String Encoding
    Str(String),
    /// Numeric Encoding
    Int(u64),
}

/// Message Reaction
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Reaction {
    /// Number of users who have reacted
    pub count: u64,
    /// Whether the current user has reacted with this emoji
    pub me: bool,
    /// Reaction Emoji
    pub emoji: Emoji,
}

/// Channel Mention Info
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelMention {
    /// Channel ID
    pub id: Snowflake,
    /// Guild ID
    pub guild_id: Snowflake,
    /// Channel Type
    pub r#type: ChannelType,
    /// Channel Name
    pub name: String,
}
