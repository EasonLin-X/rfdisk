use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use crate::util::i18n::Lang;

pub(crate) fn load_language() -> Lang {
    read_language_config()
        .or_else(detect_system_language)
        .unwrap_or_default()
}

pub(crate) fn save_language(lang: Lang) -> io::Result<PathBuf> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, format!("lang={}\n", lang.as_config_value()))?;
    Ok(path)
}

fn read_language_config() -> Option<Lang> {
    let content = fs::read_to_string(config_path()).ok()?;
    content.lines().find_map(|line| {
        let line = line.trim();
        let value = line
            .strip_prefix("lang=")
            .or_else(|| line.strip_prefix("language="))?;
        Lang::parse(value.trim())
    })
}

fn detect_system_language() -> Option<Lang> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|key| env::var(key).ok())
        .find_map(|value| Lang::from_locale(&value))
}

fn config_path() -> PathBuf {
    config_dir().join("config")
}

fn config_dir() -> PathBuf {
    env::var_os("RFDISK_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("XDG_CONFIG_HOME").map(|base| Path::new(&base).join("rfdisk")))
        .or_else(|| env::var_os("HOME").map(|home| Path::new(&home).join(".config/rfdisk")))
        .or_else(|| env::var_os("APPDATA").map(|base| Path::new(&base).join("rfdisk")))
        .unwrap_or_else(|| PathBuf::from(".rfdisk"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_language_detects_chinese_locale() {
        assert_eq!(Lang::from_locale("zh_CN.UTF-8"), Some(Lang::ZhCn));
    }

    #[test]
    fn system_language_detects_english_locale() {
        assert_eq!(Lang::from_locale("en_US.UTF-8"), Some(Lang::En));
    }
}
