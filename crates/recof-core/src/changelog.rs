//! Release notes, read as the Markdown GitHub shows them.
//!
//! Only what release notes actually use is understood: headings, lists,
//! paragraphs, block quotes with GitHub's alerts (`> [!WARNING]`), tables
//! and fenced code, and inside a line bold, inline code and links.
//! Anything else stays as its text, so nothing in the notes is lost, it
//! is only shown plain.

/// A run of text inside a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Span {
    Text(String),
    Bold(String),
    Code(String),
    Link { text: String, url: String },
}

/// The kinds of alert GitHub draws as a coloured box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alert {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl Alert {
    fn from_marker(line: &str) -> Option<Alert> {
        let inner = line.trim().strip_prefix("[!")?.strip_suffix(']')?;
        match inner.to_ascii_uppercase().as_str() {
            "NOTE" => Some(Alert::Note),
            "TIP" => Some(Alert::Tip),
            "IMPORTANT" => Some(Alert::Important),
            "WARNING" => Some(Alert::Warning),
            "CAUTION" => Some(Alert::Caution),
            _ => None,
        }
    }

    /// The Fluent message id of the title the box is shown with.
    pub fn message_id(self) -> &'static str {
        match self {
            Alert::Note => "changelog-note",
            Alert::Tip => "changelog-tip",
            Alert::Important => "changelog-important",
            Alert::Warning => "changelog-warning",
            Alert::Caution => "changelog-caution",
        }
    }
}

/// One block of the notes, in reading order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading {
        level: u8,
        spans: Vec<Span>,
    },
    Paragraph(Vec<Span>),
    /// A list item. `depth` 0 is the outer list; `number` is set for an
    /// ordered list.
    Item {
        depth: usize,
        number: Option<u32>,
        spans: Vec<Span>,
    },
    /// A block quote, or an alert when it starts with a marker. Each entry
    /// is one paragraph.
    Quote {
        alert: Option<Alert>,
        paragraphs: Vec<Vec<Span>>,
    },
    Table {
        header: Vec<Vec<Span>>,
        rows: Vec<Vec<Vec<Span>>>,
    },
    Code(String),
    Rule,
}

/// Reads release notes into blocks.
pub fn parse(text: &str) -> Vec<Block> {
    let text = text.replace("\r\n", "\n");
    let lines: Vec<&str> = text.lines().collect();
    let mut blocks = Vec::new();
    let mut paragraph: Vec<&str> = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            flush(&mut paragraph, &mut blocks);
            index += 1;
            continue;
        }

        if let Some(fence) = fence_of(trimmed) {
            flush(&mut paragraph, &mut blocks);
            let mut code = Vec::new();
            index += 1;
            while index < lines.len() && !lines[index].trim().starts_with(fence) {
                code.push(lines[index]);
                index += 1;
            }
            blocks.push(Block::Code(code.join("\n")));
            index += 1;
            continue;
        }

        if let Some((level, rest)) = heading(trimmed) {
            flush(&mut paragraph, &mut blocks);
            blocks.push(Block::Heading {
                level,
                spans: spans(rest),
            });
            index += 1;
            continue;
        }

        if is_rule(trimmed) {
            flush(&mut paragraph, &mut blocks);
            blocks.push(Block::Rule);
            index += 1;
            continue;
        }

        if trimmed.starts_with('>') {
            flush(&mut paragraph, &mut blocks);
            let mut quoted = Vec::new();
            while index < lines.len() && lines[index].trim().starts_with('>') {
                let inner = lines[index].trim().trim_start_matches('>');
                quoted.push(inner.strip_prefix(' ').unwrap_or(inner));
                index += 1;
            }
            blocks.push(quote(&quoted));
            continue;
        }

        if trimmed.starts_with('|') && index + 1 < lines.len() && is_separator(lines[index + 1]) {
            flush(&mut paragraph, &mut blocks);
            let header = cells(trimmed);
            index += 2;
            let mut rows = Vec::new();
            while index < lines.len() && lines[index].trim().starts_with('|') {
                rows.push(cells(lines[index].trim()));
                index += 1;
            }
            blocks.push(Block::Table { header, rows });
            continue;
        }

        if let Some((number, rest)) = list_item(trimmed) {
            flush(&mut paragraph, &mut blocks);
            let depth = indent_of(line) / 2;
            let mut text = rest.to_owned();
            index += 1;
            // A wrapped item continues on indented lines that start
            // nothing of their own.
            while index < lines.len() {
                let next = lines[index];
                let next_trimmed = next.trim();
                if next_trimmed.is_empty()
                    || indent_of(next) == 0
                    || list_item(next_trimmed).is_some()
                {
                    break;
                }
                text.push(' ');
                text.push_str(next_trimmed);
                index += 1;
            }
            blocks.push(Block::Item {
                depth,
                number,
                spans: spans(&text),
            });
            continue;
        }

        paragraph.push(trimmed);
        index += 1;
    }

    flush(&mut paragraph, &mut blocks);
    blocks
}

