//! Explicit image-build inputs. Values are argv data, never Dockerfile syntax.
use std::{collections::BTreeMap, fmt, str::FromStr};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildArg(String);

impl FromStr for BuildArg {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (name, text) = value
            .split_once('=')
            .ok_or("build arguments require NAME=VALUE; environment lookup is not implicit")?;
        if name.is_empty()
            || name.len() > 128
            || !name
                .bytes()
                .enumerate()
                .all(|(i, c)| c == b'_' || c.is_ascii_alphabetic() || (i > 0 && c.is_ascii_digit()))
        {
            return Err("build argument names must be ASCII identifiers (letters, digits and underscores; no leading digit)".into());
        }
        let upper = name.to_ascii_uppercase();
        if upper.starts_with("ZELLIJ_") || upper == "FEATURES" {
            return Err("Zellij build settings are reserved; use --features, --base-path, --site-url or --no-check instead".into());
        }
        if matches!(
            upper.as_str(),
            "BUILDPLATFORM"
                | "BUILDOS"
                | "BUILDARCH"
                | "BUILDVARIANT"
                | "TARGETPLATFORM"
                | "TARGETOS"
                | "TARGETARCH"
                | "TARGETVARIANT"
        ) {
            return Err("automatic platform arguments are reserved; use --platform instead".into());
        }
        if text.chars().any(char::is_control) {
            return Err("build argument values cannot contain control characters".into());
        }
        Ok(Self(value.to_owned()))
    }
}

impl BuildArg {
    pub fn name(&self) -> &str {
        self.0.split_once('=').unwrap().0
    }

    pub fn assignment(&self) -> &str {
        &self.0
    }

    /// Proxies and BuildKit controls already have builder-defined semantics.
    /// In particular, declaring proxy ARGs would opt them into image history.
    fn predefined(&self) -> bool {
        let name = self.name().to_ascii_uppercase();
        name.starts_with("BUILDKIT_")
            || matches!(
                name.as_str(),
                "HTTP_PROXY" | "HTTPS_PROXY" | "FTP_PROXY" | "NO_PROXY" | "ALL_PROXY"
            )
    }
}

impl fmt::Display for BuildArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.assignment())
    }
}

/// Repeated names use their last explicit value, with deterministic ordering.
pub fn normalize(args: &[BuildArg]) -> Vec<BuildArg> {
    args.iter()
        .map(|arg| (arg.name(), arg.clone()))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect()
}

pub fn declarations(args: &[BuildArg]) -> String {
    if args.is_empty() {
        return String::new();
    }
    let mut body = "# Values are supplied with --build-arg NAME=VALUE when building.\n".to_owned();
    for arg in normalize(args).iter().filter(|arg| !arg.predefined()) {
        body.push_str(&format!("ARG {}\n", arg.name()));
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_values_keep_literal_data_and_last_duplicate() {
        let value = "日本語 '$(touch marker)`echo x` = ";
        let args: Vec<BuildArg> = [
            "RELEASE=old",
            "EMPTY=",
            &format!("RELEASE={value}"),
            "HTTP_PROXY=http://proxy.example",
            "BUILDKIT_INLINE_CACHE=1",
        ]
        .iter()
        .map(|value| value.parse().unwrap())
        .collect();
        let args = normalize(&args);
        assert_eq!(args.len(), 4);
        assert_eq!(
            args.last().unwrap().assignment(),
            format!("RELEASE={value}")
        );
        let body = declarations(&args);
        assert!(body.contains("ARG RELEASE\n") && body.contains("ARG EMPTY\n"));
        assert!(
            !body.contains(value)
                && !body.contains("ARG HTTP_PROXY")
                && !body.contains("ARG BUILDKIT_")
        );
        for bad in [
            "MISSING",
            "=value",
            "1NAME=x",
            "A-B=x",
            "A\nRUN=x",
            "NAME=x\ny",
            "NAME=x\0",
            "ZELLIJ_OUTPUT=x",
            "zellij_build_overrides=x",
            "FEATURES=all",
            "TARGETARCH=arm64",
        ] {
            assert!(bad.parse::<BuildArg>().is_err(), "{bad:?}");
        }
    }
}
