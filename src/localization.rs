//! Small keyed localization table with English fallback and missing-key audit.

use macroquad_toolkit::data_loader::load_embedded_json_labeled;
use serde::Deserialize;
use std::collections::HashSet;

const LOCALES_JSON: &str = macroquad_toolkit::include_json_str!("../assets/data/localization.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    German,
    French,
}

impl Language {
    pub fn id(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::German => "de",
            Self::French => "fr",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "de" => Self::German,
            "fr" => Self::French,
            _ => Self::English,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocaleTable {
    pub en: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub de: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub fr: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Localizer {
    table: LocaleTable,
    language: Language,
    missing: HashSet<String>,
}

impl Localizer {
    pub fn load(language: Language) -> Result<Self, String> {
        let table = load_embedded_json_labeled("localization", LOCALES_JSON)?;
        Ok(Self {
            table,
            language,
            missing: HashSet::new(),
        })
    }

    pub fn english() -> Self {
        Self::load(Language::English).expect("embedded localization is valid")
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn text<'a>(&'a mut self, key: &'a str) -> &'a str {
        let localized = match self.language {
            Language::English => self.table.en.get(key),
            Language::German => self.table.de.get(key),
            Language::French => self.table.fr.get(key),
        };
        if let Some(text) = localized.or_else(|| self.table.en.get(key)) {
            return text;
        }
        self.missing.insert(key.to_owned());
        key
    }

    /// Read-only rendering lookup for immediate-mode UI. Missing keys still
    /// render the key itself; the mutable `text` API is used by audits that
    /// need to record diagnostics.
    pub fn display(&self, key: &str) -> String {
        match self.language {
            Language::English => self.table.en.get(key),
            Language::German => self.table.de.get(key),
            Language::French => self.table.fr.get(key),
        }
        .or_else(|| self.table.en.get(key))
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| key.to_owned())
    }

    pub fn layout_warnings(&self) -> Vec<String> {
        let table = match self.language {
            Language::English => &self.table.en,
            Language::German => &self.table.de,
            Language::French => &self.table.fr,
        };
        table
            .iter()
            .filter(|(_, text)| text.chars().count() > 72)
            .map(|(key, text)| format!("{key} is {} characters", text.chars().count()))
            .collect()
    }

    pub fn missing_keys(&self) -> impl Iterator<Item = &str> {
        self.missing.iter().map(String::as_str)
    }
}

pub fn font_fallbacks(language: Language) -> &'static [&'static str] {
    match language {
        Language::English => &["assets/fonts/english.ttf"],
        Language::German => &[
            "assets/fonts/latin_extended.ttf",
            "assets/fonts/english.ttf",
        ],
        Language::French => &[
            "assets/fonts/latin_extended.ttf",
            "assets/fonts/english.ttf",
        ],
    }
}

#[cfg(test)]
mod tests;
