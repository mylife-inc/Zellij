//! Shared Sphinx path matching for toctrees and source selection.
use super::invalid;
use camino::Utf8Path;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use zellij_core::Result;

// Sphinx's patfilter semantics, not filesystem globbing: * and ? cannot
// cross '/', ** can, and **/ still requires a slash. Toctrees match names
// without .rst; exclusions match complete source-relative filenames.
pub(super) fn matcher(pattern: &str) -> Result<Regex> {
    if pattern.len() > 4096 {
        return Err(invalid("glob exceeds 4096 bytes"));
    }
    let chars: Vec<_> = pattern.chars().collect();
    let mut out = String::from(r"\A(?:");
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        i += 1;
        match c {
            '*' if chars.get(i) == Some(&'*') => {
                i += 1;
                out.push_str(".*");
            }
            '*' => out.push_str("[^/]*"),
            '?' => out.push_str("[^/]"),
            '[' => {
                let start = i;
                let negative = chars.get(i) == Some(&'!');
                if negative {
                    i += 1;
                }
                if chars.get(i) == Some(&']') {
                    i += 1;
                }
                while i < chars.len() && chars[i] != ']' {
                    i += 1;
                }
                if i == chars.len() {
                    out.push_str(r"\[");
                    i = start;
                    continue;
                }
                out.push('[');
                if negative {
                    out.push_str("^/");
                }
                let contents = &chars[start + usize::from(negative)..i];
                for (offset, c) in contents.iter().enumerate() {
                    // Preserve ranges, but escape Rust regex class operators
                    // and literal leading/trailing hyphens.
                    if *c == '-' && offset > 0 && offset + 1 < contents.len() {
                        out.push('-');
                    } else {
                        out.push_str(&regex::escape(&c.to_string()));
                    }
                }
                out.push(']');
                i += 1;
            }
            _ => out.push_str(&regex::escape(&c.to_string())),
        }
    }
    out.push_str(r")\z");
    Regex::new(&out).map_err(|_| invalid(format!("invalid glob {pattern:?}")))
}

pub(super) fn validate(setting: &str, patterns: &[String]) -> Result<()> {
    if patterns.len() > 1024 || patterns.iter().map(String::len).sum::<usize>() > 128 * 1024 {
        return Err(invalid(format!(
            "{setting} exceeds 1024 patterns or 128 KiB"
        )));
    }
    for pattern in patterns {
        // Patterns select source-relative names; they are never filesystem
        // paths to canonicalize. Excluding Git metadata is legitimate.
        if pattern.is_empty()
            || pattern.len() > 4096
            || pattern.contains(['\\', ':'])
            || pattern.chars().any(char::is_control)
            || pattern.split('/').any(|p| matches!(p, "" | "." | ".."))
        {
            return Err(invalid(format!(
                "{setting} requires source-relative slash-separated patterns without parent segments (at most 4096 bytes): {pattern:?}"
            )));
        }
    }
    Ok(())
}

pub(super) struct SourcePatterns(Vec<Regex>);
impl SourcePatterns {
    pub(super) fn new(setting: &str, patterns: &[String]) -> Result<Self> {
        validate(setting, patterns)?;
        patterns
            .iter()
            .map(|p| matcher(p))
            .collect::<Result<Vec<_>>>()
            .map(Self)
    }

    pub(super) fn includes(&self, path: &Utf8Path) -> bool {
        let name: String = path.as_str().nfc().collect();
        self.0.iter().any(|pattern| pattern.is_match(&name))
    }

