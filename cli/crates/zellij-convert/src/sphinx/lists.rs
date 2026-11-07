//! Split RST list items without interpreting their block content.
use super::invalid;
use zellij_core::Result;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Bullet(char),
    Ordered(char, Style),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Style {
    Decimal,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
}
impl Style {
    pub fn html_type(self) -> Option<char> {
        match self {
            Self::Decimal => None,
            Self::LowerAlpha => Some('a'),
            Self::UpperAlpha => Some('A'),
            Self::LowerRoman => Some('i'),
            Self::UpperRoman => Some('I'),
        }
    }
    fn limit(self) -> u32 {
        match self {
            Self::Decimal => i32::MAX as u32,
            Self::LowerAlpha | Self::UpperAlpha => 26,
            Self::LowerRoman | Self::UpperRoman => 3999,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Numbering {
    pub start: u32,
    pub style: Style,
}

pub struct Marker {
    kind: Kind,
    number: Option<u32>,
    offset: usize,
    indent: usize,
}

pub struct List {
    pub numbering: Option<Numbering>,
    pub items: Vec<String>,
}

pub fn marker(line: &str) -> Result<Option<Marker>> {
    marker_in_sequence(line, None)
}

fn roman(token: &str) -> Option<u32> {
    if token.is_empty() || token.len() > 15 {
        return None;
    }
    let pairs = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let upper = token.to_ascii_uppercase();
    let mut rest = upper.as_str();
    let mut value = 0;
    for (n, text) in pairs {
        while let Some(tail) = rest.strip_prefix(text) {
            value += n;
            rest = tail;
        }
    }
    if !rest.is_empty() || !(1..=3999).contains(&value) {
        return None;
    }
    let mut remaining = value;
    let mut canonical = String::new();
    for (n, text) in pairs {
        while remaining >= n {
            remaining -= n;
            canonical.push_str(text);
        }
    }
    (canonical == upper).then_some(value)
}

fn sequence(token: &str, expected: Option<Style>) -> Result<Option<(Style, Option<u32>)>> {
    if token == "#" {
        return Ok(Some((expected.unwrap_or(Style::Decimal), None)));
    }
    if token.is_empty() {
        return Ok(None);
    }
    if token.bytes().all(|b| b.is_ascii_digit()) {
        let number = token
            .parse::<u32>()
            .ok()
            .filter(|n| *n <= i32::MAX as u32)
            .ok_or_else(|| {
                invalid("list number exceeds the supported HTML range (0–2147483647)")
            })?;
        return Ok(Some((Style::Decimal, Some(number))));
    }
    let lower = token.bytes().all(|b| b.is_ascii_lowercase());
    let upper = token.bytes().all(|b| b.is_ascii_uppercase());
    let alpha = if lower {
        Style::LowerAlpha
    } else {
        Style::UpperAlpha
    };
    let roman_style = if lower {
        Style::LowerRoman
    } else {
        Style::UpperRoman
    };
    // Within an existing list, its sequence resolves ambiguous I/V/X/etc.
    if (lower || upper)
        && expected == Some(roman_style)
        && let Some(number) = roman(token)
    {
        return Ok(Some((roman_style, Some(number))));
    }
    if token.len() == 1 && (lower || upper) {
        if expected != Some(alpha) && token.eq_ignore_ascii_case("i") {
            return Ok(Some((roman_style, Some(1))));
        }
        return Ok(Some((
            alpha,
            Some(u32::from(token.as_bytes()[0].to_ascii_lowercase() - b'a') + 1),
        )));
    }
    if token.chars().all(|c| "ivxlcdmIVXLCDM".contains(c)) {
        let number = if lower || upper { roman(token) } else { None }.ok_or_else(|| {
            invalid("Roman list markers must be canonical, consistently cased and within 1–3999")
        })?;
        return Ok(Some((roman_style, Some(number))));
    }
    Ok(None)
}

/// A wrapped sentence such as "A. Einstein ..." is prose when its next line
/// is unindented prose, not a new list. Decimal handling retains its strict
/// existing boundary checks; Markdown would otherwise invent a list again.
pub fn starts(lines: &[&str]) -> Result<bool> {
    let Some(first) = marker(lines[0])? else {
        return Ok(false);
    };
    if matches!(first.kind, Kind::Ordered(_, style) if style != Style::Decimal)
        && let Some(next) = lines.get(1)
        && !next.trim().is_empty()
        && !next.starts_with(' ')
        && marker(next)?.is_none()
    {
        return Ok(false);
    }
    Ok(true)
}

fn marker_in_sequence(line: &str, expected: Option<Style>) -> Result<Option<Marker>> {
    let Some(first) = line.chars().next() else {
        return Ok(None);
    };
    let (kind, number, end) = if matches!(first, '*' | '+' | '-' | '•' | '‣' | '⁃') {
        (Kind::Bullet(first), None, first.len_utf8())
    } else {
        let (token, format, end) = if let Some(tail) = line.strip_prefix('(') {
            let Some(end) = tail.find(')') else {
                return Ok(None);
            };
            (&tail[..end], '(', end + 2)
        } else {
            let Some(end) = line.find(['.', ')']) else {
                return Ok(None);
            };
            (&line[..end], line.as_bytes()[end] as char, end + 1)
        };
        if !line[end..].is_empty() && !line[end..].starts_with(' ') {
            return Ok(None);
        }
        let Some((style, number)) = sequence(token, expected)? else {
            return Ok(None);
        };
        (Kind::Ordered(format, style), number, end)
    };
    let tail = &line[end..];
    if !tail.is_empty() && !tail.starts_with(' ') {
        return Ok(None);
    }
    let spaces = tail.bytes().take_while(|b| *b == b' ').count();
    Ok(Some(Marker {
        kind,
        number,
        offset: end + spaces.max(1),
        // Unicode bullets occupy one display column, but multiple UTF-8 bytes.
        indent: if matches!(kind, Kind::Bullet(_)) {
            1
        } else {
            end
        } + spaces.max(1),
    }))
}

pub fn parse(lines: &[&str]) -> Result<(List, usize)> {
    let first = marker(lines[0])?.ok_or_else(|| invalid("expected a list marker"))?;
    let numbering = match first.kind {
        Kind::Ordered(_, style) => Some(Numbering {
            start: first.number.unwrap_or(1),
            style,
        }),
        Kind::Bullet(_) => None,
    };
    let mut list = List {
        numbering,
        items: Vec::new(),
    };
    let mut i = 0;
    let mut auto = false;
    while let Some(line) = lines.get(i) {
        let Some(current) = marker_in_sequence(line, numbering.map(|n| n.style))? else {
            break;
        };
        if current.kind != first.kind {
            break;
        }
        if let Some(numbering) = numbering {
            let expected = numbering.start + list.items.len() as u32;
            if current
                .number
                .is_some_and(|number| auto || number != expected)
            {
                break; // RST starts a new list when explicit numbering restarts.
            }
            if expected > numbering.style.limit() {
                return Err(invalid(
                    "automatic list number exceeds the supported sequence range",
                ));
            }
            auto |= current.number.is_none();
        }
        if list.items.len() >= 10000 {
            return Err(invalid("list exceeds 10000 items"));
        }
        let mut body = line.get(current.offset..).unwrap_or("").to_owned();
        i += 1;
        while let Some(line) = lines.get(i) {
            if line.trim().is_empty() {
                body.push('\n');
            } else if line.starts_with(' ') {
                let indent = line.bytes().take_while(|b| *b == b' ').count();
                if indent < current.indent {
                    return Err(invalid("list continuation must align with its item's text"));
                }
                body.push('\n');
                body.push_str(&line[current.indent..]);
            } else {
                break;
            }
            i += 1;
        }
        list.items.push(body);
        if let Some(next) = lines.get(i)
            && marker(next)?.is_none()
            && !lines[i - 1].trim().is_empty()
        {
            return Err(invalid("a list needs a blank line before following text"));
        }
    }
    Ok((list, i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_indentation_preserves_nested_blocks_and_empty_items() {
        let (list, used) = parse(&[
            "•   first",
            "    continued",
            "",
            "    - nested",
            "",
            "•",
            "",
            "after",
        ])
        .unwrap();
        assert_eq!(used, 7);
        assert_eq!(list.numbering, None);
        assert_eq!(list.items, ["first\ncontinued\n\n- nested\n", "\n"]);
        assert!(parse(&["-   first", "  under-indented"]).is_err());
        assert!(parse(&["- first", "outside without blank"]).is_err());
    }

    #[test]
    fn numbering_formats_auto_numbers_and_restarts_keep_separate_lists() {
        for (first, second, third) in [
            ("9. a", "10. b", "#. c"),
            ("9) a", "10) b", "#) c"),
            ("(9) a", "(10) b", "(#) c"),
        ] {
            let (list, used) = parse(&[first, second, third]).unwrap();
            assert_eq!(
                list.numbering,
                Some(Numbering {
                    start: 9,
                    style: Style::Decimal
                })
            );
            assert_eq!(list.items, ["a", "b", "c"]);
            assert_eq!(used, 3);
        }
        let (list, used) = parse(&["#. one", "#. two", "8. restart"]).unwrap();
        assert_eq!(
            list.numbering,
            Some(Numbering {
                start: 1,
                style: Style::Decimal
            })
        );
        assert_eq!(used, 2);
        assert_eq!(parse(&["- a", "+ b"]).unwrap().1, 1);
        assert_eq!(parse(&["1. a", "2) b"]).unwrap().1, 1);
        // Docutils starts a new list even when an explicit number after #
        // happens to be consecutive.
        assert_eq!(parse(&["9. a", "#. b", "11. c"]).unwrap().1, 2);
    }

    #[test]
    fn alphabetic_and_roman_sequences_resolve_ambiguous_letters() {
        for (markers, style, start) in [
            (["h", "i", "j", "#"], Style::LowerAlpha, 8),
            (["H", "I", "J", "#"], Style::UpperAlpha, 8),
            (["iv", "v", "vi", "#"], Style::LowerRoman, 4),
            (["IV", "V", "VI", "#"], Style::UpperRoman, 4),
            (["i", "ii", "iii", "#"], Style::LowerRoman, 1),
            (["I", "II", "III", "#"], Style::UpperRoman, 1),
            (["V", "W", "X", "#"], Style::UpperAlpha, 22),
        ] {
            for format in ['.', ')', '('] {
                let text: Vec<_> = markers
                    .iter()
                    .map(|m| match format {
                        '(' => format!("({m}) item"),
                        _ => format!("{m}{format} item"),
                    })
                    .collect();
                let lines: Vec<_> = text.iter().map(String::as_str).collect();
                let (list, used) = parse(&lines).unwrap();
                assert_eq!(used, 4, "{lines:?}");
                assert_eq!(list.numbering, Some(Numbering { start, style }));
                assert_eq!(list.items, ["item"; 4]);
            }
        }
        for lines in [
            ["a. one", "A. changed case"],
            ["iv. four", "vi. skipped"],
            ["h. eight", "i) changed format"],
            ["#. one", "ii. changed sequence"],
        ] {
            assert_eq!(parse(&lines).unwrap().1, 1, "{lines:?}");
        }
        assert_eq!(parse(&["h. eight", "#. nine", "j. ten"]).unwrap().1, 2);
        assert_eq!(parse(&["iv. four", "#. five", "vi. six"]).unwrap().1, 2);
    }

    #[test]
    fn roman_canonical_forms_and_sequence_bounds_are_checked() {
        for (text, number) in [
            ("I", 1),
            ("IV", 4),
            ("IX", 9),
            ("XL", 40),
            ("XC", 90),
            ("CD", 400),
            ("CM", 900),
            ("MMXXVI", 2026),
            ("MMMCMXCIX", 3999),
        ] {
            assert_eq!(roman(text), Some(number));
            assert_eq!(roman(&text.to_lowercase()), Some(number));
        }
        for text in [
            "iiii. invalid",
            "iV. mixed case",
            "IC. invalid",
            "VX. invalid",
            "MMMM. too large",
        ] {
            assert!(marker(text).is_err(), "{text}");
        }
        assert!(parse(&["z. last", "#. overflow"]).is_err());
        assert!(parse(&["Z. last", "#. overflow"]).is_err());
        assert!(parse(&["MMMCMXCIX. last", "#. overflow"]).is_err());
        assert_eq!(parse(&["y. next to last", "#. last"]).unwrap().1, 2);
        assert_eq!(
            parse(&["mmmcmxcviii. penultimate", "#. last"]).unwrap().1,
            2
        );
    }

    #[test]
    fn wrapped_prose_and_escaped_letters_are_not_lists() {
        assert!(!starts(&["A. Einstein was", "a physicist."]).unwrap());
        assert!(!starts(&["I. Introduction discusses", "the project."]).unwrap());
        assert!(starts(&["A. first", "   continued"]).unwrap());
        assert!(starts(&["A. first", "", "following paragraph"]).unwrap());
        for text in [
            "\\a. literal",
            "\\(iv) literal",
            "a.word",
            "iv)word",
            "word. prose",
            "(aa) prose",
        ] {
            assert!(marker(text).unwrap().is_none(), "{text}");
        }
    }

    #[test]
    fn list_bounds_and_literal_prefixes_are_checked() {
        assert!(marker("2147483648. too large").is_err());
        assert!(parse(&["2147483647. last", "#. overflow"]).is_err());
        assert!(parse(&vec!["- x"; 10001]).is_err());
        for text in [
            "*emphasis*",
            "-word",
            "1.2",
            "999999999999999.5",
            "(1)word",
            "\\- escaped",
        ] {
            assert!(marker(text).unwrap().is_none(), "{text}");
        }
    }
}
