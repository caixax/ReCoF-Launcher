//! The text dialect of KeyValues, as Steam writes it.
//!
//! Handles quoted and unquoted tokens, nested blocks, `//` comments, the
//! `[$CONDITION]` suffix Valve puts on platform specific lines, a UTF-8 BOM,
//! and the escape sequences that appear in Windows paths inside
//! `libraryfolders.vdf`.

use super::{Object, Value};

/// Where parsing gave up and why.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("line {line}, column {column}: {kind}")]
pub struct KvError {
    pub line: usize,
    pub column: usize,
    pub kind: KvErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KvErrorKind {
    #[error("a quoted string was never closed")]
    UnterminatedString,
    #[error("a block was never closed")]
    UnterminatedBlock,
    #[error("a closing brace without a matching opening brace")]
    UnexpectedBrace,
    #[error("a key with no value")]
    MissingValue,
    #[error("a conditional was never closed")]
    UnterminatedConditional,
    #[error("the document nests more than {0} levels deep")]
    TooDeep(usize),
}

/// A document nested deeper than this is treated as malformed rather than
/// recursed into. Steam's own files are three or four levels deep.
const MAX_DEPTH: usize = 64;

/// Parses a KeyValues document and returns its top level block.
///
/// A document normally holds a single root pair, for example
/// `"libraryfolders" { ... }`, so the returned object usually has one entry.
pub fn parse(input: &str) -> Result<Object, KvError> {
    let without_bom = input.strip_prefix('\u{feff}').unwrap_or(input);
    Parser {
        bytes: without_bom.as_bytes(),
        position: 0,
        line: 1,
        line_start: 0,
    }
    .document()
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
    line: usize,
    /// Byte offset of the current line, so columns can be reported.
    line_start: usize,
}

/// One lexical item.
#[derive(Debug, PartialEq, Eq)]
enum Token {
    Open,
    Close,
    Text(String),
}

impl<'a> Parser<'a> {
    fn document(mut self) -> Result<Object, KvError> {
        let mut root = Object::new();
        loop {
            match self.next_token()? {
                None => return Ok(root),
                Some(Token::Close) => return Err(self.error(KvErrorKind::UnexpectedBrace)),
                Some(Token::Open) => return Err(self.error(KvErrorKind::MissingValue)),
                Some(Token::Text(key)) => {
                    let value = self.value(0)?;
                    root.push(key, value);
                }
            }
        }
    }

    /// Reads the value that follows a key.
    fn value(&mut self, depth: usize) -> Result<Value, KvError> {
        if depth >= MAX_DEPTH {
            return Err(self.error(KvErrorKind::TooDeep(MAX_DEPTH)));
        }
        match self.next_token()? {
            Some(Token::Text(text)) => {
                // A value may carry a platform condition, for example
                // "value" [$WIN32]. The condition is read and dropped: the
                // launcher only ever runs the Windows branch of a Steam file.
                self.skip_conditional()?;
                Ok(Value::String(text))
            }
            Some(Token::Open) => Ok(Value::Object(self.block(depth + 1)?)),
            Some(Token::Close) => Err(self.error(KvErrorKind::UnexpectedBrace)),
            None => Err(self.error(KvErrorKind::MissingValue)),
        }
    }

    /// Reads pairs until the matching closing brace.
    fn block(&mut self, depth: usize) -> Result<Object, KvError> {
        let mut object = Object::new();
        loop {
            match self.next_token()? {
                Some(Token::Close) => return Ok(object),
                Some(Token::Text(key)) => {
                    let value = self.value(depth)?;
                    object.push(key, value);
                }
                Some(Token::Open) => return Err(self.error(KvErrorKind::MissingValue)),
                None => return Err(self.error(KvErrorKind::UnterminatedBlock)),
            }
        }
    }

    /// Consumes a `[$CONDITION]` suffix if one follows on the same logical
    /// position. Nothing is returned: the launcher does not evaluate them.
    fn skip_conditional(&mut self) -> Result<(), KvError> {
        let saved = (self.position, self.line, self.line_start);
        self.skip_trivia();
        if self.peek() != Some(b'[') {
            self.position = saved.0;
            self.line = saved.1;
            self.line_start = saved.2;
            return Ok(());
        }
        self.advance();
        while let Some(byte) = self.peek() {
            self.advance();
            if byte == b']' {
                return Ok(());
            }
        }
        Err(self.error(KvErrorKind::UnterminatedConditional))
    }

