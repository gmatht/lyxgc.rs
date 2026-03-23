//! Rule utilities: set_diff, simple_rule, generate_vowel_regex.

use std::collections::HashSet;

use crate::tokenizer::{END_MATH_CHAR, START_MATH_CHAR};

pub fn set_diff(a: &str, b: &str) -> String {
    let b_set: HashSet<char> = b.chars().collect();
    a.chars().filter(|c| !b_set.contains(c)).collect()
}

pub fn generate_vowel_regex_en() -> (String, String) {
    let endnumber = r"(?=[^0-9]|$)";
    let funnynumber = format!(
        r"(?:11|18)(?:[0-9]{{2}})?(?:[0-9]{{3}})*{}",
        endnumber
    );
    let vowelnumber = format!(r"\b(?:8[0-9]*{}|{})", endnumber, funnynumber);
    let consonantnumber = format!(
        r"\b(?![0-9]*(?:11|18)(?:[0-9]{{2}})?(?:[0-9]{{3}})*{})[012345679][0-9]*{}",
        endnumber, endnumber
    );

    let vowels_l = "aeiou";
    let vowels_u = "FHILMANXAEIOS";
    let vowels_d = "8";
    let consonants_l = set_diff("abcdefghijklmnopqrstuvwxyz", vowels_l);
    let consonants_u = set_diff("ABCDEFGHIJKLMNOPQRSTUVWXYZ", vowels_u);
    let consonants_d = set_diff("0123456789", vowels_d);

    let v_li: String = format!(
        "{}{}",
        vowels_l,
        vowels_l.to_uppercase()
    );
    let v_ui: String = format!(
        "{}{}",
        vowels_u,
        vowels_u.to_lowercase()
    );
    let c_li: String = format!(
        "{}{}",
        consonants_l,
        consonants_l.to_uppercase()
    );
    let c_ui: String = format!(
        "{}{}",
        consonants_u,
        consonants_u.to_lowercase()
    );

    let includewords = r"(?:MF|NP|NL|LP|MPC|RTL|RMS|heir|RME|ME|heirloom|honest|honor|honorable|honorarium|honorary|honorific|honour|hour|hourglass|hourly|HTML|XML|FBI|SGML|SDL|HAA|LTL|SAA|S5|FSA|SSPM)";
    let excludewords = r"(?:US[a-zA-Z]*|Eur[a-zA-Z]*|Unix|eurhythmic|eurhythmy|euripus|one|unary|US|usage|useful|user|UK|unanimous|utrees?|uni[a-zA-Z]*|util[a-zA-Z]*|usual)";

    let mathignorelist = [
        "frac", "hat", "acute", "bar", "dot",
        "check", "grave", "vec", "ddot", "breve", "tilde",
    ];
    let mathignore = mathignorelist
        .iter()
        .map(|m| format!(r"\\\\{}\s*\\{{ ", m))
        .collect::<Vec<_>>()
        .join(r"|\\");

    fn build(
        l: &str,
        u: &str,
        d: &str,
        number: &str,
        includewords: &str,
        excludewords: &str,
        mathignore: &str,
    ) -> String {
        let simple_word = format!(r"\b[{}][a-zA-Z0-9]+\b", l);
        let excluded_word = format!(r"\b{}\b", excludewords);
        let good_simple_word = format!("(?!{}){}", excluded_word, simple_word);
        let complex_word = format!(r"\b{}\b", includewords);
        let word = format!("(?:{}|{})", complex_word, good_simple_word);
        let letter = format!(r"\b[{}]\b", u);
        let math = format!(
            "{}(?:{}|[(])*(?:[{}]|\\[{}{}])[^{}]*{}",
            START_MATH_CHAR,
            mathignore,
            format!("{}{}", d, u),
            l,
            r"]",
            END_MATH_CHAR,
            END_MATH_CHAR
        );
        format!(
            "(?:{}|{}|{}|(?:\\\\\\$)?{})",
            word, letter, math, number
        )
    }

    let vowel_sound = build(
        &v_li,
        &v_ui,
        vowels_d,
        &vowelnumber,
        includewords,
        excludewords,
        &mathignore,
    );
    let consonant_sound = build(
        &c_li,
        &c_ui,
        &consonants_d,
        &consonantnumber,
        excludewords,
        includewords,
        &mathignore,
    );

    (vowel_sound, consonant_sound)
}
