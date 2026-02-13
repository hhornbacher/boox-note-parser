use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, OnceLock},
};

trait CheckUuid {
    fn id(&self) -> &uuid::Uuid;
    fn to_string(&self) -> String;
}
static KNOWN_UUIDS: OnceLock<Mutex<HashMap<uuid::Uuid, Vec<Box<dyn CheckUuid + Send + Sync>>>>> =
    OnceLock::new();

fn normalized_uuid_input(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_ascii_whitespace() || c == '\0')
}

fn check_uuid(id: impl CheckUuid + Send + Sync + 'static) {
    // Lock the mutex and clear the HashMap
    let mut map = KNOWN_UUIDS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();
    // Check if the id already exists in the HashMap
    if let Some(existing) = map.get(id.id()) {
        // log::warn!("Duplicate UUID found: {}. Already registered as:", id.to_string());
        let mut types = HashSet::new();
        for existing_id in existing {
            types.insert(existing_id.to_string());
        }

        if types.len() > 1 {
            log::warn!(
                "UUIDs with different types found: {}. Already registered as: {}",
                id.to_string(),
                types.into_iter().collect::<Vec<_>>().join(", ")
            );
        }
    }

    // Insert the id into the HashMap
    map.entry(id.id().clone()).or_default().push(Box::new(id));
}

macro_rules! implement_uuid {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $name(uuid::Uuid);

        impl $name {
            pub fn new(id: uuid::Uuid) -> Self {
                let id = Self(id);
                check_uuid(id.clone());
                id
            }

            pub fn from_str(s: &str) -> crate::error::Result<Self> {
                let normalized = normalized_uuid_input(s);
                let parsed = uuid::Uuid::parse_str(normalized).map_err(|e| {
                    log::error!(
                        "Failed to parse UUID from string '{}'(normalized '{}'): {}",
                        s.escape_debug(),
                        normalized.escape_debug(),
                        e
                    );
                    e
                })?;
                let id = Self(parsed);
                check_uuid(id.clone());
                Ok(id)
            }

            pub fn from_byte_str(s: &[u8]) -> crate::error::Result<Self> {
                let s =
                    std::str::from_utf8(s).map_err(|e| crate::error::Error::UuidInvalidUtf8(e))?;
                Self::from_str(s)
            }

            pub fn to_simple_string(&self) -> String {
                self.0.simple().to_string()
            }

            pub fn to_hyphenated_string(&self) -> String {
                self.0.hyphenated().to_string()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0.to_string())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let id: uuid::Uuid = serde::Deserialize::deserialize(deserializer)?;
                Ok(Self(id))
            }
        }

        impl CheckUuid for $name {
            fn id(&self) -> &uuid::Uuid {
                &self.0
            }

            fn to_string(&self) -> String {
                format!("{}({})", stringify!($name), self.0.to_string())
            }
        }
    };
}

implement_uuid!(NoteUuid);
implement_uuid!(VirtualPageUuid);
implement_uuid!(VirtualDocUuid);
implement_uuid!(StrokeUuid);
implement_uuid!(PageUuid);
implement_uuid!(PageModelUuid);
implement_uuid!(PenUuid);
implement_uuid!(ShapeGroupUuid);
implement_uuid!(PointsUuid);
implement_uuid!(ResourceUuid);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PenId {
    Uuid(PenUuid),
    Id(u32),
}

impl PenId {
    pub fn from_uuid(uuid: PenUuid) -> Self {
        Self::Uuid(uuid)
    }

    pub fn from_id(id: u32) -> Self {
        Self::Id(id)
    }
}

impl std::fmt::Display for PenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uuid(uuid) => write!(f, "{}", uuid),
            Self::Id(id) => write!(f, "PenId({})", id),
        }
    }
}

impl<'de> serde::Deserialize<'de> for PenId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id: String = serde::Deserialize::deserialize(deserializer)?;
        let trimmed = id.trim();

        if let Ok(uuid) = PenUuid::from_str(trimmed) {
            return Ok(Self::from_uuid(uuid));
        }

        Ok(Self::from_id(
            trimmed.parse().map_err(serde::de::Error::custom)?,
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerId(u32);

impl LayerId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for LayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LayerId({})", self.0)
    }
}

impl<'de> serde::Deserialize<'de> for LayerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id: u32 = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self(id))
    }
}

#[cfg(test)]
mod tests {
    use super::{NoteUuid, PenId, StrokeUuid};

    #[test]
    fn uuid_from_str_accepts_whitespace_padding() {
        let id = NoteUuid::from_str("  7a960ca753b0420ea2d5b88d57f7bf62  ").unwrap();
        assert_eq!(id.to_simple_string(), "7a960ca753b0420ea2d5b88d57f7bf62");
    }

    #[test]
    fn uuid_from_str_accepts_hyphenated_uuid() {
        let id = NoteUuid::from_str("7a960ca7-53b0-420e-a2d5-b88d57f7bf62").unwrap();
        assert_eq!(
            id.to_hyphenated_string(),
            "7a960ca7-53b0-420e-a2d5-b88d57f7bf62"
        );
    }

    #[test]
    fn uuid_from_byte_str_accepts_null_and_whitespace_padding() {
        let raw = b"92c1ab73-4ec1-4f70-907a-dc11dcb0806d\0\0  ";
        let id = StrokeUuid::from_byte_str(raw).unwrap();
        assert_eq!(
            id.to_hyphenated_string(),
            "92c1ab73-4ec1-4f70-907a-dc11dcb0806d"
        );
    }

    #[test]
    fn pen_id_deserializes_uuid_forms_and_numeric_ids() {
        let simple: PenId = serde_json::from_str("\"abb5f970105848349b5dfa80d7b0fa5b\"").unwrap();
        assert!(matches!(simple, PenId::Uuid(_)));

        let hyphenated: PenId =
            serde_json::from_str("\" abb5f970-1058-4834-9b5d-fa80d7b0fa5b \"").unwrap();
        assert!(matches!(hyphenated, PenId::Uuid(_)));

        let numeric: PenId = serde_json::from_str("\" 1 \"").unwrap();
        assert_eq!(numeric, PenId::Id(1));
    }
}
