use camino::Utf8PathBuf;

/// Everything the CLI can fail at, in the terms a user would describe it.
///
/// Typed rather than stringly, because the exit code is derived from the
/// variant: a pipeline distinguishing "your content is wrong" from "Docker is
/// not running" is the difference between a useful failure and a retry loop.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no Zellij content folder found at {path}")]
    NoContentFolder { path: Utf8PathBuf },

    #[error("{path} already exists; pass --force to write into it anyway")]
    NotEmpty { path: Utf8PathBuf },

    #[error("could not tell what documentation tool {path} was written for")]
    UnknownFormat { path: Utf8PathBuf },

    #[error("no container runtime available: {reason}")]
    NoEngine { reason: String },

    #[error("the content folder is not valid")]
    ContentInvalid,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Error {
    /// The process exit code for this failure, as specified in §4.3.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::NotEmpty { .. } | Error::UnknownFormat { .. } => 2,
            Error::ContentInvalid => 3,
            Error::NoEngine { .. } => 4,
            _ => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
