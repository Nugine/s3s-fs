use s3s::S3Error;
use s3s::S3ErrorCode;
use s3s::StdError;

use std::fmt;
use std::panic::Location;

use tracing::Level;
use tracing::enabled;
use tracing::error;

#[derive(Debug)]
pub struct Error {
    source: StdError,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    #[must_use]
    #[track_caller]
    pub fn new(source: StdError) -> Self {
        if enabled!(Level::DEBUG) {
            let caller = Location::caller();
            error!(%source, %caller);
        }
        Self { source }
    }

    #[must_use]
    #[track_caller]
    pub fn from_string(s: impl Into<String>) -> Self {
        Self::new(s.into().into())
    }
}

impl<E> From<E> for Error
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[track_caller]
    fn from(source: E) -> Self {
        Self::new(Box::new(source))
    }
}

impl From<Error> for S3Error {
    fn from(e: Error) -> Self {
        S3Error::with_source(S3ErrorCode::InternalError, e.source)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}
