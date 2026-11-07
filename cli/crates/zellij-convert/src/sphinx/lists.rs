//! Split RST list items without interpreting their block content.
use super::invalid;
use zellij_core::Result;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Bullet(char),
    Ordered(char),
}

pub struct Marker {
    kind: Kind,
    number: Option<u32>,
    offset: usize,
    indent: usize,
}

pub struct List {
    pub start: Option<u32>,
    pub items: Vec<String>,
}

pub fn marker(line: &str) -> Result<Option<Marker>> {
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
        if token != "#" && (token.is_empty() || !token.bytes().all(|b| b.is_ascii_digit())) {
            if token.len() == 1 && token.as_bytes()[0].is_ascii_alphabetic()
                || !token.is_empty() && token.chars().all(|c| "ivxlcdmIVXLCDM".contains(c))
            {
                return Err(invalid(
                    "alphabetic/Roman list numbering requires manual migration; escape the marker for literal prose",
                ));
            }
            return Ok(None);
        }
        let number = if token == "#" {
            None
        } else {
            Some(
                token
                    .parse::<u32>()
                    .ok()
                    .filter(|n| *n <= i32::MAX as u32)
                    .ok_or_else(|| {
                        invalid("list number exceeds the supported HTML range (0–2147483647)")
                    })?,
            )
        };
        (Kind::Ordered(format), number, end)
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
    let start = matches!(first.kind, Kind::Ordered(_)).then(|| first.number.unwrap_or(1));
    let mut list = List {
        start,
        items: Vec::new(),
    };
    let mut i = 0;
    while let Some(line) = lines.get(i) {
        let Some(current) = marker(line)? else { break };
        if current.kind != first.kind {
            break;
        }
        if let Some(start) = start {
            let expected = start + list.items.len() as u32;
            if current.number.is_some_and(|number| number != expected) {
                break; // RST starts a new list when explicit numbering restarts.
            }
            if expected > i32::MAX as u32 {
                return Err(invalid(
                    "automatic list number exceeds the supported HTML range",
                ));
            }
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
        assert_eq!(list.start, None);
        assert_eq!(list.items, ["first\ncontinued\n\n- nested\n", "\n"]);
        assert!(parse(&["-   first", "  under-indented"]).is_err());
        assert!(parse(&["- first", "outside without blank"]).is_err());
    }

    #[test]
    fn numbering_formats_auto_numbers_and_restarts_keep_separate_lists() {
        for (first, second, third) in [
            ("9. a", "#. b", "11. c"),
            ("9) a", "#) b", "11) c"),
            ("(9) a", "(#) b", "(11) c"),
        ] {
            let (list, used) = parse(&[first, second, third]).unwrap();
            assert_eq!(list.start, Some(9));
            assert_eq!(list.items, ["a", "b", "c"]);
            assert_eq!(used, 3);
        }
        let (list, used) = parse(&["#. one", "#. two", "8. restart"]).unwrap();
        assert_eq!(list.start, Some(1));
        assert_eq!(used, 2);
        assert_eq!(parse(&["- a", "+ b"]).unwrap().1, 1);
        assert_eq!(parse(&["1. a", "2) b"]).unwrap().1, 1);
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
