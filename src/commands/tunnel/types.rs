use serde::de::Error as SerdeDeError;
use serde::ser::Error as SerdeSerError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_repr::Deserialize_repr;

#[derive(Debug, Serialize, Deserialize_repr)]
#[repr(u8)]
pub enum OpCodes {
    Auth = 1,
    Connect = 2,
    Unkown = !0,
}

#[derive(Debug, Clone)]
pub enum TonneruPacket {
    Auth {
        token: String,
        resource_id: String,
        port: u16,
    },
    Connect {
        resource_id: String,
    },
}

impl<'de> Deserialize<'de> for TonneruPacket {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut gw_event = serde_json::Map::deserialize(deserializer)?;

        let op_code = gw_event
            .remove("op")
            .ok_or_else(|| serde::de::Error::missing_field("op"))
            .and_then(OpCodes::deserialize)
            .map_err(SerdeDeError::custom)?;

        match op_code {
            OpCodes::Connect => {
                let data = gw_event
                    .remove("d")
                    .ok_or_else(|| serde::de::Error::missing_field("d"))?;

                let data = data
                    .as_object()
                    .ok_or_else(|| serde::de::Error::custom("d is not an object"))?;

                let container_id = data
                    .get("container_id")
                    .ok_or_else(|| serde::de::Error::missing_field("container_id"))?;

                let container_id = container_id
                    .as_str()
                    .ok_or_else(|| serde::de::Error::custom("container_id is not a string"))?;

                Ok(TonneruPacket::Connect {
                    resource_id: container_id.to_string(),
                })
            }
            _ => Err(SerdeDeError::custom("invalid opcode received")),
        }
    }
}

impl Serialize for TonneruPacket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Auth {
                token,
                resource_id,
                port,
            } => {
                let packet = json!({
                    "op": OpCodes::Auth as u8,
                    "d": {
                        "token": token,
                        "resource_id": resource_id,
                        "port": port,
                    }
                });

                packet.serialize(serializer)
            }

            _ => Err(SerdeSerError::custom("invalid opcode sent"))?,
        }
    }
}
