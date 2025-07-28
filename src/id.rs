use std::{
    cell::LazyCell,
    collections::{HashMap, HashSet},
    sync::{LazyLock, Mutex, OnceLock},
};

trait CheckUuid {
    fn id(&self) -> &uuid::Uuid;
    fn to_string(&self) -> String;
}
static KNOWN_UUIDS: OnceLock<Mutex<HashMap<uuid::Uuid, Vec<Box<dyn CheckUuid + Send + Sync>>>>> =
    OnceLock::new();

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
                let id = Self(uuid::Uuid::parse_str(s).inspect_err(|e| {
                    log::error!("Failed to parse UUID from byte string: {}", s);
                })?);
                check_uuid(id.clone());
                Ok(id)
            }

            pub fn from_byte_str(s: &[u8]) -> crate::error::Result<Self> {
                let s =
                    std::str::from_utf8(s).map_err(|e| crate::error::Error::UuidInvalidUtf8(e))?;
                let id = Self(uuid::Uuid::parse_str(s).inspect_err(|e| {
                    log::error!("Failed to parse UUID from byte string: {}", s);
                })?);
                check_uuid(id.clone());
                Ok(id)
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
        if id.len() == 32 {
            // UUID format
            let uuid = PenUuid::from_str(&id).map_err(serde::de::Error::custom)?;
            return Ok(Self::from_uuid(uuid));
        }
        Ok(Self::from_id(id.parse().map_err(serde::de::Error::custom)?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerId(u32);

impl LayerId {
    pub fn new(id: u32) -> Self {
        Self(id)
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