    fn next_token(&mut self) -> Result<Option<Token>, KvError> {
        self.skip_trivia();
        let Some(byte) = self.peek() else {
            return Ok(None);
        };
        match byte {
            b'{' => {
                self.advance();
                Ok(Some(Token::Open))
            }
            b'}' => {
                self.advance();
                Ok(Some(Token::Close))
            }
            b'"' => self.quoted().map(|text| Some(Token::Text(text))),
            b'[' => {
                // A conditional on its own, which belongs to the previous
                // line. Skip it and read the next real token.
                self.skip_conditional()?;
                self.next_token()
            }
            _ => Ok(Some(Token::Text(self.unquoted()))),
        }
    }

    fn quoted(&mut self) -> Result<String, KvError> {
        let opened_at = (self.line, self.column());
        self.advance(); // the opening quote
        let mut text = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(KvError {
                    line: opened_at.0,
                    column: opened_at.1,
                    kind: KvErrorKind::UnterminatedString,
                });
            };
            match byte {
                b'"' => {
                    self.advance();
                    return Ok(text);
                }
                b'\\' => {
                    self.advance();
                    match self.peek() {
                        Some(b'n') => {
                            text.push('\n');
                            self.advance();
                        }
                        Some(b't') => {
                            text.push('\t');
                            self.advance();
                        }
                        Some(b'r') => {
                            text.push('\r');
                            self.advance();
                        }
                        Some(b'\\') => {
                            text.push('\\');
                            self.advance();
                        }
                        Some(b'"') => {
                            text.push('"');
                            self.advance();
                        }
                        // Not an escape Valve defines. Keep both characters,
                        // so a path written as C:\steam survives unchanged.
                        Some(_) => text.push('\\'),
                        None => {
                            return Err(KvError {
                                line: opened_at.0,
                                column: opened_at.1,
                                kind: KvErrorKind::UnterminatedString,
                            })
                        }
                    }
                }
                _ => {
                    text.push_str(&self.take_utf8_char());
                }
            }
        }
    }

    fn unquoted(&mut self) -> String {
        let mut text = String::new();
        while let Some(byte) = self.peek() {
            if byte.is_ascii_whitespace() || matches!(byte, b'{' | b'}' | b'"') {
                break;
            }
            if byte == b'/' && self.peek_at(1) == Some(b'/') {
                break;
            }
            text.push_str(&self.take_utf8_char());
        }
        text
    }

    /// Whitespace and `//` comments.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(byte) if byte.is_ascii_whitespace() => self.advance(),
                Some(b'/') if self.peek_at(1) == Some(b'/') => {
                    while let Some(byte) = self.peek() {
                        if byte == b'\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                _ => return,
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.position + offset).copied()
    }

    fn advance(&mut self) {
        if self.peek() == Some(b'\n') {
            self.line += 1;
            self.line_start = self.position + 1;
        }
        self.position += 1;
    }

    /// Consumes one whole UTF-8 character, so a multi byte character in a
    /// path or a game name is never split.
    fn take_utf8_char(&mut self) -> String {
        let start = self.position;
        let first = self.bytes[self.position];
        let width = match first {
            0x00..=0x7f => 1,
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            0xf0..=0xf7 => 4,
            // A stray continuation byte: take it alone and let the lossy
            // conversion below turn it into a replacement character.
            _ => 1,
        };
        for _ in 0..width.min(self.bytes.len() - start) {
            self.advance();
        }
        String::from_utf8_lossy(&self.bytes[start..self.position]).into_owned()
    }

    fn column(&self) -> usize {
        self.position - self.line_start + 1
    }

    fn error(&self, kind: KvErrorKind) -> KvError {
        KvError {
            line: self.line,
            column: self.column(),
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_modern_libraryfolders_file() {
        // Shape taken from a current Steam client. Paths are escaped.
        let input = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
		"contentid"		"1234567890123456789"
		"totalsize"		"0"
		"apps"
		{
			"228980"		"62939497"
		}
	}
	"1"
	{
		"path"		"I:\\SteamLibrary"
		"label"		""
		"apps"
		{
			"223710"		"4695601334"
		}
	}
}
"#;
        let root = parse(input).expect("a well formed file parses");
        let folders = root.get_object("libraryfolders").expect("the root block");
        assert_eq!(folders.len(), 2);

        let second = folders.get_object("1").expect("the second library");
        assert_eq!(second.get_str("path"), Some(r"I:\SteamLibrary"));
        assert_eq!(
            second
                .get_object("apps")
                .and_then(|apps| apps.get_u64("223710")),
            Some(4_695_601_334)
        );
    }

    #[test]
    fn parses_the_old_flat_libraryfolders_format() {
        // Before Steam moved to one block per library, a library was a bare
        // numbered key whose value was the path.
        let input = r#"
"LibraryFolders"
{
	"TimeNextStatsReport"		"1234567890"
	"ContentStatsID"		"-1234567890123456789"
	"1"		"D:\\SteamLibrary"
	"2"		"E:\\Games\\Steam"
}
"#;
        let root = parse(input).unwrap();
        let folders = root.get_object("libraryfolders").unwrap();
        assert_eq!(folders.get_str("1"), Some(r"D:\SteamLibrary"));
        assert_eq!(folders.get_str("2"), Some(r"E:\Games\Steam"));
    }

    #[test]
    fn parses_an_app_manifest() {
        let input = r#"
"AppState"
{
	"appid"		"223710"
	"Universe"		"1"
	"name"		"Cry of Fear"
	"StateFlags"		"4"
	"installdir"		"Cry of Fear"
	"LastUpdated"		"1648000000"
	"SizeOnDisk"		"4695601334"
	"buildid"		"5821301"
	"InstalledDepots"
	{
		"223711"
		{
			"manifest"		"3465817078078599269"
			"size"		"4695601334"
		}
	}
}
"#;
        let root = parse(input).unwrap();
        let state = root.get_object("AppState").unwrap();
        assert_eq!(state.get_str("installdir"), Some("Cry of Fear"));
        assert_eq!(state.get_u64("buildid"), Some(5_821_301));
        assert_eq!(state.get_u64("SizeOnDisk"), Some(4_695_601_334));
        assert_eq!(
            state.path(&["InstalledDepots", "223711", "manifest"]),
            Some(&Value::String("3465817078078599269".to_owned()))
        );
    }

    #[test]
    fn lookup_ignores_case() {
        let root = parse(r#""AppState" { "InstallDir" "Cry of Fear" }"#).unwrap();
        let state = root.get_object("appstate").unwrap();
        assert_eq!(state.get_str("INSTALLDIR"), Some("Cry of Fear"));
    }

    #[test]
    fn comments_are_ignored() {
        let input = r#"
// a leading comment
"root"
{
	// a comment inside the block
	"a"	"1"	// a trailing comment
	"b"	"2"
}
"#;
        let root = parse(input).unwrap().get_object("root").cloned().unwrap();
        assert_eq!(root.len(), 2);
        assert_eq!(root.get_str("b"), Some("2"));
    }

    #[test]
    fn a_slash_inside_a_quoted_value_is_not_a_comment() {
        let root = parse(r#""root" { "path" "c:/steam//apps" }"#).unwrap();
        assert_eq!(
            root.get_object("root").unwrap().get_str("path"),
            Some("c:/steam//apps")
        );
    }

    #[test]
    fn conditionals_are_dropped() {
        let input = r#"
"root"
{
	"a"	"win"	[$WIN32]
	"b"	"other"	[!$WIN32]
}
"#;
        let root = parse(input).unwrap();
        let block = root.get_object("root").unwrap();
        assert_eq!(block.get_str("a"), Some("win"));
        assert_eq!(block.get_str("b"), Some("other"));
    }

    #[test]
    fn unquoted_tokens_are_accepted() {
        let root = parse("root { appid 223710 name Cry }").unwrap();
        let block = root.get_object("root").unwrap();
        assert_eq!(block.get_u64("appid"), Some(223_710));
        assert_eq!(block.get_str("name"), Some("Cry"));
    }

    #[test]
    fn escapes_are_resolved() {
        let root = parse(r#""root" { "a" "x\\y" "b" "say \"hi\"" "c" "one\ttwo" }"#).unwrap();
        let block = root.get_object("root").unwrap();
        assert_eq!(block.get_str("a"), Some(r"x\y"));
        assert_eq!(block.get_str("b"), Some(r#"say "hi""#));
        assert_eq!(block.get_str("c"), Some("one\ttwo"));
    }

    #[test]
    fn an_unknown_escape_keeps_both_characters() {
        // Some third party tools write Windows paths unescaped.
        let root = parse(r#""root" { "path" "C:\steam" }"#).unwrap();
        assert_eq!(
            root.get_object("root").unwrap().get_str("path"),
            Some(r"C:\steam")
        );
    }

    #[test]
    fn non_ascii_values_survive() {
        let input = "\"root\" { \"path\" \"D:\\\\Librería\\\\游戏\" \"name\" \"Поиск\" }";
        let root = parse(input).unwrap();
        let block = root.get_object("root").unwrap();
        assert_eq!(block.get_str("path"), Some(r"D:\Librería\游戏"));
        assert_eq!(block.get_str("name"), Some("Поиск"));
    }

    #[test]
    fn a_utf8_bom_is_skipped() {
        let root = parse("\u{feff}\"root\" { \"a\" \"1\" }").unwrap();
        assert!(root.get_object("root").is_some());
    }

    #[test]
    fn duplicate_keys_are_all_kept() {
        let root = parse(r#""root" { "a" "1" "a" "2" }"#).unwrap();
        let block = root.get_object("root").unwrap();
        assert_eq!(block.get_str("a"), Some("1"));
        let all: Vec<_> = block
            .get_all("a")
            .filter_map(|value| value.as_str())
            .collect();
        assert_eq!(all, vec!["1", "2"]);
    }

    #[test]
    fn an_empty_document_is_an_empty_object() {
        assert!(parse("").unwrap().is_empty());
        assert!(parse("   \n // nothing here \n").unwrap().is_empty());
    }

    #[test]
    fn an_unterminated_block_is_reported() {
        let error = parse(r#""root" { "a" "1" "#).unwrap_err();
        assert_eq!(error.kind, KvErrorKind::UnterminatedBlock);
    }

    #[test]
    fn an_unterminated_string_is_reported_at_its_opening_quote() {
        let error = parse("\"root\"\n{\n\t\"a\" \"unclosed\n}\n").unwrap_err();
        assert_eq!(error.kind, KvErrorKind::UnterminatedString);
        assert_eq!(error.line, 3);
    }

    #[test]
    fn a_stray_closing_brace_is_reported() {
        let error = parse(r#""root" { "a" "1" } }"#).unwrap_err();
        assert_eq!(error.kind, KvErrorKind::UnexpectedBrace);
    }

    #[test]
    fn a_key_without_a_value_is_reported() {
        let error = parse(r#""root" { "a" }"#).unwrap_err();
        assert_eq!(error.kind, KvErrorKind::UnexpectedBrace);
    }

    #[test]
    fn deep_nesting_is_refused_rather_than_overflowing_the_stack() {
        let depth = MAX_DEPTH + 10;
        let mut input = String::from("\"root\"\n");
        for _ in 0..depth {
            input.push_str("{ \"k\"\n");
        }
        input.push_str("\"v\"\n");
        for _ in 0..depth {
            input.push_str("}\n");
        }
        let error = parse(&input).unwrap_err();
        assert_eq!(error.kind, KvErrorKind::TooDeep(MAX_DEPTH));
    }

    #[test]
    fn crlf_line_endings_parse() {
        let root = parse("\"root\"\r\n{\r\n\t\"a\"\t\"1\"\r\n}\r\n").unwrap();
        assert_eq!(root.get_object("root").unwrap().get_str("a"), Some("1"));
    }

    #[test]
    fn invalid_utf8_does_not_panic() {
        // A Latin-1 byte that is not valid UTF-8, as a mangled file might
        // hold. It must come back as a replacement character, not a panic.
        let bytes = b"\"root\" { \"a\" \"caf\xe9\" }";
        let input = String::from_utf8_lossy(bytes);
        let root = parse(&input).unwrap();
        assert!(root.get_object("root").unwrap().get_str("a").is_some());
    }
}
