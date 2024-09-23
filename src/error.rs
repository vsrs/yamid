use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    InvalidUuid(#[from] uuid::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
