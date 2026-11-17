//! Fixed locale/font choices; no authored package names or font paths enter TeX.
use crate::pdf::invalid;
use zellij_core::{
    Result,
    book::Manifest,
    pdf::{Font, Settings},
};

pub(crate) struct Language {
    pub name: &'static str,
    pub rtl: bool,
}
impl Language {
    pub fn resolve(book: &Manifest, settings: &Settings) -> Result<Self> {
        let primary = book
            .language
            .split('-')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let name = match primary.as_str() {
            "en" => "english",
            "fr" => "french",
            "de" => "ngerman",
            "es" => "spanish",
            "ar" => "arabic",
            _ => {
                return Err(invalid(
                    "print languages currently supported: en, fr, de, es (LTR) and ar (RTL); web-only books may use other languages",
                ));
            }
        };
        let rtl = name == "arabic";
        if (book.direction.as_str() == "rtl") != rtl {
            return Err(invalid(format!(
                "book.language {} requires book.direction: {} for print",
                book.language,
                if rtl { "rtl" } else { "ltr" }
            )));
        }
        for script in book
            .language
            .split('-')
            .skip(1)
            .filter(|s| s.len() == 4 && s.bytes().all(|b| b.is_ascii_alphabetic()))
        {
            if !script.eq_ignore_ascii_case(if rtl { "Arab" } else { "Latn" }) {
                return Err(invalid(
                    "the selected print language does not support this script subtag",
                ));
            }
        }
        if rtl
            && settings.style.as_ref().is_some_and(|s| {
                s.body_font == Some(Font::Libertinus) || s.title_font == Some(Font::Libertinus)
            })
        {
            return Err(invalid(
                "Arabic print uses Amiri; Libertinus does not contain Arabic glyphs",
            ));
        }
        Ok(Self { name, rtl })
    }

    pub fn preamble(&self) -> String {
        if self.name == "english" {
            return String::new();
        }
        if self.rtl {
            // The pinned image's Babel supports LuaTeX bidi. Amiri is shipped
            // with the source, never resolved from the host's installed fonts.
            "\\usepackage[bidi=basic,layout=counters tabular,shorthands=off]{babel}\n\\babelprovide[import]{english}\n\\babelprovide[import,main]{arabic}\n".into()
        } else {
            format!(
                "\\usepackage[shorthands=off]{{babel}}\n\\babelprovide[import,main]{{{}}}\n",
                self.name
            )
        }
    }
}

pub(crate) const AMIRI: &[(&str, &[u8])] = &[
    (
        "Amiri-Regular.ttf",
        include_bytes!("../assets/print-fonts/amiri/Amiri-Regular.ttf"),
    ),
    (
        "Amiri-Bold.ttf",
        include_bytes!("../assets/print-fonts/amiri/Amiri-Bold.ttf"),
    ),
    (
        "Amiri-Italic.ttf",
        include_bytes!("../assets/print-fonts/amiri/Amiri-Italic.ttf"),
    ),
    (
        "Amiri-BoldItalic.ttf",
        include_bytes!("../assets/print-fonts/amiri/Amiri-BoldItalic.ttf"),
    ),
    (
        "OFL.txt",
        include_bytes!("../assets/print-fonts/amiri/OFL.txt"),
    ),
];
pub(crate) const LIBERTINUS: &[(&str, &[u8])] = &[
    (
        "LibertinusSerif-Regular.otf",
        include_bytes!("../assets/print-fonts/libertinus/LibertinusSerif-Regular.otf"),
    ),
    (
        "LibertinusSerif-Bold.otf",
        include_bytes!("../assets/print-fonts/libertinus/LibertinusSerif-Bold.otf"),
    ),
    (
        "LibertinusSerif-Italic.otf",
        include_bytes!("../assets/print-fonts/libertinus/LibertinusSerif-Italic.otf"),
    ),
    (
        "LibertinusSerif-BoldItalic.otf",
        include_bytes!("../assets/print-fonts/libertinus/LibertinusSerif-BoldItalic.otf"),
    ),
    (
        "OFL.txt",
        include_bytes!("../assets/print-fonts/libertinus/OFL.txt"),
    ),
];

pub(crate) const AMIRI_OPTIONS: &str = "Path=fonts/amiri/,Renderer=HarfBuzz,UprightFont=Amiri-Regular.ttf,BoldFont=Amiri-Bold.ttf,ItalicFont=Amiri-Italic.ttf,BoldItalicFont=Amiri-BoldItalic.ttf";
pub(crate) const LIBERTINUS_OPTIONS: &str = "Path=fonts/libertinus/,UprightFont=LibertinusSerif-Regular.otf,BoldFont=LibertinusSerif-Bold.otf,ItalicFont=LibertinusSerif-Italic.otf,BoldItalicFont=LibertinusSerif-BoldItalic.otf";
