//! Load language rules from JSON. Shares py/lyxgc/lang/data/*.json.
//! If a JSON file is missing locally, it is downloaded from lyx-gc.py.

use miniserde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[cfg(any(feature = "download-ureq", feature = "download-curl"))]
const LANG_DATA_BASE_URL: &str =
    "https://raw.githubusercontent.com/gmatht/lyx-gc.py/refs/heads/master/lyxgc/lang/data";

use crate::rules;
use crate::tokenizer;

#[derive(Deserialize)]
struct LangData {
    custom_rules: Option<Vec<Vec<String>>>,
}

fn build_recursive_brace() -> String {
    let mut rb = r"\{[^{}]*\}".to_string();
    for _ in 0..4 {
        rb = format!(r"\{{(?:[^{{}}]|{})*}}", rb);
    }
    rb
}

fn build_placeholders() -> HashMap<String, String> {
    let (vowel_sound, consonant_sound) = rules::generate_vowel_regex_en();
    let start = tokenizer::START_MATH_CHAR;
    let end = tokenizer::END_MATH_CHAR;

    let mut m = HashMap::new();
    m.insert("{{LBS}}".to_string(), r"\\".to_string());
    m.insert("{{START_MATH_CHAR}}".to_string(), start.to_string());
    m.insert("{{END_MATH_CHAR}}".to_string(), end.to_string());
    m.insert(
        "{{MATHBLOCK}}".to_string(),
        format!("{}[^{}]*{}", start, end, end),
    );
    m.insert("{{PAR}}".to_string(), r"(?:\A|\n\s*\n|\Z)".to_string());
    m.insert("{{RECURSIVE_BRACE}}".to_string(), build_recursive_brace());
    m.insert("{{MACROBLOCK}}".to_string(), r"\\term\{[^}]*\}".to_string());
    m.insert(
        "{{NOTINMATH}}".to_string(),
        format!(
            r"(?![^{}]*{})",
            start,
            end
        ),
    );
    m.insert("{{VOWEL_SOUND_EN}}".to_string(), vowel_sound);
    m.insert("{{CONSONANT_SOUND_EN}}".to_string(), consonant_sound);
    m
}

lazy_static::lazy_static! {
    static ref PLACEHOLDERS: HashMap<String, String> = build_placeholders();
}

fn substitute_placeholders(s: &str) -> String {
    let mut result = s.to_string();
    for (placeholder, value) in PLACEHOLDERS.iter() {
        result = result.replace(placeholder, value);
    }
    result
}

#[cfg(any(feature = "download-ureq", feature = "download-curl"))]
fn is_safe_module_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[cfg(feature = "download-curl")]
fn try_download_lang_json(rule_module: &str) -> Option<String> {
    if !is_safe_module_name(rule_module) {
        return None;
    }
    let url = format!("{}/{}.json", LANG_DATA_BASE_URL, rule_module);
    let curl_cmd = if cfg!(target_os = "windows") { "curl.exe" } else { "curl" };
    std::process::Command::new(curl_cmd)
        .args(["-fsSL", &url])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
}

#[cfg(all(feature = "download-ureq", not(feature = "download-curl")))]
fn try_download_lang_json(rule_module: &str) -> Option<String> {
    if !is_safe_module_name(rule_module) {
        return None;
    }
    let url = format!("{}/{}.json", LANG_DATA_BASE_URL, rule_module);
    ureq::get(&url)
        .call()
        .ok()
        .and_then(|r| r.into_string().ok())
}

#[cfg(not(any(feature = "download-ureq", feature = "download-curl")))]
fn try_download_lang_json(_rule_module: &str) -> Option<String> {
    None
}

fn get_bundled_data_path(rule_module: &str) -> std::path::PathBuf {
    // Prefer the JSON shipped alongside the repository.
    // For distributed binaries without the `py/` directory, this path may not exist.
    let data_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|repo| Some(repo.join("py").join("lyxgc").join("lang").join("data")))
        .unwrap_or_else(|| Path::new("py/lyxgc/lang/data").to_path_buf());
    data_dir.join(format!("{}.json", rule_module))
}

fn get_cache_base_dir() -> std::path::PathBuf {
    // Cache directory for downloaded JSON.
    // Default:
    // - Unix: $XDG_CACHE_HOME/lyxgc, fallback to $HOME/.cache/lyxgc
    // - Windows: %LOCALAPPDATA%\\lyxgc
    //
    // Note: `LYXGC_DATA` overrides reads+writes entirely (handled in `load_language`).
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return Path::new(&local).join("lyxgc");
        }
        if let Ok(user) = std::env::var("USERPROFILE") {
            return Path::new(&user).join(".cache").join("lyxgc");
        }
        return Path::new(".").join("lyxgc_cache");
    }

    #[cfg(not(windows))]
    {
        if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
            return Path::new(&xdg).join("lyxgc");
        }
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(".cache").join("lyxgc");
        }
        return Path::new(".").join("lyxgc_cache");
    }
}

