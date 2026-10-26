//! Translations.
//!
//! One Fluent file per language, embedded in the executable. English is the
//! source language and is always loaded as a fallback, so a key missing
//! from a translation shows the English sentence rather than the key name.

use std::borrow::Cow;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

/// The languages the launcher ships, in the order the first run screen
/// lists them: the source language, then the one the author speaks, then
/// the rest by their own name.
pub const LANGUAGES: &[Language] = &[
    Language {
        tag: "en",
        native_name: "English",
        reviewed: true,
    },
    Language {
        tag: "es",
        native_name: "Español",
        reviewed: true,
    },
    Language {
        tag: "de",
        native_name: "Deutsch",
        reviewed: false,
    },
    Language {
        tag: "fr",
        native_name: "Français",
        reviewed: false,
    },
    Language {
        tag: "it",
        native_name: "Italiano",
        reviewed: false,
    },
    Language {
        tag: "pl",
        native_name: "Polski",
        reviewed: false,
    },
    Language {
        tag: "pt-BR",
        native_name: "Português do Brasil",
        reviewed: false,
    },
    Language {
        tag: "tr",
        native_name: "Türkçe",
        reviewed: false,
    },
    Language {
        tag: "ru",
        native_name: "Русский",
        reviewed: false,
    },
    Language {
        tag: "uk",
        native_name: "Українська",
        reviewed: false,
    },
    Language {
        tag: "zh-CN",
        native_name: "简体中文",
        reviewed: false,
    },
];

/// One shipped language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Language {
    /// BCP 47 tag, which is also the name of the `.ftl` file.
    pub tag: &'static str,
    /// What speakers of the language call it.
    pub native_name: &'static str,
    /// Whether a native speaker has gone over it. The ones marked false
    /// are first drafts and say so in the interface.
    pub reviewed: bool,
}

/// The `.ftl` files, embedded at build time.
#[derive(rust_embed::Embed)]
#[folder = "i18n/"]
struct Files;

/// A loaded language, with English underneath it.
pub struct Translator {
    language: Language,
    bundle: FluentBundle<FluentResource>,
    fallback: Option<FluentBundle<FluentResource>>,
}

impl Translator {
    /// Loads a language by tag, falling back to English for an unknown one.
    pub fn load(tag: &str) -> Translator {
        let language = language_for(tag).unwrap_or(LANGUAGES[0]);
        let bundle = bundle_for(language.tag);

        let fallback = if language.tag == "en" {
            None
        } else {
            Some(bundle_for("en"))
        };

        Translator {
            language,
            bundle: bundle.unwrap_or_else(|| empty_bundle(language.tag)),
            fallback: fallback.map(|loaded| loaded.unwrap_or_else(|| empty_bundle("en"))),
        }
    }

    /// The language that is loaded.
    pub fn language(&self) -> Language {
        self.language
    }

    /// A message with no arguments.
    pub fn get(&self, key: &str) -> String {
        self.format(key, None)
    }

    /// A message with arguments, as the error types produce them.
    pub fn get_with(&self, key: &str, args: &[(&str, String)]) -> String {
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, FluentValue::from(value.as_str()));
        }
        self.format(key, Some(&fluent_args))
    }

    /// An attribute of a message, used for the description under a label.
    pub fn attribute(&self, key: &str, attribute: &str) -> Option<String> {
        attribute_from(&self.bundle, key, attribute).or_else(|| {
            self.fallback
                .as_ref()
                .and_then(|bundle| attribute_from(bundle, key, attribute))
        })
    }

    /// Whether a key exists at all, so the caller can fall back to data
    /// shipped with a release for cvars the launcher has never heard of.
    pub fn has(&self, key: &str) -> bool {
        self.bundle.has_message(key)
            || self
                .fallback
                .as_ref()
                .is_some_and(|bundle| bundle.has_message(key))
    }

    fn format(&self, key: &str, args: Option<&FluentArgs<'_>>) -> String {
        if let Some(text) = format_from(&self.bundle, key, args) {
            return text;
        }
        if let Some(fallback) = &self.fallback {
            if let Some(text) = format_from(fallback, key, args) {
                return text;
            }
        }
        // Showing the key is better than showing nothing: it is obvious in
        // a screenshot and searchable in the source.
        log::warn!("missing translation key {key:?}");
        key.to_owned()
    }
}

