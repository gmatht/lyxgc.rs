//! LaTeX tokenizer - replaces \\, $, $$ with tokens for regex processing.

use lazy_static::lazy_static;
use regex::Regex;

pub const START_MATH_CHAR: char = '\x01';
pub const END_MATH_CHAR: char = '\x02';
pub const C_BACKSLASH: char = '\x03';
pub const C_DOLLAR_SIGN: char = '\x04';

pub fn math_char_s() -> String {
    START_MATH_CHAR.to_string()
}
pub fn math_char_e() -> String {
    END_MATH_CHAR.to_string()
}

lazy_static! {
    static ref BACKSLASH_PAT: Regex = Regex::new(r"\\\\").unwrap();
    static ref DOLLAR_ESC_PAT: Regex = Regex::new(r"\\\$").unwrap();
}

pub fn tokenize(text: &str) -> String {
    let mut result = BACKSLASH_PAT
        .replace_all(text, C_BACKSLASH.to_string())
        .into_owned();
    result = DOLLAR_ESC_PAT
        .replace_all(&result, C_DOLLAR_SIGN.to_string())
        .into_owned();

    // Display math $$...$$
    let disp_math = Regex::new(r"\$\$([^\$]*)\$\$").unwrap();
    result = disp_math
        .replace_all(&result, |caps: &regex::Captures| {
            format!(
                "{}{}{}{}{}",
                START_MATH_CHAR,
                START_MATH_CHAR,
                caps.get(1).map_or("", |m| m.as_str()),
                END_MATH_CHAR,
                END_MATH_CHAR
            )
        })
        .into_owned();

    // Inline math $...$
    let inline_math = Regex::new(r"\$([^\$]+)\$").unwrap();
    result = inline_math
        .replace_all(&result, |caps: &regex::Captures| {
            format!(
                "{}{}{}",
                START_MATH_CHAR,
                caps.get(1).map_or("", |m| m.as_str()),
                END_MATH_CHAR
            )
        })
        .into_owned();

    result
}

pub fn detokenize(text: &str) -> String {
    let mut result = text.to_string();
    result = result.replace(C_DOLLAR_SIGN, r"\$");
    result = result.replace(C_BACKSLASH, r"\\");
    result = result.replace(START_MATH_CHAR, "$");
    result = result.replace(END_MATH_CHAR, "$");
    result
}

pub fn tokens_to_user(text: &str) -> String {
    detokenize(text)
}

pub fn num_newlines(s: &str) -> usize {
    s.matches('\n').count()
}
