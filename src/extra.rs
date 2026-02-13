use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraMetadata {
    pub flag: u32,
    pub app_build: u32,
    pub key: String,
    pub value: String,
}

impl ExtraMetadata {
    pub fn read(mut reader: impl std::io::Read) -> Result<Self> {
        let extra = protobuf::Extra::read(&mut reader)?;
        Ok(Self {
            flag: extra.flag,
            app_build: extra.app_build,
            key: extra.key,
            value: extra.value,
        })
    }
}

mod protobuf {
    use prost::Message;

    use crate::error::Result;

    #[derive(Clone, PartialEq, Message)]
    pub struct Extra {
        #[prost(uint32, tag = "1")]
        pub flag: u32,
        #[prost(uint32, tag = "2")]
        pub app_build: u32,
        #[prost(string, tag = "3")]
        pub key: String,
        #[prost(string, tag = "4")]
        pub value: String,
    }

    impl Extra {
        pub fn read(mut reader: impl std::io::Read) -> Result<Self> {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            Ok(Extra::decode(&buf[..])?)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::ExtraMetadata;

    #[test]
    fn decodes_extra_metadata() {
        let bytes = [
            0x08, 0x01, 0x10, 0xDA, 0xCE, 0x02, 0x1A, 0x0A, 0x73, 0x68, 0x61, 0x72, 0x65, 0x5F,
            0x75, 0x73, 0x65, 0x72, 0x22, 0x20, 0x37, 0x61, 0x39, 0x36, 0x30, 0x63, 0x61, 0x37,
            0x35, 0x33, 0x62, 0x30, 0x34, 0x32, 0x30, 0x65, 0x61, 0x32, 0x64, 0x35, 0x62, 0x38,
            0x38, 0x64, 0x35, 0x37, 0x66, 0x37, 0x62, 0x66, 0x36, 0x32,
        ];
        let extra = ExtraMetadata::read(Cursor::new(bytes)).unwrap();

        assert_eq!(extra.flag, 1);
        assert_eq!(extra.app_build, 42842);
        assert_eq!(extra.key, "share_user");
        assert_eq!(extra.value, "7a960ca753b0420ea2d5b88d57f7bf62");
    }
}
