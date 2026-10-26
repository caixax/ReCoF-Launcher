//! Valve's binary KeyValues, which is what `shortcuts.vdf` is.
//!
//! The format is a stream of typed entries:
//!
//! | Byte | Meaning                                              |
//! |------|------------------------------------------------------|
//! | 0x00 | a nested map: NUL terminated key, entries, then 0x08 |
//! | 0x01 | a string: NUL terminated key, NUL terminated value    |
//! | 0x02 | a 32 bit little endian integer: key, then four bytes  |
//! | 0x08 | end of the current map                               |
//!
//! Adding a non Steam game means reading this file, appending an entry
//! and writing it back, which is the only time the launcher writes
//! outside its own folders. It is therefore parsed strictly and written
//! back byte for byte faithfully: the round trip test reads a file,
//! writes it out again and compares.

/// A value in a binary KeyValues document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinValue {
    String(String),
    Int(i32),
    Map(BinMap),
}

impl BinValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            BinValue::String(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            BinValue::Int(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&BinMap> {
        match self {
            BinValue::Map(map) => Some(map),
            _ => None,
        }
    }
}

/// An ordered map, because Steam cares about the order of the numbered
/// shortcut entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BinMap {
    entries: Vec<(String, BinValue)>,
}

impl BinMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, key: impl Into<String>, value: BinValue) {
        self.entries.push((key.into(), value));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &BinValue)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_str(), value))
    }

    /// Lookup is case insensitive, as Valve's own reader is.
    pub fn get(&self, key: &str) -> Option<&BinValue> {
        self.entries
            .iter()
            .find(|(stored, _)| stored.eq_ignore_ascii_case(key))
            .map(|(_, value)| value)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.get(key)?.as_int()
    }

    pub fn get_map(&self, key: &str) -> Option<&BinMap> {
        self.get(key)?.as_map()
    }

    /// The next free numbered key, which is how a shortcut is appended.
    pub fn next_index(&self) -> String {
        let highest = self
            .entries
            .iter()
            .filter_map(|(key, _)| key.parse::<u32>().ok())
            .max();
        match highest {
            Some(highest) => (highest + 1).to_string(),
            None => "0".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BinError {
    #[error("the file ends in the middle of an entry")]
    Truncated,
    #[error("unknown entry type {0:#04x} at offset {1}")]
    UnknownType(u8, usize),
    #[error("a string is not valid UTF-8")]
    BadString,
    #[error("the document nests more than {0} levels deep")]
    TooDeep(usize),
}

/// Steam's own files are two levels deep. Anything far beyond that is not
/// a shortcuts file.
const MAX_DEPTH: usize = 32;

/// Parses a binary KeyValues document.
pub fn parse(bytes: &[u8]) -> Result<BinMap, BinError> {
    let mut reader = Reader { bytes, at: 0 };
    let map = reader.map(0)?;
    Ok(map)
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn map(&mut self, depth: usize) -> Result<BinMap, BinError> {
        if depth >= MAX_DEPTH {
            return Err(BinError::TooDeep(MAX_DEPTH));
        }
        let mut map = BinMap::new();

        loop {
            // The outermost map ends at the end of the file, which is how
            // Steam writes it: no trailing 0x08 is required.
            let Some(kind) = self.byte() else {
                if depth == 0 {
                    return Ok(map);
                }
                return Err(BinError::Truncated);
            };

            match kind {
                0x08 => return Ok(map),
                0x00 => {
                    let key = self.string()?;
                    let nested = self.map(depth + 1)?;
                    map.push(key, BinValue::Map(nested));
                }
                0x01 => {
                    let key = self.string()?;
                    let value = self.string()?;
                    map.push(key, BinValue::String(value));
                }
                0x02 => {
                    let key = self.string()?;
                    let value = self.int()?;
                    map.push(key, BinValue::Int(value));
                }
                other => return Err(BinError::UnknownType(other, self.at - 1)),
            }
        }
    }

    fn byte(&mut self) -> Option<u8> {
        let byte = self.bytes.get(self.at).copied()?;
        self.at += 1;
        Some(byte)
    }

    fn string(&mut self) -> Result<String, BinError> {
        let start = self.at;
        while self.at < self.bytes.len() && self.bytes[self.at] != 0 {
            self.at += 1;
        }
        if self.at >= self.bytes.len() {
            return Err(BinError::Truncated);
        }
        let text =
            std::str::from_utf8(&self.bytes[start..self.at]).map_err(|_| BinError::BadString)?;
        self.at += 1; // the NUL
        Ok(text.to_owned())
    }

    fn int(&mut self) -> Result<i32, BinError> {
        if self.at + 4 > self.bytes.len() {
            return Err(BinError::Truncated);
        }
        let value = i32::from_le_bytes([
            self.bytes[self.at],
            self.bytes[self.at + 1],
            self.bytes[self.at + 2],
            self.bytes[self.at + 3],
        ]);
        self.at += 4;
        Ok(value)
    }
}

/// Writes a document back out.
///
/// The outermost map gets a trailing 0x08, which is what Steam writes.
pub fn write(map: &BinMap) -> Vec<u8> {
    let mut out = Vec::new();
    write_entries(map, &mut out);
    out.push(0x08);
    out
}

fn write_entries(map: &BinMap, out: &mut Vec<u8>) {
    for (key, value) in map.iter() {
        match value {
            BinValue::Map(nested) => {
                out.push(0x00);
                write_cstring(key, out);
                write_entries(nested, out);
                out.push(0x08);
            }
            BinValue::String(text) => {
                out.push(0x01);
                write_cstring(key, out);
                write_cstring(text, out);
            }
            BinValue::Int(number) => {
                out.push(0x02);
                write_cstring(key, out);
                out.extend_from_slice(&number.to_le_bytes());
            }
        }
    }
}

fn write_cstring(text: &str, out: &mut Vec<u8>) {
    // A NUL inside a value would truncate it on the way back in, so it is
    // dropped rather than written.
    for byte in text.bytes().filter(|byte| *byte != 0) {
        out.push(byte);
    }
    out.push(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shortcuts.vdf with one entry, built the way Steam writes it.
    fn sample() -> Vec<u8> {
        let mut shortcut = BinMap::new();
        shortcut.push("appid", BinValue::Int(-1_234_567_890));
        shortcut.push("AppName", BinValue::String("Half-Life".to_owned()));
        shortcut.push("Exe", BinValue::String("\"C:\\Games\\hl.exe\"".to_owned()));
        shortcut.push("StartDir", BinValue::String("\"C:\\Games\\\"".to_owned()));
        shortcut.push("icon", BinValue::String(String::new()));
        shortcut.push("ShortcutPath", BinValue::String(String::new()));
        shortcut.push("LaunchOptions", BinValue::String(String::new()));
        shortcut.push("IsHidden", BinValue::Int(0));
        shortcut.push("AllowDesktopConfig", BinValue::Int(1));
        shortcut.push("AllowOverlay", BinValue::Int(1));
        shortcut.push("OpenVR", BinValue::Int(0));
        shortcut.push("Devkit", BinValue::Int(0));
        shortcut.push("DevkitGameID", BinValue::String(String::new()));
        shortcut.push("DevkitOverrideAppID", BinValue::Int(0));
        shortcut.push("LastPlayTime", BinValue::Int(0));
        shortcut.push("tags", BinValue::Map(BinMap::new()));

        let mut zero = BinMap::new();
        zero.push("0", BinValue::Map(shortcut));

        let mut root = BinMap::new();
        root.push("shortcuts", BinValue::Map(zero));
        write(&root)
    }

    #[test]
    fn a_shortcuts_file_round_trips_byte_for_byte() {
        let bytes = sample();
        let parsed = parse(&bytes).expect("a well formed file parses");
        assert_eq!(write(&parsed), bytes, "writing back must be faithful");
    }

    #[test]
    fn the_entries_come_back_with_their_types() {
        let parsed = parse(&sample()).unwrap();
        let shortcuts = parsed.get_map("shortcuts").expect("the root block");
        let first = shortcuts.get_map("0").expect("the first shortcut");

        assert_eq!(first.get_str("AppName"), Some("Half-Life"));
        assert_eq!(first.get_int("appid"), Some(-1_234_567_890));
        assert_eq!(first.get_int("AllowOverlay"), Some(1));
        assert_eq!(first.get_str("icon"), Some(""));
        assert!(first.get_map("tags").unwrap().is_empty());
    }

    #[test]
    fn lookup_ignores_case_as_valves_reader_does() {
        let parsed = parse(&sample()).unwrap();
        let shortcuts = parsed.get_map("SHORTCUTS").unwrap();
        assert!(shortcuts.get_map("0").unwrap().get_str("appname").is_some());
    }

    #[test]
    fn an_empty_file_parses_as_an_empty_document() {
        assert_eq!(parse(&[]).unwrap(), BinMap::new());
        // Steam writes a lone terminator for a file with no shortcuts.
        assert_eq!(parse(&[0x08]).unwrap(), BinMap::new());
    }

    #[test]
    fn the_next_index_follows_the_highest_one_used() {
        let mut map = BinMap::new();
        assert_eq!(map.next_index(), "0");

        map.push("0", BinValue::Map(BinMap::new()));
        map.push("1", BinValue::Map(BinMap::new()));
        assert_eq!(map.next_index(), "2");

        // A gap does not make the launcher reuse a number.
        let mut sparse = BinMap::new();
        sparse.push("0", BinValue::Map(BinMap::new()));
        sparse.push("7", BinValue::Map(BinMap::new()));
        assert_eq!(sparse.next_index(), "8");

        // A key that is not a number is ignored for this.
        let mut odd = BinMap::new();
        odd.push("not a number", BinValue::Int(1));
        assert_eq!(odd.next_index(), "0");
    }

    #[test]
    fn non_ascii_names_and_paths_survive_the_round_trip() {
        let mut shortcut = BinMap::new();
        shortcut.push("AppName", BinValue::String("ReCoF ñ 游戏".to_owned()));
        shortcut.push(
            "Exe",
            BinValue::String("\"D:\\Librería\\ReCoF\\cof.exe\"".to_owned()),
        );
        let mut zero = BinMap::new();
        zero.push("0", BinValue::Map(shortcut));
        let mut root = BinMap::new();
        root.push("shortcuts", BinValue::Map(zero));

        let bytes = write(&root);
        let parsed = parse(&bytes).unwrap();
        assert_eq!(
            parsed
                .get_map("shortcuts")
                .unwrap()
                .get_map("0")
                .unwrap()
                .get_str("AppName"),
            Some("ReCoF ñ 游戏")
        );
        assert_eq!(write(&parsed), bytes);
    }

    #[test]
    fn a_truncated_file_is_refused_rather_than_half_read() {
        let bytes = sample();
        for cut in [1, 5, 20, bytes.len() / 2, bytes.len() - 2] {
            let outcome = parse(&bytes[..cut]);
            assert!(outcome.is_err(), "a file cut at {cut} should be refused");
        }
    }

    #[test]
    fn an_unknown_entry_type_is_refused() {
        let bytes = [0x07u8, b'k', 0x00];
        assert!(matches!(parse(&bytes), Err(BinError::UnknownType(0x07, 0))));
    }

    #[test]
    fn a_string_that_is_not_utf8_is_refused() {
        // Type 0x01, key "k", then a value with an invalid byte.
        let bytes = [0x01u8, b'k', 0x00, 0xff, 0xfe, 0x00];
        assert_eq!(parse(&bytes), Err(BinError::BadString));
    }

    #[test]
    fn absurd_nesting_is_refused_rather_than_overflowing_the_stack() {
        let mut bytes = Vec::new();
        for _ in 0..(MAX_DEPTH + 5) {
            bytes.push(0x00);
            bytes.extend_from_slice(b"k\0");
        }
        assert!(matches!(parse(&bytes), Err(BinError::TooDeep(_))));
    }

    #[test]
    fn a_nul_inside_a_value_is_dropped_rather_than_truncating_the_file() {
        let mut map = BinMap::new();
        map.push("k", BinValue::String("a\0b".to_owned()));
        let bytes = write(&map);
        assert_eq!(parse(&bytes).unwrap().get_str("k"), Some("ab"));
    }
}