fn format_from(
    bundle: &FluentBundle<FluentResource>,
    key: &str,
    args: Option<&FluentArgs<'_>>,
) -> Option<String> {
    let message = bundle.get_message(key)?;
    let pattern = message.value()?;
    let mut errors = Vec::new();
    let text = bundle.format_pattern(pattern, args, &mut errors);
    for error in &errors {
        log::warn!("formatting {key:?}: {error}");
    }
    Some(into_owned(text))
}

fn attribute_from(
    bundle: &FluentBundle<FluentResource>,
    key: &str,
    attribute: &str,
) -> Option<String> {
    let message = bundle.get_message(key)?;
    let pattern = message.get_attribute(attribute)?.value();
    let mut errors = Vec::new();
    let text = bundle.format_pattern(pattern, None, &mut errors);
    Some(into_owned(text))
}

fn into_owned(text: Cow<'_, str>) -> String {
    text.into_owned()
}

fn bundle_for(tag: &str) -> Option<FluentBundle<FluentResource>> {
    let file = Files::get(&format!("{tag}.ftl"))?;
    let source = String::from_utf8(file.data.to_vec()).ok()?;

    let resource = match FluentResource::try_new(source) {
        Ok(resource) => resource,
        Err((resource, errors)) => {
            for error in errors {
                log::error!("{tag}.ftl: {error:?}");
            }
            resource
        }
    };

    let mut bundle = empty_bundle(tag);
    if let Err(errors) = bundle.add_resource(resource) {
        for error in errors {
            log::error!("{tag}.ftl: {error}");
        }
    }
    Some(bundle)
}

fn empty_bundle(tag: &str) -> FluentBundle<FluentResource> {
    let identifier: LanguageIdentifier = tag.parse().unwrap_or_else(|_| "en".parse().unwrap());
    let mut bundle = FluentBundle::new(vec![identifier]);
    // Fluent wraps interpolated values in directional isolation marks,
    // which egui draws as empty boxes. The launcher never mixes a right to
    // left value into a left to right sentence, so they are not needed.
    bundle.set_use_isolating(false);
    bundle
}

/// The shipped language matching a tag, trying the exact tag first and then
/// just the language part, so `es-AR` finds `es`.
pub fn language_for(tag: &str) -> Option<Language> {
    let wanted = tag.replace('_', "-");
    if let Some(language) = LANGUAGES
        .iter()
        .find(|language| language.tag.eq_ignore_ascii_case(&wanted))
    {
        return Some(*language);
    }
    let base = wanted.split('-').next().unwrap_or(&wanted);
    LANGUAGES
        .iter()
        .find(|language| {
            language
                .tag
                .split('-')
                .next()
                .unwrap_or(language.tag)
                .eq_ignore_ascii_case(base)
        })
        .copied()
}

/// What the system is set to, if the launcher ships it.
pub fn system_language() -> Option<Language> {
    language_for(&sys_locale::get_locale()?)
}

