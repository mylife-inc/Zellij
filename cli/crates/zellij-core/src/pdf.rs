//! Opt-in print settings shared by build, preview and delivery.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Settings {
    pub enabled: bool,
    pub paper: Paper,
    pub font_size: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            paper: Paper::A5,
            font_size: 11,
            style: None,
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
        if let Some(style) = &self.style {
            style.validate()?;
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

/// Bounded print-only overrides. No authored TeX, paths or system font names.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_font: Option<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_font: Option<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_style: Option<crate::book::Edition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_spacing: Option<LineSpacing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margins: Option<Margins>,
}
impl Style {
    fn validate(&self) -> Result<()> {
        if let Some(color) = &self.accent {
            if color.len() != 7
                || !color.starts_with('#')
                || !color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
            {
                return Err(Error::InvalidArgument(
                    "pdf.style.accent must be a quoted #RRGGBB color".into(),
                ));
            }
        }
        if let Some(margins) = &self.margins {
            for (name, value) in [
                ("left", margins.left),
                ("right", margins.right),
                ("top", margins.top),
                ("bottom", margins.bottom),
            ] {
                if !(15..=40).contains(&value) {
                    return Err(Error::InvalidArgument(format!(
                        "pdf.style.margins.{name} must be an integer from 15 to 40 (millimeters)"
                    )));
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Font {
    Serif,
    Sans,
}
impl Font {
    pub fn latex(self) -> &'static str {
        match self {
            Self::Serif => "\\rmfamily",
            Self::Sans => "\\sffamily",
        }
    }
    pub fn default_family(self) -> &'static str {
        match self {
            Self::Serif => "\\rmdefault",
            Self::Sans => "\\sfdefault",
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LineSpacing {
    Normal,
    Relaxed,
    Spacious,
}
impl LineSpacing {
    pub fn latex(self) -> &'static str {
        match self {
            Self::Normal => "1",
            Self::Relaxed => "1.15",
            Self::Spacious => "1.3",
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Margins {
    pub left: u8,
    pub right: u8,
    pub top: u8,
    pub bottom: u8,
}
impl Default for Margins {
    fn default() -> Self {
        Self {
            left: 20,
            right: 18,
            top: 20,
            bottom: 22,
        }
    }
}
