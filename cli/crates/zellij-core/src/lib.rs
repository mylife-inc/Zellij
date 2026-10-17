//! What the rest of the CLI agrees on: where a project is, what a content
//! folder is made of, and how failures are described.
//!
//! Deliberately knows nothing about Docker, about any documentation tool, and
//! about the command line. Everything here should still make sense if the tool
//! grew a second front end.

pub mod error;
pub mod files;
pub mod output;
pub mod project;
pub mod schema;
pub mod yaml;

pub use error::{Error, Result};
pub use project::Project;