fn get_cache_data_path(rule_module: &str) -> std::path::PathBuf {
    get_cache_base_dir().join(format!("{}.json", rule_module))
}

fn get_data_path(rule_module: &str) -> std::path::PathBuf {
    // Single decision point for where `load_language()` reads/writes:
    // - If `LYXGC_DATA` is set: always use it for reads+writes.
    // - Otherwise: prefer bundled JSON if it exists, else use the OS cache location.
    if let Ok(override_dir) = std::env::var("LYXGC_DATA") {
        return Path::new(&override_dir).join(format!("{}.json", rule_module));
    }

    let bundled_path = get_bundled_data_path(rule_module);
    if bundled_path.exists() {
        bundled_path
    } else {
        get_cache_data_path(rule_module)
    }
}

pub fn load_language(rule_module: &str) -> Vec<Vec<String>> {
    let path = get_data_path(rule_module);
    let json_str = if path.exists() {
        std::fs::read_to_string(&path).unwrap_or_default()
    } else if let Some(body) = try_download_lang_json(rule_module) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, &body);
        body
    } else {
        String::new()
    };
    let data: LangData = miniserde::json::from_str(&json_str).unwrap_or(LangData {
        custom_rules: None,
    });

    let mut rules: Vec<Vec<String>> = vec![];

    if let Some(custom_rules) = data.custom_rules {
        for rule in custom_rules {
            let name = rule.get(0).cloned().unwrap_or_default();
            let regex = rule.get(1).cloned().unwrap_or_default();
            let special = rule.get(2).cloned().unwrap_or_default();
            let desc = rule.get(3).cloned().unwrap_or_default();
            rules.push(vec![
                name,
                substitute_placeholders(&regex),
                substitute_placeholders(&special),
                desc,
            ]);
        }
    }

    rules
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    #[cfg(not(windows))]
    fn cache_dir_uses_xdg_cache_home() {
        let _guard = ENV_LOCK.lock().unwrap();

        let prev_xdg = std::env::var("XDG_CACHE_HOME").ok();
        let prev_home = std::env::var("HOME").ok();

        std::env::set_var("XDG_CACHE_HOME", "/tmp/xdg_cache_home_test");
        std::env::remove_var("HOME");

        let base = get_cache_base_dir();
        let expected = Path::new("/tmp/xdg_cache_home_test").join("lyxgc");
        assert_eq!(base, expected);

        match prev_xdg {
            Some(v) => std::env::set_var("XDG_CACHE_HOME", v),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }
        match prev_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    #[cfg(not(windows))]
    fn cache_dir_falls_back_to_home_cache() {
        let _guard = ENV_LOCK.lock().unwrap();

        let prev_xdg = std::env::var("XDG_CACHE_HOME").ok();
        let prev_home = std::env::var("HOME").ok();

        std::env::remove_var("XDG_CACHE_HOME");
        std::env::set_var("HOME", "/tmp/home_cache_test");

        let base = get_cache_base_dir();
        let expected = Path::new("/tmp/home_cache_test").join(".cache").join("lyxgc");
        assert_eq!(base, expected);

        match prev_xdg {
            Some(v) => std::env::set_var("XDG_CACHE_HOME", v),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }
        match prev_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    #[cfg(windows)]
    fn cache_dir_uses_local_app_data() {
        let _guard = ENV_LOCK.lock().unwrap();

        let prev_local = std::env::var("LOCALAPPDATA").ok();
        let prev_user = std::env::var("USERPROFILE").ok();

        std::env::set_var("LOCALAPPDATA", r"C:\TempLocalAppData");
        std::env::remove_var("USERPROFILE");

        let base = get_cache_base_dir();
        let expected = Path::new(r"C:\TempLocalAppData").join("lyxgc");
        assert_eq!(base, expected);

        match prev_local {
            Some(v) => std::env::set_var("LOCALAPPDATA", v),
            None => std::env::remove_var("LOCALAPPDATA"),
        }
        match prev_user {
            Some(v) => std::env::set_var("USERPROFILE", v),
            None => std::env::remove_var("USERPROFILE"),
        }
    }
}
