//! Valve KeyValues.
//!
//! Steam stores everything the launcher needs to find the game in this
//! format: `libraryfolders.vdf`, `appmanifest_223710.acf` and, for the
//! optional non Steam shortcut, the binary `shortcuts.vdf`.
//!
//! The text parser is written here rather than taken from a crate because
//! the files are load bearing (getting the library list wrong means not
//! finding the game) and because the dialect Steam writes is small and well
//! covered by the tests next to the code.

pub mod text;

use std::fmt;

pub use text::{parse, KvError};

/// A value in a KeyValues document: either a leaf string or a block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Object(Object),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(text) => Some(text),
            Value::Object(_) => None,
        }
    }

    pub fn as_object(&self) -> Option<&Object> {
        match self {
            Value::Object(object) => Some(object),
            Value::String(_) => None,
        }
    }

    /// Parses the leaf as an unsigned number. Steam writes every number as a
    /// quoted string.
    pub fn as_u64(&self) -> Option<u64> {
        self.as_str()?.trim().parse().ok()
    }
}

/// A block of key and value pairs.
///
/// Order is kept, duplicate keys are kept, and lookup is case insensitive,
/// all three to match how Steam's own reader behaves.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Object {
    entries: Vec<(String, Value)>,
}

impl Object {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, key: impl Into<String>, value: Value) {
        self.entries.push((key.into(), value));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &Value)> {
        self.entries
            .iter()
            .map(|(key, value)| (key.as_str(), value))
    }

    /// The first value stored under `key`, compared without case.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries
            .iter()
            .find(|(stored, _)| stored.eq_ignore_ascii_case(key))
            .map(|(_, value)| value)
    }

    /// Every value stored under `key`.
    pub fn get_all<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a Value> + 'a {
        self.entries
            .iter()
            .filter(move |(stored, _)| stored.eq_ignore_ascii_case(key))
            .map(|(_, value)| value)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

    pub fn get_object(&self, key: &str) -> Option<&Object> {
        self.get(key)?.as_object()
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get(key)?.as_u64()
    }

    /// Follows a path of keys, for example `["libraryfolders", "0", "apps"]`.
    pub fn path(&self, keys: &[&str]) -> Option<&Value> {
        let mut current = self.get(keys.first()?)?;
        for key in &keys[1..] {
            current = current.as_object()?.get(key)?;
        }
        Some(current)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(text) => write!(formatter, "{text:?}"),
            Value::Object(object) => write!(formatter, "{{{} entries}}", object.len()),
        }
    }
}