/// Reads the inline markup of one line.
pub fn spans(text: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut plain = String::new();
    let mut rest = text;

    while let Some(character) = rest.chars().next() {
        if character == '\\' {
            let mut chars = rest.chars();
            chars.next();
            match chars.next() {
                Some(escaped) if escaped.is_ascii_punctuation() => {
                    plain.push(escaped);
                    rest = chars.as_str();
                }
                _ => {
                    plain.push('\\');
                    rest = &rest[1..];
                }
            }
            continue;
        }

        if let Some(after) = rest.strip_prefix('`') {
            if let Some(end) = after.find('`') {
                push_plain(&mut plain, &mut spans);
                spans.push(Span::Code(after[..end].to_owned()));
                rest = &after[end + 1..];
                continue;
            }
        }

        if let Some(after) = rest.strip_prefix("**") {
            if let Some(end) = after.find("**") {
                push_plain(&mut plain, &mut spans);
                spans.push(Span::Bold(after[..end].replace('`', "")));
                rest = &after[end + 2..];
                continue;
            }
        }

        let image = rest.starts_with("![");
        let link_start = if image { &rest[1..] } else { rest };
        if let Some((label, url, used)) = link(link_start) {
            let label = label.replace('`', "");
            if image {
                // A picture cannot be drawn here; its description can.
                plain.push_str(&label);
                rest = &rest[1 + used..];
            } else {
                push_plain(&mut plain, &mut spans);
                spans.push(Span::Link {
                    text: label,
                    url: url.to_owned(),
                });
                rest = &rest[used..];
            }
            continue;
        }

        plain.push(character);
        rest = &rest[character.len_utf8()..];
    }

    push_plain(&mut plain, &mut spans);
    spans
}

/// The text of the notes with every mark taken out, for a log line or a
/// place that cannot draw them.
pub fn plain_text(spans: &[Span]) -> String {
    spans
        .iter()
        .map(|span| match span {
            Span::Text(text) | Span::Bold(text) | Span::Code(text) => text.as_str(),
            Span::Link { text, .. } => text.as_str(),
        })
        .collect()
}

fn push_plain(plain: &mut String, spans: &mut Vec<Span>) {
    if !plain.is_empty() {
        spans.push(Span::Text(std::mem::take(plain)));
    }
}

fn flush(paragraph: &mut Vec<&str>, blocks: &mut Vec<Block>) {
    if !paragraph.is_empty() {
        blocks.push(Block::Paragraph(spans(&paragraph.join(" "))));
        paragraph.clear();
    }
}

fn fence_of(trimmed: &str) -> Option<&'static str> {
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

fn heading(trimmed: &str) -> Option<(u8, &str)> {
    let hashes = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let rest = trimmed[hashes..].strip_prefix(' ')?;
    Some((hashes as u8, rest.trim().trim_end_matches('#').trim_end()))
}

