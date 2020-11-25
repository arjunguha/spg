use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("{0}")]
    ImageError(#[from] image::error::ImageError),
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    Exif(#[from] exif::Error),
    #[error("{0}")]
    Other(String),
    #[error("{0}\n{1}")]
    Trace(String, Box<CommandError>),
}

pub fn trace<E>(message: impl Into<String>) -> Box<dyn FnOnce(E) -> CommandError> where
  E : Into<CommandError> {
    let message = message.into();
    return Box::new(move |err| {
        return CommandError::Trace(message, Box::new(err.into()));
    });
}

pub fn error(message: impl Into<String>) -> CommandError {
    return CommandError::Other(message.into());
}