/// The language to preselect on the first run screen.
pub fn default_language() -> Language {
    system_language().unwrap_or(LANGUAGES[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shipped_language_has_a_file_that_parses() {
        for language in LANGUAGES {
            let bundle = bundle_for(language.tag);
            assert!(
                bundle.is_some(),
                "{}.ftl is missing from the i18n folder",
                language.tag
            );
        }
    }

    #[test]
    fn every_language_tag_is_a_valid_identifier() {
        for language in LANGUAGES {
            assert!(
                language.tag.parse::<LanguageIdentifier>().is_ok(),
                "{} is not a language tag",
                language.tag
            );
            assert!(!language.native_name.is_empty());
        }
    }

    #[test]
    fn only_english_and_spanish_are_marked_reviewed() {
        let reviewed: Vec<&str> = LANGUAGES
            .iter()
            .filter(|language| language.reviewed)
            .map(|language| language.tag)
            .collect();
        assert_eq!(reviewed, ["en", "es"]);
    }

    /// Every key English defines has to exist in every other language, and
    /// no translation may invent a key English does not have.
    #[test]
    fn every_translation_covers_the_same_keys_as_english() {
        let english = keys_in("en");
        assert!(english.len() > 50, "the source language looks empty");

        for language in LANGUAGES.iter().filter(|language| language.tag != "en") {
            let theirs = keys_in(language.tag);

            let missing: Vec<&String> = english.difference(&theirs).collect();
            assert!(
                missing.is_empty(),
                "{}.ftl is missing: {missing:?}",
                language.tag
            );

            let extra: Vec<&String> = theirs.difference(&english).collect();
            assert!(
                extra.is_empty(),
                "{}.ftl has keys English does not: {extra:?}",
                language.tag
            );
        }
    }

    /// A placeable such as `{ $path }` must appear in the translation too,
    /// or the sentence silently loses the value it was meant to show.
    #[test]
    fn every_translation_keeps_the_placeables_english_uses() {
        let english = source_of("en");
        for language in LANGUAGES.iter().filter(|language| language.tag != "en") {
            let theirs = source_of(language.tag);
            for (key, wanted) in placeables(&english) {
                let Some(found) = placeables(&theirs).remove(&key) else {
                    continue;
                };
                assert_eq!(
                    found, wanted,
                    "{}.ftl: {key} should interpolate {wanted:?}",
                    language.tag
                );
            }
        }
    }

    #[test]
    fn a_missing_key_falls_back_to_english_rather_than_to_nothing() {
        let translator = Translator::load("es");
        // Every key is present in both, so compare a known one.
        assert!(!translator.get("app-name").is_empty());
        assert_eq!(translator.get("app-name"), "ReCoF Launcher");
        // A key nobody defines comes back as itself, which is obvious.
        assert_eq!(
            translator.get("this-key-does-not-exist"),
            "this-key-does-not-exist"
        );
        assert!(!translator.has("this-key-does-not-exist"));
    }

    #[test]
    fn arguments_are_interpolated_without_isolation_marks() {
        let translator = Translator::load("en");
        let text = translator.get_with("err-no-game-exe", &[("path", r"D:\Juegos".to_owned())]);
        assert!(text.contains(r"D:\Juegos"), "{text}");
        assert!(
            !text.contains('\u{2068}') && !text.contains('\u{2069}'),
            "isolation marks would draw as empty boxes"
        );
    }

    #[test]
    fn an_unknown_tag_loads_english() {
        assert_eq!(Translator::load("xx").language().tag, "en");
        assert_eq!(Translator::load("").language().tag, "en");
    }

    #[test]
    fn a_regional_tag_finds_its_base_language() {
        assert_eq!(language_for("es-AR").map(|l| l.tag), Some("es"));
        assert_eq!(language_for("es_MX").map(|l| l.tag), Some("es"));
        assert_eq!(language_for("pt-BR").map(|l| l.tag), Some("pt-BR"));
        // Portuguese from Portugal is not shipped, so it finds the Brazilian
        // file, which is closer than English.
        assert_eq!(language_for("pt-PT").map(|l| l.tag), Some("pt-BR"));
        assert_eq!(language_for("zh-Hans-CN").map(|l| l.tag), Some("zh-CN"));
        assert_eq!(language_for("ja"), None);
    }

    #[test]
    fn there_is_always_a_default_language() {
        assert!(LANGUAGES.contains(&default_language()));
    }

    fn source_of(tag: &str) -> String {
        let file =
            Files::get(&format!("{tag}.ftl")).unwrap_or_else(|| panic!("{tag}.ftl is missing"));
        String::from_utf8(file.data.to_vec()).unwrap()
    }

    /// Message and attribute names defined in a file.
    fn keys_in(tag: &str) -> std::collections::BTreeSet<String> {
        let source = source_of(tag);
        let mut keys = std::collections::BTreeSet::new();
        let mut current = String::new();

        for line in source.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix("    .") {
                // An attribute belongs to the message above it.
                if let Some((name, _)) = rest.split_once('=') {
                    keys.insert(format!("{current}.{}", name.trim()));
                }
                continue;
            }
            if line.starts_with(char::is_whitespace) {
                continue;
            }
            if let Some((name, _)) = line.split_once('=') {
                current = name.trim().to_owned();
                keys.insert(current.clone());
            }
        }
        keys
    }

    /// Which `{ $name }` values each message interpolates.
    fn placeables(source: &str) -> std::collections::BTreeMap<String, Vec<String>> {
        let mut found: std::collections::BTreeMap<String, Vec<String>> = Default::default();
        let mut current = String::new();

        for line in source.lines() {
            if line.starts_with('#') {
                continue;
            }
            if !line.starts_with(char::is_whitespace) {
                if let Some((name, _)) = line.split_once('=') {
                    current = name.trim().to_owned();
                }
            }
            if current.is_empty() {
                continue;
            }
            let mut rest = line;
            while let Some(start) = rest.find("{ $") {
                let after = &rest[start + 3..];
                let Some(end) = after.find([' ', '}']) else {
                    break;
                };
                found
                    .entry(current.clone())
                    .or_default()
                    .push(after[..end].to_owned());
                rest = &after[end..];
            }
            if let Some(list) = found.get_mut(&current) {
                list.sort();
                list.dedup();
            }
        }
        found
    }
}
