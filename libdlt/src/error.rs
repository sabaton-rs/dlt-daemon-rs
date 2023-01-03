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
    ConfigFileError(String)
}