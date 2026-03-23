//! Language registry: map LyX language names and locale codes to rule modules.

use std::collections::HashSet;

fn locale_modules() -> HashSet<&'static str> {
    [
        "af", "sq", "ar", "hy", "eu", "be", "br", "bg", "ca", "zh", "hr", "cs", "da",
        "dv", "eo", "et", "fa", "fi", "gl", "el", "he", "hi", "hu", "is", "id", "ia",
        "ga", "ja", "kk", "ko", "ku", "lo", "la", "lv", "lt", "dsb", "ms", "mr", "mn",
        "nb", "nn", "oc", "pl", "ro", "ru", "se", "sa", "gd", "sr", "sk", "sl",
        "sv", "ta", "te", "th", "tr", "tk", "uk", "hsb", "ur", "vi", "cop", "syc",
        "en", "fr", "de", "es", "it", "pt", "nl",
    ]
    .into_iter()
    .collect()
}

const LYX_TO_MODULE: &[(&str, &str)] = &[
    ("English", "en"),
    ("English (USA)", "en"),
    ("English (UK)", "en"),
    ("French", "fr"),
    ("French (Canada)", "fr"),
    ("German", "de"),
    ("Spanish", "es"),
    ("Italian", "it"),
    ("Portuguese", "pt"),
    ("Portuguese (Brazil)", "pt"),
    ("Dutch", "nl"),
];

pub fn resolve_language(lang: &str) -> Option<String> {
    let lang = lang.trim();
    if lang.is_empty() {
        return None;
    }

    for (lyx_name, module) in LYX_TO_MODULE {
        if lang == *lyx_name {
            return Some((*module).to_string());
        }
    }

    let lang_lower = lang.to_lowercase();
    for prefix in ["dsb", "hsb", "syc", "cop"] {
        if lang_lower.starts_with(prefix) {
            return Some(if locale_modules().contains(prefix) {
                prefix.to_string()
            } else {
                "generic".to_string()
            });
        }
    }
    if lang_lower.starts_with("nb") {
        return Some("nb".to_string());
    }
    if lang_lower.starts_with("nn") {
        return Some("nn".to_string());
    }

    let short = if lang_lower.len() >= 2 {
        &lang_lower[..2]
    } else {
        &lang_lower[..]
    };
    if locale_modules().contains(short) {
        return Some(short.to_string());
    }
    if short.len() == 2 && short.chars().all(|c| c.is_ascii_alphabetic()) {
        return Some("generic".to_string());
    }
    None
}