fn is_rule(trimmed: &str) -> bool {
    let compact: Vec<u8> = trimmed
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    compact.len() >= 3
        && b"-*_"
            .iter()
            .any(|mark| compact.iter().all(|byte| byte == mark))
}

fn is_separator(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|')
        && trimmed.contains('-')
        && trimmed.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

fn cells(row: &str) -> Vec<Vec<Span>> {
    let inner = row.trim().trim_start_matches('|');
    let inner = inner.strip_suffix('|').unwrap_or(inner);
    inner.split('|').map(|cell| spans(cell.trim())).collect()
}

fn list_item(trimmed: &str) -> Option<(Option<u32>, &str)> {
    for bullet in ["- ", "* ", "+ "] {
        if let Some(rest) = trimmed.strip_prefix(bullet) {
            return Some((None, rest));
        }
    }
    let digits = trimmed.bytes().take_while(u8::is_ascii_digit).count();
    if digits > 0 && digits <= 9 {
        let rest = trimmed[digits..]
            .strip_prefix(". ")
            .or_else(|| trimmed[digits..].strip_prefix(") "))?;
        return Some((trimmed[..digits].parse().ok(), rest));
    }
    None
}

fn indent_of(line: &str) -> usize {
    line.chars()
        .take_while(|c| c.is_whitespace())
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

fn quote(lines: &[&str]) -> Block {
    let mut alert = None;
    let mut body = lines;
    if let Some(first) = lines.first() {
        if let Some(found) = Alert::from_marker(first) {
            alert = Some(found);
            body = &lines[1..];
        }
    }

    let mut paragraphs = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in body {
        if line.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(spans(&current.join(" ")));
                current.clear();
            }
        } else {
            current.push(line.trim());
        }
    }
    if !current.is_empty() {
        paragraphs.push(spans(&current.join(" ")));
    }

    Block::Quote { alert, paragraphs }
}