    pub(super) fn excludes(&self, path: &Utf8Path) -> bool {
        if self.0.is_empty() {
            return false;
        }
        // Sphinx's source discovery normalizes filenames to NFC, then matches
        // files and directories. Checking ancestors retains directory pruning
        // semantics while the inventory still checks excluded trees for links.
        let name: String = path.as_str().nfc().collect();
        Utf8Path::new(&name)
            .ancestors()
            .take_while(|p| !p.as_str().is_empty())
            .any(|p| self.0.iter().any(|pattern| pattern.is_match(p.as_str())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sphinx_patterns_are_case_sensitive_and_separator_aware() {
        for (pattern, yes, no) in [
            ("guide/*", "guide/intro", "guide/deep/intro"),
            ("guide/**", "guide/deep/intro", "Guide/intro"),
            ("**/intro", "guide/intro", "intro"),
            ("p?", "pé", "p/"),
            ("[a-c]", "b", "d"),
            ("[!a-c]", "z", "/"),
            ("[^a]", "^", "b"),
            ("[]a]", "]", "b"),
            ("[-a]", "-", "b"),
            ("[a-]", "-", "b"),
            ("[a&&b]", "&", "c"),
            ("[open", "[open", "o"),
            ("a.b", "a.b", "axb"),
        ] {
            let regex = matcher(pattern).unwrap();
            assert!(regex.is_match(yes), "{pattern}: {yes}");
            assert!(!regex.is_match(no), "{pattern}: {no}");
        }
        assert!(matcher("[z-a]").is_err());
        assert!(matcher(&"*".repeat(4097)).is_err());
    }

    #[test]
    fn exclusions_match_source_names_and_directory_ancestors() {
        let exclusions = SourcePatterns::new(
            "exclude_patterns",
            &[
                "draft?.rst".into(),
                "**/private*".into(),
                "Café.rst".into(),
                "data/[!a-c]*".into(),
            ],
        )
        .unwrap();
        for path in [
            "draft1.rst",
            "guide/private/deep/page.rst",
            "guide/private.rst",
            "Cafe\u{301}.rst",
            "data/z.txt",
        ] {
            assert!(exclusions.excludes(Utf8Path::new(path)), "{path}");
        }
        for path in [
            "draft12.rst",
            "guide/draft1.rst",
            "private/page.rst",
            "Draft1.rst",
            "data/b.txt",
        ] {
            assert!(!exclusions.excludes(Utf8Path::new(path)), "{path}");
        }
        // Unlike the HTML builder's asset Matcher, source discovery does not
        // expand a leading **/ to match the source root as well.
        let roots = SourcePatterns::new("exclude_patterns", &["**/draft.rst".into()]).unwrap();
        assert!(!roots.excludes(Utf8Path::new("draft.rst")));
        assert!(roots.excludes(Utf8Path::new("guide/draft.rst")));
    }

    #[test]
    fn exclusion_limits_and_invalid_patterns_are_rejected() {
        assert!(SourcePatterns::new("exclude_patterns", &vec!["*".into(); 1025]).is_err());
        assert!(SourcePatterns::new("include_patterns", &vec!["a".repeat(4096); 33]).is_err());
        for pattern in [
            "",
            "../*",
            "/private",
            "a//b",
            "C:foo",
            "a\\b",
            "line\nbreak",
            "a/./b",
            "[z-a]",
        ] {
            for setting in ["include_patterns", "exclude_patterns"] {
                assert!(
                    SourcePatterns::new(setting, &[pattern.into()]).is_err(),
                    "{setting}: {pattern}"
                );
            }
        }
    }

    #[test]
    fn inclusions_match_only_complete_filenames_and_preserve_empty_selection() {
        let included = SourcePatterns::new(
            "include_patterns",
            &[
                "directory".into(),
                "Café.rst".into(),
                "guide/*.rst".into(),
                "literal%20.rst".into(),
            ],
        )
        .unwrap();
        assert!(included.includes(Utf8Path::new("Cafe\u{301}.rst")));
        assert!(included.includes(Utf8Path::new("guide/page.rst")));
        assert!(included.includes(Utf8Path::new("literal%20.rst")));
        assert!(!included.includes(Utf8Path::new("directory/page.rst")));
        assert!(!included.includes(Utf8Path::new("guide/deep/page.rst")));
        assert!(!included.includes(Utf8Path::new("café.rst")));
        assert!(
            SourcePatterns::new("include_patterns", &["**".into()])
                .unwrap()
                .includes(Utf8Path::new("deep/page.rst"))
        );
        assert!(
            !SourcePatterns::new("include_patterns", &[])
                .unwrap()
                .includes(Utf8Path::new("index.rst"))
        );
    }
}
