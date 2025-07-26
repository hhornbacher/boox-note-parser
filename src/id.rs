macro_rules! implement_uuid {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $name(uuid::Uuid);

        impl $name {
            pub fn new(id: uuid::Uuid) -> Self {
                Self(id)
            }

            pub fn from_str(s: &str) -> crate::error::Result<Self> {
                Ok(Self(uuid::Uuid::parse_str(s)?))
            }

            pub fn from_byte_str(s: &[u8]) -> crate::error::Result<Self> {
                let s =
                    std::str::from_utf8(s).map_err(|e| crate::error::Error::UuidInvalidUtf8(e))?;
                Ok(Self(uuid::Uuid::parse_str(s)?))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0.to_string())
            }
        }
    };
}

implement_uuid!(NoteId);
implement_uuid!(VirtualPageId);
implement_uuid!(VirtualDocId);
implement_uuid!(ShapeId);
implement_uuid!(StrokeId);
implement_uuid!(PageModelId);
