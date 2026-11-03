use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Language {
    En,
    Es,
    Fr,
    De,
    It,
    Pt,
}

impl Default for Language {
    fn default() -> Self {
        Language::Es
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl Language {
    pub const fn code(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Es => "es",
            Language::Fr => "fr",
            Language::De => "de",
            Language::It => "it",
            Language::Pt => "pt",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Language::En => "English",
            Language::Es => "Español",
            Language::Fr => "Français",
            Language::De => "Deutsch",
            Language::It => "Italiano",
            Language::Pt => "Português",
        }
    }

    pub const fn flag_file(self) -> &'static str {
        match self {
            Language::En => "gb.png",
            Language::Es => "es.png",
            Language::Fr => "fr.png",
            Language::De => "de.png",
            Language::It => "it.png",
            Language::Pt => "pt.png",
        }
    }

    pub const fn all() -> [Language; 6] {
        [
            Language::En,
            Language::Es,
            Language::Fr,
            Language::De,
            Language::It,
            Language::Pt,
        ]
    }

    pub fn from_code(code: &str) -> Option<Language> {
        match code {
            "en" => Some(Language::En),
            "es" => Some(Language::Es),
            "fr" => Some(Language::Fr),
            "de" => Some(Language::De),
            "it" => Some(Language::It),
            "pt" => Some(Language::Pt),
            _ => None,
        }
    }
}

static TRANSLATIONS: Lazy<Mutex<HashMap<Language, HashMap<String, String>>>> =
    Lazy::new(|| {
        let mut map = HashMap::new();

        macro_rules! load_lang {
            ($lang:expr, $code:literal) => {
                let json = include_str!(concat!("../translations/", $code, ".json"));
                if let Ok(translations) =
                    serde_json::from_str::<HashMap<String, String>>(json)
                {
                    map.insert($lang, translations);
                }
            };
        }

        load_lang!(Language::En, "en");
        load_lang!(Language::Es, "es");
        load_lang!(Language::Fr, "fr");
        load_lang!(Language::De, "de");
        load_lang!(Language::It, "it");
        load_lang!(Language::Pt, "pt");

        Mutex::new(map)
    });

static CURRENT_LANG: Lazy<Mutex<Language>> = Lazy::new(|| Mutex::new(Language::Es));

pub fn set_language(lang: Language) {
    if let Ok(mut current) = CURRENT_LANG.lock() {
        *current = lang;
    }
}

pub fn get_language() -> Language {
    CURRENT_LANG.lock().map(|l| *l).unwrap_or(Language::Es)
}

pub fn t(key: &str) -> String {
    let lang = get_language();
    let translations = TRANSLATIONS.lock().unwrap();

    if let Some(lang_map) = translations.get(&lang) {
        if let Some(value) = lang_map.get(key) {
            return value.clone();
        }
    }

    if let Some(en_map) = translations.get(&Language::En) {
        if let Some(value) = en_map.get(key) {
            return value.clone();
        }
    }

    key.to_string()
}

#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::translations::t($key)
    };
}
