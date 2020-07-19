use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("{0}")]
    ImageError(#[from] image::error::ImageError),
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

pub fn error(message: impl Into<String>) -> CommandError {
    return CommandError::Other(message.into());
}
