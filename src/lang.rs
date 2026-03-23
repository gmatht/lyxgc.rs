//! Load language rules from JSON. Shares py/lyxgc/lang/data/*.json.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::rules;
use crate::tokenizer;

#[derive(Debug, Deserialize)]
struct LangData {
    msgs: Option<HashMap<String, MsgValue>>,
    simple_rules: Option<Vec<Vec<String>>>,
    common_rules: Option<bool>,
    custom_rules: Option<Vec<Vec<serde_json::Value>>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MsgValue {
    Tuple([String; 2]),
    String(String),
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

fn get_data_path(rule_module: &str) -> std::path::PathBuf {
    let data_dir = std::env::var("LYXGC_DATA").ok().map(std::path::PathBuf::from)
        .or_else(|| {
            let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let repo = manifest.parent()?;
            Some(repo.join("py").join("lyxgc").join("lang").join("data"))
        })
        .unwrap_or_else(|| Path::new("py/lyxgc/lang/data").to_path_buf());
    data_dir.join(format!("{}.json", rule_module))
}

pub fn load_language(rule_module: &str) -> Vec<Vec<String>> {
    let path = get_data_path(rule_module);
    if !path.exists() {
        return vec![];
    }

    let data: LangData = serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap_or_default(),
    )
    .unwrap_or(LangData {
        msgs: None,
        simple_rules: None,
        common_rules: None,
        custom_rules: None,
    });

    let mut rules: Vec<Vec<String>> = vec![];

    if let Some(custom_rules) = data.custom_rules {
        for rule in custom_rules {
            let name = rule
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let regex = rule
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let special = rule
                .get(2)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let desc = rule
                .get(3)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
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
