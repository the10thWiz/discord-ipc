//! Voice Channel Details

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    channel::{GuildMember, PartialUser},
    discord::Snowflake,
};

/// Volume level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Volume(pub u8);

/// Volume Pan (left/right balance)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pan {
    /// Left balance
    pub left: f32,
    /// Right balance
    pub right: f32,
}

/// Details of a Vendor or Model
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Details {
    /// Vendor/Model name
    pub name: String,
    /// Associated URL
    pub url: String,
}

/// Audio Device Type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    /// Input Device
    AudioInput,
    /// Output Device
    AudioOutput,
    /// Video Device
    VideoInput,
}

/// Certified device information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CertifiedDevice {
    /// Device Type
    pub r#type: DeviceType,
    /// Windows Device UUID
    pub id: String,
    /// Vendor details
    pub vendor: Details,
    /// Model details
    pub model: Details,
    /// UUIDs of related devices
    pub related: Vec<String>,
    /// Whether echo cancellation is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo_cancellation: Option<bool>,
    /// Whether noise suppression is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_suppression: Option<bool>,
    /// Whether automatic gain control is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automatic_gain_control: Option<bool>,
    /// Whether hardware mute is enabled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_mute: Option<bool>,
}

/// Auto Device Description
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Device {
    /// Unique ID
    pub id: String,
    /// Human friendly name
    pub name: String,
}

/// Device Input Settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputSettings {
    /// Device to use
    pub device_id: String,
    /// Input Volume
    pub volume: f32,
    /// List of available devices (readonly)
    pub available_devices: Vec<Device>,
}

/// Device Output Settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputSettings {
    /// Device to use
    pub device_id: String,
    /// Output Volume
    pub volume: f32,
    /// List of available devices (readonly)
    pub available_devices: Vec<Device>,
}

/// Voice Mode Settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeSettings {
    /// Voice Mode
    pub r#type: VoiceMode,
    /// Voice activity threshold
    pub threshold: f32,
    /// Voice activity automatic threshold
    pub auto_threshold: Option<bool>,
    /// Push to talk shortcut key
    pub shortcut: Vec<Key>,
    /// Push to talk delay
    pub delay: f32,
}

/// Push to talk vs Voice activity
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VoiceMode {
    /// Push to talk
    PushToTalk,
    /// Voice Activity
    VoiceActivity,
}

/// Shortcut Key
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key {
    /// Key location
    pub r#type: KeyType,
    /// Key Code
    pub code: u64,
    /// Key Name
    pub name: String,
}

/// Key location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum KeyType {
    /// On Keyboard
    KeyboardKey = 0,
    /// On Mouse
    MouseButton = 1,
    /// Keyboard Modifier
    KeyboardModifierKey = 2,
    /// On Gamepad
    GamepadButton = 3,
}

/// Current Voice State
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceState {
    /// TODO
    pub guild_id: Option<Snowflake>,
    /// TODO
    pub channel_id: Option<Snowflake>,
    /// TODO
    pub user_id: Option<Snowflake>,
    /// TODO
    pub member: Option<GuildMember>,
    /// TODO
    pub user: Option<PartialUser>,
    /// TODO
    pub session_id: Option<Snowflake>,
    /// TODO
    #[serde(default)]
    pub deaf: bool,
    /// TODO
    #[serde(default)]
    pub mute: bool,
    /// TODO
    #[serde(default)]
    pub self_deaf: bool,
    /// TODO
    #[serde(default)]
    pub self_mute: bool,
    /// TODO
    #[serde(default)]
    pub self_stream: Option<bool>,
    /// TODO
    #[serde(default)]
    pub self_video: bool,
    /// TODO
    #[serde(default)]
    pub suppress: bool,
    /// TODO
    #[serde(default)]
    pub request_to_speak_timestamp: Option<DateTime<Local>>,
}

impl VoiceState {
    /// TODO
    pub fn user_id(&self) -> Snowflake {
        self.user_id
            .or_else(|| self.user.as_ref().map(|u| u.id))
            .unwrap_or_else(|| self.member.as_ref().unwrap().user.as_ref().unwrap().id)
    }
}

