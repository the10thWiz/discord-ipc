use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{channel::GuildMember, discord::Snowflake};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Volume(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pan {
    pub left: f32,
    pub right: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Details {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    AudioInput,
    AudioOutput,
    VideoInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CertifiedDevice {
    pub r#type: DeviceType,
    /// Windows UUID
    pub id: String,
    pub vendor: Details,
    pub model: Details,
    pub related: Vec<String>,
    pub echo_cancellation: Option<bool>,
    pub noise_suppression: Option<bool>,
    pub automatic_gain_control: Option<bool>,
    pub hardware_mute: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputSettings {
    pub device_id: String,
    pub volume: f32,
    pub available_devices: Vec<Device>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutputSettings {
    pub device_id: String,
    pub volume: f32,
    pub available_devices: Vec<Device>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeSettings {
    pub r#type: String,
    pub threshold: f32,
    pub auto_threshold: Option<bool>,
    pub shortcut: Vec<Key>,
    pub delay: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key {
    pub r#type: KeyType,
    pub code: u64,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum KeyType {
    KeyboardKey = 0,
    MouseButton = 1,
    KeyboardModifierKey = 2,
    GamepadButton = 3,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceState {
    guild_id: Option<Snowflake>,
    channel_id: Option<Snowflake>,
    user_id: Snowflake,
    member: Option<GuildMember>,
    session_id: Snowflake,
    deaf: bool,
    mute: bool,
    self_deaf: bool,
    self_mute: bool,
    self_stream: Option<bool>,
    self_video: bool,
    suppress: bool,
    request_to_speak_timestamp: Option<DateTime<Local>>,
}
