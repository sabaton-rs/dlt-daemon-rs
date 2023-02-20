use thiserror::Error;

#[derive(Error, Debug)]
pub enum DltUserError {
    #[error("unknown error")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum DltError {
    #[error("Config file not found")]
    IoError(#[from] std::io::Error),
    #[error("Config file error")]
    ConfigFileError(String),
    #[error("file size error")]
    FileSizeError,
    #[error("Logging diabled")]
    DltReturnLoggingDisabled,
    #[error("User Buffer Full")]
    DltReturnUserBufferFull,
    #[error("Wrong Parameter")]
    DltReturnWrongParameter,
    #[error("Buffer Full")]
    DltReturnBufferFull,
    #[error("Pipe Full")]
    DltReturnPipeFull,
    #[error("Pipe error")]
    DltReturnPipeError,
    #[error("error")]
    DltReturnError,
    #[error("ok")]
    DltReturnOk,
    #[error("true")]
    DltReturnTrue,
}
