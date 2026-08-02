//! Turning somebody else's documentation into a Zellij content folder.
//!
//! Every format goes through the same pipeline — **detect, parse into an
//! intermediate, emit, validate, report** — and differs only in the parse. That
//! is what keeps the emitting written once instead of once per tool, and it is
//! where "MkDocs nav" and "VitePress sidebar" become the same thing.
//!
//! A converter is registered by existing rather than by being listed anywhere,
//! so adding one touches no file that is not about that format.

pub mod emit;
pub mod mdx;
pub mod mkdocs;
pub mod model;

use camino::{Utf8Path, Utf8PathBuf};

/// How strongly a source looks like a given format.
///
/// A converter that found its own config file is certain. One inferring from
/// the shape of a folder is not, and the difference decides whether the CLI
/// proceeds or asks — guessing between two plausible formats produces a
/// conversion that is wrong in ways nobody notices until much later.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Confidence(pub f32);

impl Confidence {
    pub const NONE: Self = Self(0.0);
    /// Found the format's own configuration file.
    pub const CERTAIN: Self = Self(1.0);
    /// Inferred from folder layout alone.
    pub const LIKELY: Self = Self(0.6);
}

/// A file the conversion will write.
#[derive(Debug, Clone)]
pub struct OutputFile {
    /// Relative to the destination content folder.
    pub path: Utf8PathBuf,
    pub body: String,
}

/// Something in the source that has no Zellij equivalent.
///
/// Reported rather than discarded. A converter that silently drops a page is
/// worse than one that refuses: the author has no way to know what is missing
/// until a reader tells them.
#[derive(Debug, Clone)]
pub struct Unmapped {
    pub source: Utf8PathBuf,
    pub reason: String,
}

/// The whole result of a conversion, held in memory.
///
/// Nothing is written until every part has succeeded, so a failure halfway
/// through leaves no half-built folder for someone to mistake for a finished
/// one.
#[derive(Debug, Default)]
pub struct Conversion {
    pub files: Vec<OutputFile>,
    pub assets: Vec<Utf8PathBuf>,
    pub unmapped: Vec<Unmapped>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ConvertOptions {
    pub copy_assets: bool,
}

pub trait Converter: Send + Sync {
    /// Stable identifier, and what `--from` accepts.
    fn id(&self) -> &'static str;

    /// What to call this format when talking to a person.
    fn label(&self) -> &'static str;

    fn detect(&self, source: &Utf8Path) -> Confidence;

    fn convert(
        &self,
        source: &Utf8Path,
        options: &ConvertOptions,
    ) -> zellij_core::Result<Conversion>;
}

inventory::collect!(&'static dyn Converter);

/// Every registered converter.
pub fn all() -> impl Iterator<Item = &'static dyn Converter> {
    inventory::iter::<&'static dyn Converter>().copied()
}

/// The converters that recognise this source, most confident first.
pub fn detect(source: &Utf8Path) -> Vec<(&'static dyn Converter, Confidence)> {
    let mut found: Vec<_> = all()
        .map(|converter| (converter, converter.detect(source)))
        .filter(|(_, confidence)| *confidence > Confidence::NONE)
        .collect();

    found.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    found
}
