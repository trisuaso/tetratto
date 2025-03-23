use pathbufd::PathBufD;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string},
    sync::{LazyLock, RwLock},
};

pub static ENGLISH_US: LazyLock<RwLock<LangFile>> = LazyLock::new(RwLock::default);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangFile {
    pub name: String,
    pub version: String,
    pub data: HashMap<String, String>,
}

impl Default for LangFile {
    fn default() -> Self {
        Self {
            name: "com.tettrato.langs.testing:aa-BB".to_string(),
            version: "0.0.0".to_string(),
            data: HashMap::new(),
        }
    }
}

impl LangFile {
    /// Check if a value exists in `data` (and isn't empty)
    pub fn exists(&self, key: &str) -> bool {
        if let Some(value) = self.data.get(key) {
            if value.is_empty() {
                return false;
            }

            return true;
        }

        false
    }

    /// Get a value from `data`, returns an empty string if it doesn't exist
    pub fn get(&self, key: &str) -> String {
        if !self.exists(key) {
            if (self.name == "com.tettrato.langs.testing:aa-BB")
                | (self.name == "com.tettrato.langs.testing:en-US")
            {
                return key.to_string();
            } else {
                // load english instead
                let reader = ENGLISH_US
                    .read()
                    .expect("failed to pull reader for ENGLISH_US");
                return reader.get(key);
            }
        }

        self.data.get(key).unwrap().to_owned()
    }
}

/// Read the `langs` directory and return a [`Hashmap`] containing all files
pub fn read_langs() -> HashMap<String, LangFile> {
    let mut out = HashMap::new();

    let langs_dir = PathBufD::current().join("langs");
    if let Ok(files) = read_dir(langs_dir) {
        for file in files.into_iter() {
            if file.is_err() {
                continue;
            }

            let de: LangFile = match toml::from_str(&match read_to_string(file.unwrap().path()) {
                Ok(f) => f,
                Err(_) => continue,
            }) {
                Ok(de) => de,
                Err(_) => continue,
            };

            if de.name.ends_with("en-US") {
                let mut writer = ENGLISH_US
                    .write()
                    .expect("failed to pull writer for ENGLISH_US");
                *writer = de.clone();
                drop(writer);
            }

            out.insert(de.name.clone(), de);
        }
    }

    // return
    out
}
