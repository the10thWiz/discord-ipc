use serde::Deserialize;

use crate::{command::EventResponse as Event, Error, Result};

#[derive(Debug, PartialEq)]
pub enum OutPayload<C> {
    Event(Event),
    CommandResponse(C),
}

#[derive(Debug, Deserialize)]
struct Params<'a> {
    // TODO: this is never used
    _cmd: &'a str,
    evt: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct Data<D> {
    data: D,
}

pub fn parse_response<'a, C: Deserialize<'a>>(s: &'a [u8]) -> Result<OutPayload<C>> {
    let Params { evt, .. } = serde_json::from_slice(s)?;
    if let Some(evt) = evt {
        Ok(OutPayload::Event(match evt {
            "READY" => Event::Ready(serde_json::from_slice::<Data<_>>(s)?.data),
            "ERROR" => Event::Error(serde_json::from_slice::<Data<_>>(s)?.data),
            "GUILD_STATUS" => Event::GuildStatus(serde_json::from_slice::<Data<_>>(s)?.data),
            "GUILD_CREATE" => Event::GuildCreate(serde_json::from_slice::<Data<_>>(s)?.data),
            "CHANNEL_CREATE" => Event::ChannelCreate(serde_json::from_slice::<Data<_>>(s)?.data),
            "VOICE_CHANNEL_SELECT" => {
                Event::VoiceChannelSelect(serde_json::from_slice::<Data<_>>(s)?.data)
            }
            "VOICE_STATE_CREATE" => {
                Event::VoiceStateCreate(serde_json::from_slice::<Data<_>>(s)?.data)
            }
            "VOICE_STATE_UPDATE" => {
                Event::VoiceStateUpdate(serde_json::from_slice::<Data<_>>(s)?.data)
            }
            "VOICE_STATE_DELETE" => {
                Event::VoiceStateDelete(serde_json::from_slice::<Data<_>>(s)?.data)
            }
            "VOICE_SETTINGS_UPDATE" => {
                Event::VoiceSettingsUpdate(serde_json::from_slice::<Data<_>>(s)?.data)
            }
            "VOICE_CONNECTION_STATUS" => {
                Event::VoiceConnectionStatus(serde_json::from_slice::<Data<_>>(s)?.data)
            }
            "SPEAKING_START" => Event::SpeakingStart(serde_json::from_slice::<Data<_>>(s)?.data),
            "SPEAKING_STOP" => Event::SpeakingStop(serde_json::from_slice::<Data<_>>(s)?.data),
            "MESSAGE_CREATE" => Event::MessageCreate(serde_json::from_slice::<Data<_>>(s)?.data),
            "MESSAGE_UPDATE" => Event::MessageUpdate(serde_json::from_slice::<Data<_>>(s)?.data),
            "MESSAGE_DELETE" => Event::MessageDelete(serde_json::from_slice::<Data<_>>(s)?.data),
            "NOTIFICATION_CREATE" => {
                Event::NotificationCreate(serde_json::from_slice::<Data<_>>(s)?.data)
            }
            "ACTIVITY_JOIN" => Event::ActivityJoin(serde_json::from_slice::<Data<_>>(s)?.data),
            "ACTIVITY_SPECTATE" => {
                Event::ActivitySpectate(serde_json::from_slice::<Data<_>>(s)?.data)
            }
            "ACTIVITY_JOIN_REQUEST" => {
                Event::ActivityJoinRequest(serde_json::from_slice::<Data<_>>(s)?.data)
            }
            _ => return Err(Error::InvalidEvent(evt.to_string())),
        }))
    } else {
        Ok(OutPayload::CommandResponse(
            serde_json::from_slice::<Data<C>>(s)?.data,
        ))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        command::VoiceSettingsUpdate,
        voice::{Device, InputSettings, Key, KeyType, ModeSettings, OutputSettings},
    };

    use super::*;

    #[test]
    fn simple_test() {
        let s = "{
  \"cmd\": \"DISPATCH\",
  \"data\": {
    \"input\": {
      \"available_devices\": [
        {
          \"id\": \"default\",
          \"name\": \"Default\"
        },
        {
          \"id\": \"Built-in Microphone\",
          \"name\": \"Built-in Microphone\"
        }
      ],
      \"device_id\": \"default\",
      \"volume\": 49.803921580314636
    },
    \"output\": {
      \"available_devices\": [
        {
          \"id\": \"default\",
          \"name\": \"Default\"
        },
        {
          \"id\": \"Built-in Output\",
          \"name\": \"Built-in Output\"
        }
      ],
      \"device_id\": \"default\",
      \"volume\": 93.00000071525574
    },
    \"mode\": {
      \"type\": \"VOICE_ACTIVITY\",
      \"auto_threshold\": true,
      \"threshold\": -46.92622950819673,
      \"shortcut\": [{ \"type\": 0, \"code\": 12, \"name\": \"i\" }],
      \"delay\": 98.36065573770492
    },
    \"automatic_gain_control\": false,
    \"echo_cancellation\": false,
    \"noise_suppression\": false,
    \"qos\": false,
    \"silence_warning\": false
  },
  \"evt\": \"VOICE_SETTINGS_UPDATE\"
}";
        println!("{}", s);

        let event = parse_response::<()>(s.as_bytes()).unwrap();
        assert_eq!(
            event,
            OutPayload::Event(Event::VoiceSettingsUpdate(VoiceSettingsUpdate {
                input: Some(InputSettings {
                    device_id: "default".into(),
                    volume: 49.803921580314636,
                    available_devices: vec![
                        Device {
                            id: "default".into(),
                            name: "Default".into(),
                        },
                        Device {
                            id: "Built-in Microphone".into(),
                            name: "Built-in Microphone".into(),
                        }
                    ]
                }),
                output: Some(OutputSettings {
                    device_id: "default".into(),
                    volume: 93.00000071525574,
                    available_devices: vec![
                        Device {
                            id: "default".into(),
                            name: "Default".into(),
                        },
                        Device {
                            id: "Built-in Output".into(),
                            name: "Built-in Output".into(),
                        }
                    ]
                }),
                mode: Some(ModeSettings {
                    r#type: "VOICE_ACTIVITY".into(),
                    threshold: -46.92622950819673,
                    auto_threshold: Some(true),
                    shortcut: vec![Key {
                        r#type: KeyType::KeyboardKey,
                        code: 12,
                        name: "i".into(),
                    }],
                    delay: 98.36065573770492,
                }),
                automatic_gain_control: Some(false),
                echo_cancellation: Some(false),
                noise_suppression: Some(false),
                qos: Some(false),
                silence_warning: Some(false),
                deaf: None,
                mute: None
            }))
        )
    }

    #[allow(unused)]
    fn gen_fibb() {
        let mut a = 1;
        let mut b = 1;
        for _ in 0..100 {
            let c = a + b;
            println!("{c}");
            a = b;
            b = c;
        }
    }
}
