//! Saved CI intent. No credentials, executable snippets or unpinned tool versions.
use crate::{Error, Result};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trigger {
    PullRequest,
    Push,
    Manual,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum System {
    #[default]
    Github,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Settings {
    pub system: System,
    #[serde(deserialize_with = "strict_string")]
    pub branch: String,
    pub workflow: String,
    pub triggers: Vec<Trigger>,
    #[serde(deserialize_with = "strict_strings")]
    pub tags: Vec<String>,
    #[serde(deserialize_with = "strict_strings")]
    pub schedules: Vec<String>,
    #[serde(deserialize_with = "strict_version")]
    pub zel_version: Option<String>,
    pub production: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            system: System::Github,
            branch: "main".into(),
            workflow: "docs.yml".into(),
            triggers: vec![Trigger::PullRequest, Trigger::Push, Trigger::Manual],
            tags: vec![],
            schedules: vec![],
            zel_version: None,
            production: false,
        }
    }
}
fn strict_string<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    match serde_yaml_ng::Value::deserialize(d)? {
        serde_yaml_ng::Value::String(value) => Ok(value),
        _ => Err(serde::de::Error::custom("expected a string")),
    }
}
fn strict_strings<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    let values = Vec::<serde_yaml_ng::Value>::deserialize(d)?;
    values
        .into_iter()
        .map(|value| match value {
            serde_yaml_ng::Value::String(value) => Ok(value),
            _ => Err(serde::de::Error::custom("expected a list of strings")),
        })
        .collect()
}
fn strict_version<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    strict_string(d).map(Some)
}
fn invalid(message: &str) -> Error {
    Error::InvalidArgument(format!("ci: {message}"))
}
impl Settings {
    pub fn validate(&self) -> Result<()> {
        if self.branch.is_empty()
            || self.branch.starts_with(['-', '/'])
            || self.branch.ends_with(['.', '/'])
            || self.branch.contains([']', '[', '*', '?', '!', '+'])
            || self.branch.contains("..")
            || self.branch.contains("//")
            || self
                .branch
                .split('/')
                .any(|s| s.starts_with('.') || s.ends_with(".lock"))
            || !self
                .branch
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_./-".contains(&b))
        {
            return Err(invalid(
                "branch must be a literal branch name using letters, digits, /, _, - or .",
            ));
        }
        if !self.workflow.ends_with(".yml") && !self.workflow.ends_with(".yaml")
            || self.workflow.starts_with('.')
            || !self
                .workflow
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        {
            return Err(invalid(
                "workflow must be a .yml or .yaml filename without directories",
            ));
        }
        if self.triggers.is_empty() && self.tags.is_empty() && self.schedules.is_empty() {
            return Err(invalid(
                "at least one trigger, tag pattern or schedule is required",
            ));
        }
        for (index, trigger) in self.triggers.iter().enumerate() {
            if self.triggers[..index].contains(trigger) {
                return Err(invalid("triggers must not repeat"));
            }
        }
        for tag in &self.tags {
            if tag.is_empty()
                || tag.starts_with('!')
                || !tag
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_./*?-".contains(&b))
            {
                return Err(invalid(
                    "tags must be positive patterns using letters, digits, /, _, -, ., * or ?",
                ));
            }
        }
        for schedule in &self.schedules {
            validate_cron(schedule)?;
        }
        if let Some(version) = &self.zel_version {
            version_pin(version)?;
        }
        Ok(())
    }
}

/// First compatible command/config contract is 0.2.0. This checks syntax and
/// compatibility, not whether a publisher has uploaded the release assets.
pub fn version_pin(value: &str) -> Result<String> {
    let value = value.strip_prefix('v').unwrap_or(value);
    let (base, pre) = value
        .split_once('-')
        .map_or((value, None), |(b, p)| (b, Some(p)));
    let numbers: Option<Vec<u64>> = base
        .split('.')
        .map(|n| {
            (!n.is_empty()
                && (n == "0" || !n.starts_with('0'))
                && n.bytes().all(|b| b.is_ascii_digit()))
            .then(|| n.parse().ok())
            .flatten()
        })
        .collect();
    let Some(numbers) = numbers.filter(|n| n.len() == 3) else {
        return Err(invalid(
            "zel-version must be an exact version such as 0.2.0 or 0.2.0-rc.1",
        ));
    };
    if numbers.as_slice() < [0, 2, 0].as_slice()
        || pre.is_some_and(|p| {
            p.split('.')
                .any(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
        })
    {
        return Err(invalid(
            "zel-version must support the new CLI contract (0.2.0 or later, including release candidates); 0.1.3 is incompatible",
        ));
    }
    Ok(format!("v{value}"))
}

fn validate_cron(value: &str) -> Result<()> {
    let parts: Vec<_> = value.split_whitespace().collect();
    let limits = [(0, 59), (0, 23), (1, 31), (1, 12), (0, 6)];
    if parts.len() != 5
        || parts
            .iter()
            .zip(limits)
            .any(|(part, (min, max))| !cron_field(part, min, max))
    {
        return Err(invalid(
            "schedules must be five numeric UTC cron fields; use *, ranges, lists and /steps",
        ));
    }
    Ok(())
}
fn cron_field(field: &str, min: u32, max: u32) -> bool {
    field.split(',').all(|entry| {
        let (range, step) = entry
            .split_once('/')
            .map_or((entry, None), |(r, s)| (r, Some(s)));
        let number = |n: &str| {
            !n.is_empty()
                && n.bytes().all(|b| b.is_ascii_digit())
                && n.parse::<u32>().is_ok_and(|v| (min..=max).contains(&v))
        };
        if step.is_some_and(|s| {
            s.is_empty()
                || !s.bytes().all(|b| b.is_ascii_digit())
                || !s.parse::<u32>().is_ok_and(|v| v > 0 && v <= max + 1)
        }) {
            return false;
        }
        if range == "*" {
            return true;
        }
        if let Some((start, end)) = range.split_once('-') {
            return number(start)
                && number(end)
                && start.parse::<u32>().unwrap() <= end.parse::<u32>().unwrap();
        }
        number(range)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_validate_supported_events_versions_and_literal_branches() {
        for yaml in [
            "{}",
            "branch: release/docs\nzel-version: 0.2.0-rc.1\ntags: ['v*']\nschedules: ['17 3 * * 1']",
            "triggers: []\ntags: ['v*']",
        ] {
            serde_yaml_ng::from_str::<Settings>(yaml)
                .unwrap()
                .validate()
                .unwrap();
        }
        for yaml in [
            "system: gitlab",
            "triggers: [pull_request_target]",
            "token: private",
            "branch: true",
            "tags: [true]",
            "zel-version: null",
        ] {
            assert!(serde_yaml_ng::from_str::<Settings>(yaml).is_err(), "{yaml}");
        }
        for yaml in [
            "triggers: []",
            "triggers: [push, push]",
            "branch: 'main*'",
            "branch: '../main'",
            "workflow: ../ci.yml",
            "workflow: foo.txt",
            "tags: ['!v*']",
            "schedules: ['99 3 * * *']",
            "schedules: ['0 0 1 1 * extra']",
            "schedules: ['*/0 0 * * *']",
            "zel-version: 0.1.3",
            "zel-version: latest",
        ] {
            assert!(
                serde_yaml_ng::from_str::<Settings>(yaml)
                    .unwrap()
                    .validate()
                    .is_err(),
                "{yaml}"
            );
        }
        for version in ["0.2.0", "v0.2.0-rc.1", "1.0.0", "0.3.7"] {
            assert!(version_pin(version).is_ok());
        }
        for version in [
            "v0.1.3",
            "0.02.0",
            "0.2",
            "0.2.0-",
            "0.2.0-rc..1",
            "latest",
            "0.2.0\n",
            "0.2.0;echo bad",
        ] {
            assert!(version_pin(version).is_err(), "{version}");
        }
    }
}
