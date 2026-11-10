//! Data-only subset of Jekyll collection metadata ordering.
use super::{
    Page,
    config::{self, invalid},
};
use crate::frontmatter;
use camino::{Utf8Path, Utf8PathBuf};
use serde_yaml_ng::{Number, Value};
use std::{cmp::Ordering, collections::BTreeMap, sync::OnceLock};
use zellij_core::Result;

#[derive(Debug)]
enum Numeric {
    Integer(i128),
    Float(f64),
}
impl Numeric {
    fn read(value: &Number) -> Option<Self> {
        value
            .as_i64()
            .map(|n| Self::Integer(n.into()))
            .or_else(|| value.as_u64().map(|n| Self::Integer(n.into())))
            .or_else(|| value.as_f64().filter(|n| n.is_finite()).map(Self::Float))
    }
    fn compare(&self, other: &Self) -> Ordering {
        fn mixed(integer: i128, float: f64) -> Ordering {
            // All parsed integers are in i64::MIN..=u64::MAX. Bound the cast,
            // then compare its integer and fractional parts without rounding
            // large integers through f64 (which loses precision above 2^53).
            if float >= 18_446_744_073_709_551_616.0 {
                Ordering::Less
            } else if float < -9_223_372_036_854_775_808.0 {
                Ordering::Greater
            } else {
                integer
                    .cmp(&(float as i128))
                    .then_with(|| 0.0_f64.partial_cmp(&float.fract()).unwrap())
            }
        }
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => a.cmp(b),
            (Self::Float(a), Self::Float(b)) => a.partial_cmp(b).unwrap(),
            (Self::Integer(a), Self::Float(b)) => mixed(*a, *b),
            (Self::Float(a), Self::Integer(b)) => mixed(*b, *a).reverse(),
        }
    }
}
#[derive(Debug)]
enum Key {
    Number(Numeric),
    Text(String),
    Missing,
}
impl Key {
    fn compare(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Missing, Self::Missing) => Ordering::Equal,
            (Self::Missing, _) => Ordering::Greater,
            (_, Self::Missing) => Ordering::Less,
            (Self::Number(a), Self::Number(b)) => a.compare(b),
            (Self::Text(a), Self::Text(b)) => a.cmp(b),
            _ => unreachable!("mixed key families are rejected before sorting"),
        }
    }
}
fn value(path: &Utf8Path, page: &Page, key: &str) -> Result<Key> {
    let fail = |reason| {
        invalid(format!(
            "{path}: collection sort_by {key:?}: {reason}; use a numeric rank or literal order list"
        ))
    };
    if ["date", "categories", "tags", "excerpt"].contains(&key) {
        return Err(fail(
            "generated date, array and excerpt fields are not supported",
        ));
    }
    let metadata = frontmatter::parse(&page.raw, path)?;
    let data = config::yaml(metadata.raw.unwrap_or(""))?;
    let mut field = data.get(key).cloned().unwrap_or(Value::Null);
    // Jekyll fills these three fields from the basename after applying defaults.
    if field.is_null() && ["title", "slug", "ext"].contains(&key) {
        let slug = path.file_stem().unwrap_or("").trim_end_matches('.');
        field = Value::String(match key {
            "slug" => slug.to_owned(),
            "ext" => path
                .extension()
                .map(|ext| format!(".{ext}"))
                .unwrap_or_default(),
            _ => {
                if !slug.is_ascii() {
                    return Err(fail(
                        "non-ASCII generated titles require an explicit title in front matter",
                    ));
                }
                // Ruby's capitalize uses Unicode titlecase; the ASCII subset is
                // unambiguous without executing Ruby or adding casing tables.
                slug.trim_end_matches('-')
                    .split('-')
                    .map(|word| {
                        let mut word = word.to_ascii_lowercase();
                        if let Some(first) = word.get_mut(..1) {
                            first.make_ascii_uppercase();
                        }
                        word
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        });
    }
    match field {
        Value::Null => Ok(Key::Missing),
        Value::Number(number) => Numeric::read(&number)
            .map(Key::Number)
            .ok_or_else(|| fail("numbers must be finite")),
        Value::String(text) => {
            // YAML timestamps and TOML datetimes have already become text.
            // Do not guess their source type or compare timezone offsets as text.
            static TEMPORAL: OnceLock<regex::Regex> = OnceLock::new();
            if TEMPORAL
                .get_or_init(|| {
                    regex::Regex::new(
                        r"^(?:[0-9]{4}-[0-9]{1,2}-[0-9]{1,2}(?:$|[Tt ])|[0-9]{2}:[0-9]{2}:[0-9]{2})",
                    )
                    .unwrap()
                })
                .is_match(&text)
            {
                return Err(fail("date/time-shaped values require explicit ordering"));
            }
            Ok(Key::Text(text))
        }
        _ => Err(fail("values must be numbers, text or null")),
    }
}
pub(super) fn sort(
    pages: &mut [Utf8PathBuf],
    all: &BTreeMap<Utf8PathBuf, Page>,
    key: &str,
) -> Result<()> {
    let values: BTreeMap<_, _> = pages
        .iter()
        .map(|path| value(path, &all[path], key).map(|value| (path.clone(), value)))
        .collect::<Result<_>>()?;
    let numeric = values.values().any(|v| matches!(v, Key::Number(_)));
    let text = values.values().any(|v| matches!(v, Key::Text(_)));
    if numeric && text {
        return Err(invalid(format!(
            "collection sort_by {key:?} mixes numbers and text; use one comparable type or a literal order list"
        )));
    }
    pages.sort_by(|a, b| values[a].compare(&values[b]).then_with(|| a.cmp(b)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn number(raw: &str) -> Numeric {
        let Value::Number(value) = serde_yaml_ng::from_str(raw).unwrap() else {
            panic!("expected numeric fixture: {raw}");
        };
        Numeric::read(&value).unwrap()
    }
    #[test]
    fn numeric_order_is_exact_across_integer_and_float_boundaries() {
        let rows = [
            "-1e30",
            "-9223372036854775808",
            "-1.5",
            "-1",
            "-0.5",
            "0",
            "0.5",
            "1",
            "9007199254740992.0",
            "9007199254740993",
            "18446744073709551615",
            "18446744073709551616.0",
            "1e30",
        ];
        for (i, a) in rows.iter().enumerate() {
            for (j, b) in rows.iter().enumerate() {
                assert_eq!(number(a).compare(&number(b)), i.cmp(&j), "{a} vs {b}");
            }
        }
        for (a, b) in [
            ("-0.0", "0"),
            ("2", "2.0"),
            ("-9223372036854775808", "-9223372036854775808.0"),
        ] {
            assert_eq!(number(a).compare(&number(b)), Ordering::Equal);
            assert_eq!(number(b).compare(&number(a)), Ordering::Equal);
        }
        for raw in [".nan", ".inf", "-.inf"] {
            let Value::Number(value) = serde_yaml_ng::from_str(raw).unwrap() else {
                panic!("expected numeric fixture: {raw}");
            };
            assert!(Numeric::read(&value).is_none());
        }
    }
}
