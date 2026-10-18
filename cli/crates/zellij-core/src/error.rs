use camino::Utf8PathBuf;

/// Everything the CLI can fail at, in the terms a user would describe it.
///
/// Typed rather than stringly, because the exit code is derived from the
/// variant: a pipeline distinguishing "your content is wrong" from "Docker is
/// not running" is the difference between a useful failure and a retry loop.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    InvalidArgument(String),

    #[error("no Zellij content folder found at {path}")]
    NoContentFolder { path: Utf8PathBuf },

    #[error("{path} already exists; pass --force to write into it anyway")]
    NotEmpty { path: Utf8PathBuf },

    #[error("refusing unsafe path {path}: {reason}")]
    UnsafePath { path: Utf8PathBuf, reason: String },

    #[error("{path} already exists; choose a different destination")]
    FileExists { path: Utf8PathBuf },

    #[error("could not tell what documentation tool {path} was written for")]
    UnknownFormat { path: Utf8PathBuf },

    #[error("no container runtime available: {reason}")]
    NoEngine { reason: String },

    #[error("{path} is not shared with the container runtime")]
    MountEmpty { path: Utf8PathBuf },

    #[error("the content folder is not valid")]
    ContentInvalid,

    /// The content validated and the build still failed.
    ///
    /// Kept apart from `ContentInvalid` because the two send you to different
    /// places. The build runs `check` first; everything after it is npm, Next
    /// and the network, and reporting those as invalid content is a message
    /// about the wrong thing — the output above the error is the real report.
    #[error("the site build failed — see the output above")]
    BuildFailed,

    #[error(
        "container image {operation} failed — check the runtime output above, registry address and credentials"
    )]
    ImageOperationFailed { operation: &'static str },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Error {
    /// The process exit code for this failure, as specified in §4.3.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::InvalidArgument(_)
            | Error::NotEmpty { .. }
            | Error::UnknownFormat { .. }
            | Error::UnsafePath { .. }
            | Error::FileExists { .. } => 2,
            Error::ContentInvalid | Error::BuildFailed | Error::ImageOperationFailed { .. } => 3,
            Error::NoEngine { .. } | Error::MountEmpty { .. } => 4,
            _ => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
