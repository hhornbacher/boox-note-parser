pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid container format")]
    InvalidContainerFormat,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {error} in JSON string: {json_string}")]
    Json{
        error: serde_json::Error,
        json_string: String,
    },
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Protobuf decode error: {0}")]
    ProtobufDecode(#[from] prost::DecodeError),
    #[error("UUID parse error: {0}")]
    UuidParse(#[from] uuid::Error),
    #[error("UUID invalid UTF8: {0}")]
    UuidInvalidUtf8(std::str::Utf8Error),
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(u64),
    #[error("Invalid timestamp format: {0}")]
    InvalidTimestampFormat(String),
}