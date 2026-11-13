//! Opt-in print settings shared by build, preview and delivery.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Settings {
    pub enabled: bool,
    pub paper: Paper,
    pub font_size: u8,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            paper: Paper::A5,
            font_size: 11,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        if !(10..=12).contains(&self.font_size) {
            return Err(Error::InvalidArgument(
                "pdf.font-size must be 10, 11 or 12 (points)".into(),
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Paper {
    #[default]
    A5,
    A4,
    Letter,
}
impl Paper {
    pub fn latex(self) -> &'static str {
        match self {
            Self::A5 => "a5paper",
            Self::A4 => "a4paper",
            Self::Letter => "letterpaper",
        }
    }
}
