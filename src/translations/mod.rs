use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static TRANSLATIONS: Lazy<Mutex<HashMap<String, HashMap<String, String>>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    macro_rules! load_lang {
        ($lang:literal) => {
            let json = include_str!(concat!("../translations/", $lang, ".json"));
            if let Ok(translations) = serde_json::from_str::<HashMap<String, String>>(json) {
                map.insert($lang.to_string(), translations);
            }
        };
    }
    
    load_lang!("es");
    load_lang!("en");
    load_lang!("fr");
    load_lang!("de");
    load_lang!("it");
    load_lang!("pt");
    
    Mutex::new(map)
});

static CURRENT_LANG: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("es".to_string()));

pub fn set_language(lang: &str) {
    if let Ok(mut current) = CURRENT_LANG.lock() {
        *current = lang.to_string();
    }
}

pub fn get_language() -> String {
    CURRENT_LANG.lock().map(|l| l.clone()).unwrap_or_else(|_| "es".to_string())
}

pub fn t(key: &str) -> String {
    let lang = get_language();
    let translations = TRANSLATIONS.lock().unwrap();
    
    if let Some(lang_map) = translations.get(&lang) {
        if let Some(value) = lang_map.get(key) {
            return value.clone();
        }
    }
    
    if let Some(en_map) = translations.get("en") {
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