/// `[text](url)` at the start of `text`: the label, the address and how
/// many bytes it took.
fn link(text: &str) -> Option<(&str, &str, usize)> {
    let after = text.strip_prefix('[')?;
    let close = after.find("](")?;
    let label = &after[..close];
    if label.contains('[') {
        return None;
    }
    let address = &after[close + 2..];
    let end = address.find(')')?;
    let url = address[..end].trim();
    Some((label, url, 1 + close + 2 + end + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(value: &str) -> Span {
        Span::Text(value.to_owned())
    }

    #[test]
    fn a_github_alert_becomes_an_alert_block() {
        let blocks = parse("> [!WARNING]\n> **For testing only.** Use a copy.\n\nAfter the box.");
        assert_eq!(
            blocks[0],
            Block::Quote {
                alert: Some(Alert::Warning),
                paragraphs: vec![vec![
                    Span::Bold("For testing only.".to_owned()),
                    text(" Use a copy."),
                ]],
            }
        );
        assert_eq!(blocks[1], Block::Paragraph(vec![text("After the box.")]));
    }

    #[test]
    fn every_alert_kind_is_known_and_an_ordinary_quote_has_none() {
        for (marker, kind) in [
            ("[!NOTE]", Alert::Note),
            ("[!TIP]", Alert::Tip),
            ("[!IMPORTANT]", Alert::Important),
            ("[!WARNING]", Alert::Warning),
            ("[!caution]", Alert::Caution),
        ] {
            assert_eq!(Alert::from_marker(marker), Some(kind));
            assert!(kind.message_id().starts_with("changelog-"));
        }
        let blocks = parse("> just a quote");
        assert!(matches!(&blocks[0], Block::Quote { alert: None, .. }));
    }

    #[test]
    fn headings_lists_and_paragraphs_are_told_apart() {
        let blocks = parse(
            "## What changed\n\nAbout 60 issues\nare fixed.\n\n- one\n- two\n  wrapped\n  - nested\n1. first\n",
        );
        assert_eq!(
            blocks[0],
            Block::Heading {
                level: 2,
                spans: vec![text("What changed")],
            }
        );
        assert_eq!(
            blocks[1],
            Block::Paragraph(vec![text("About 60 issues are fixed.")])
        );
        assert_eq!(
            blocks[2],
            Block::Item {
                depth: 0,
                number: None,
                spans: vec![text("one")],
            }
        );
        assert_eq!(
            blocks[3],
            Block::Item {
                depth: 0,
                number: None,
                spans: vec![text("two wrapped")],
            }
        );
        assert!(matches!(blocks[4], Block::Item { depth: 1, .. }));
        assert!(matches!(
            blocks[5],
            Block::Item {
                number: Some(1),
                ..
            }
        ));
    }

    #[test]
    fn inline_marks_become_spans() {
        assert_eq!(
            spans("Run `cof.exe` from [DeCoF](https://github.com/caixax/DeCoF), **now**."),
            vec![
                text("Run "),
                Span::Code("cof.exe".to_owned()),
                text(" from "),
                Span::Link {
                    text: "DeCoF".to_owned(),
                    url: "https://github.com/caixax/DeCoF".to_owned(),
                },
                text(", "),
                Span::Bold("now".to_owned()),
                text("."),
            ]
        );
        // Code inside bold or a link loses only its backticks.
        assert_eq!(
            spans("**`ReCoF-v0.1.0.zip`**"),
            vec![Span::Bold("ReCoF-v0.1.0.zip".to_owned())]
        );
    }

    #[test]
    fn unmatched_marks_stay_as_text() {
        assert_eq!(spans("a ` b"), vec![text("a ` b")]);
        assert_eq!(spans("2 ** 3"), vec![text("2 ** 3")]);
        assert_eq!(spans("[not a link]"), vec![text("[not a link]")]);
        assert_eq!(spans(r"\*literal\*"), vec![text("*literal*")]);
        assert_eq!(spans(r"C:\Games"), vec![text(r"C:\Games")]);
    }

    #[test]
    fn an_image_leaves_its_description() {
        assert_eq!(
            spans("see ![the menu](menu.png) here"),
            vec![text("see the menu here")]
        );
    }

    #[test]
    fn a_table_keeps_its_header_and_rows() {
        let blocks = parse(
            "| File | What it is |\n|---|---|\n| `cof.exe` | engine launcher |\n| `hw.dll`, `sw.dll` | engine |\n",
        );
        let Block::Table { header, rows } = &blocks[0] else {
            panic!("{blocks:?}");
        };
        assert_eq!(header, &vec![vec![text("File")], vec![text("What it is")]]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], vec![Span::Code("cof.exe".to_owned())]);
        assert_eq!(plain_text(&rows[1][0]), "hw.dll, sw.dll");
    }

    #[test]
    fn fenced_code_is_kept_verbatim() {
        let blocks = parse("```sh\ncargo xtask detect\n  **not bold**\n```\nafter");
        assert_eq!(
            blocks[0],
            Block::Code("cargo xtask detect\n  **not bold**".to_owned())
        );
        assert_eq!(blocks[1], Block::Paragraph(vec![text("after")]));
    }

    #[test]
    fn rules_and_windows_line_ends_are_handled() {
        let blocks = parse("one\r\n\r\n---\r\ntwo\r\n");
        assert_eq!(
            blocks,
            vec![
                Block::Paragraph(vec![text("one")]),
                Block::Rule,
                Block::Paragraph(vec![text("two")]),
            ]
        );
    }

    #[test]
    fn nothing_and_odd_input_do_not_panic() {
        assert!(parse("").is_empty());
        for odd in [
            "#", "#nospace", "> ", "|", "| a |\n|", "```", "- ", "1.", "![", "[a](", "**", "\\",
        ] {
            let _ = parse(odd);
        }
        assert_eq!(
            parse("#nospace"),
            vec![Block::Paragraph(vec![text("#nospace")])]
        );
    }
}
