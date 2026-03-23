//! Engine: find_errors - applies rules to LaTeX text.

use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::io::Write;

use crate::report::report_error;
use crate::tokenizer::{num_newlines, tokenize, tokens_to_user};

fn count_brackets(regex_str: &str) -> usize {
    let mut n = 0;
    let mut in_char_class = false;
    let mut i = 0;
    let chars: Vec<char> = regex_str.chars().collect();
    while i < chars.len() {
        let c = chars[i];
        if c == '[' && (i == 0 || chars[i - 1] != '\\') {
            in_char_class = true;
        } else if c == ']' && in_char_class {
            in_char_class = false;
        } else if !in_char_class && c == '(' && (i + 1 >= chars.len() || chars[i + 1] != '?') {
            n += 1;
        }
        i += 1;
    }
    n
}

pub struct CompiledRule {
    pub name: String,
    pub special: String,
    pub desc: String,
    pub pattern: FancyRegex,
    pub n_brackets: usize,
}

/// Compile raw rules into CompiledRule. Expensive (~70ms for 266 rules).
pub fn compile_rules(rules: &[Vec<String>]) -> Vec<CompiledRule> {
    let mut compiled = vec![];
    for rule in rules {
        let (name, regex_str, special, desc) = (
            rule.get(0).cloned().unwrap_or_default(),
            rule.get(1).cloned().unwrap_or_default(),
            rule.get(2).cloned().unwrap_or_default(),
            rule.get(3).cloned().unwrap_or_default(),
        );
        let clean_regex = regex_str.replace("(?i)", "");
        let icase = regex_str.contains("(?i)");
        let wrap_regex = format!("({})", clean_regex);
        let pattern = if icase {
            let wrapped = format!("(?i){}", wrap_regex);
            FancyRegex::new(&wrapped).unwrap_or_else(|_| FancyRegex::new("$^").unwrap())
        } else {
            FancyRegex::new(&wrap_regex).unwrap_or_else(|_| FancyRegex::new("$^").unwrap())
        };
        let n_brackets = count_brackets(&wrap_regex);
        compiled.push(CompiledRule {
            name,
            special,
            desc,
            pattern,
            n_brackets,
        });
    }
    compiled
}

/// Run rules with pre-compiled patterns. Skips regex compilation (~70ms).
pub fn find_errors_compiled(
    compiled: &[CompiledRule],
    out: &mut dyn Write,
    filetext: &str,
    filename: &str,
    output_format: &str,
    bench_internal: bool,
) -> std::io::Result<usize> {
    let mut n_errors = 0usize;
    let mut prev_newlines = 0usize;

    let t_comment = bench_internal.then(std::time::Instant::now);
    let comment_re = FancyRegex::new(r"(?<!\\)%.*(?:\$|\n)").unwrap_or_else(|_| FancyRegex::new("$^").unwrap());
    let mut filetext = comment_re.replace_all(filetext, "%\n").to_string();
    filetext = tokenize(&filetext);
    if let Some(t) = t_comment {
        eprintln!("[bench] comment_remove + tokenize: {:?}", t.elapsed());
    }

    let doc_split = Regex::new(r"\\begin\{document\}").unwrap_or_else(|_| Regex::new("$^").unwrap());
    let parts: Vec<String> = doc_split.split(&filetext).map(|s| s.to_string()).collect();
    if parts.len() > 1 {
        prev_newlines = num_newlines(&parts[0]);
        if parts.len() > 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "More than one \\begin{document} in file",
            ));
        }
        filetext = parts[1].clone();
    }

    run_compiled_rules(compiled, &filetext, prev_newlines, out, filename, output_format, bench_internal)
}

fn run_compiled_rules(
    compiled: &[CompiledRule],
    filetext: &str,
    prev_newlines: usize,
    out: &mut dyn Write,
    filename: &str,
    output_format: &str,
    bench_internal: bool,
) -> std::io::Result<usize> {
    let mut n_errors = 0usize;

    let par_pat = Regex::new(r"(?:^|\n\s*\n|\Z)").unwrap_or_else(|_| Regex::new("$^").unwrap());
    let pars: Vec<&str> = par_pat.split(filetext).collect();
    let mut old_pars: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut linenum = 1 + prev_newlines;
    for partext in &pars {
        if partext.len() > 80 && !partext.trim().is_empty() {
            if let Some(&line) = old_pars.get(*partext) {
                report_error(out, line, 1, "667", &format!("Duplicated paragraph {}", line), "", "", "", "", filename, output_format)?;
                n_errors += 1;
            } else {
                old_pars.insert((*partext).to_string(), linenum);
            }
        }
        linenum += num_newlines(partext) + 2;
    }

    let blocktext = filetext.to_string();
    let t_match = bench_internal.then(std::time::Instant::now);
    for rule in compiled {
        let mut blocktext_ = blocktext.clone();
        if rule.special.starts_with("erase:") {
            let erase_pat = &rule.special[6..];
            if let Ok(re) = FancyRegex::new(erase_pat) {
                blocktext_ = re.replace_all(&blocktext_, "").to_string();
            }
        }
        let mut offset = 0;
        let mut linenum = 1 + prev_newlines;
        for cap_res in rule.pattern.captures_iter(&blocktext_) {
            let cap = match cap_res {
                Ok(c) => c,
                _ => continue,
            };
            let (start, end) = {
                let m = cap.get(0).unwrap();
                (m.start(), m.end())
            };
            let trigger_text = cap.get(1).map_or(cap.get(0).unwrap().as_str(), |m| m.as_str());
            let before_text = &blocktext_[offset..start];
            let merged = format!("{}{}", before_text, trigger_text);
            linenum = linenum + num_newlines(&merged);
            let trigger_user = tokens_to_user(trigger_text);
            let amount = (35 as i32 - trigger_user.len() as i32).max(0) as usize;
            let before_tail = if amount > 0 {
                let chars: Vec<char> = before_text.chars().collect();
                let start = chars.len().saturating_sub(amount);
                chars[start..].iter().collect::<String>()
            } else {
                String::new()
            };
            let context_before = format!("...{}", tokens_to_user(&before_tail));
            let after_text = &blocktext_[end..];
            let context_after = tokens_to_user(&after_text.chars().take(amount).collect::<String>());
            let error_context = format!("{}{}{}..", context_before, trigger_user, context_after);
            let spaces = " ".repeat(context_before.len());
            let rule_ptrs = if trigger_user.is_empty() { "^".to_string() } else { format!("{}{}", spaces, "^".repeat(trigger_user.len())) };
            let mut this_desc = rule.desc.clone();
            for n in 1..=rule.n_brackets {
                if let Some(m) = cap.get(n) {
                    let arg = m.as_str();
                    this_desc = this_desc.replace(&format!("ARG{n}.CAP"), &format!("\"{}\" does not appear to be a name", arg));
                    this_desc = this_desc.replace(&format!("ARG{n}"), &format!("\"{}\"", arg));
                }
            }
            report_error(out, linenum, 1, "666", &rule.name, &this_desc, trigger_text, &error_context, &rule_ptrs, filename, output_format)?;
            n_errors += 1;
            offset = end;
        }
    }
    if let Some(t) = t_match {
        eprintln!("[bench] rule matching: {:?}", t.elapsed());
    }
    Ok(n_errors)
}

pub fn find_errors(
    rules: &[Vec<String>],
    out: &mut dyn Write,
    filetext: &str,
    filename: &str,
    output_format: &str,
    bench_internal: bool,
) -> std::io::Result<usize> {
    let mut n_errors = 0usize;
    let mut prev_newlines = 0usize;

    // Remove comments (keep %\n)
    let t_comment = bench_internal.then(std::time::Instant::now);
    let comment_re = FancyRegex::new(r"(?<!\\)%.*(?:\$|\n)").unwrap_or_else(|_| FancyRegex::new("$^").unwrap());
    let mut filetext = comment_re.replace_all(filetext, "%\n").to_string();
    filetext = tokenize(&filetext);
    if let Some(t) = t_comment {
        eprintln!("[bench] comment_remove + tokenize: {:?}", t.elapsed());
    }

    // Take content after \begin{document}
    let doc_split = Regex::new(r"\\begin\{document\}").unwrap_or_else(|_| Regex::new("$^").unwrap());
    let parts: Vec<String> = doc_split
        .split(&filetext)
        .map(|s| s.to_string())
        .collect();
    if parts.len() > 1 {
        prev_newlines = num_newlines(&parts[0]);
        if parts.len() > 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "More than one \\begin{document} in file",
            ));
        }
        filetext = parts[1].clone();
    }

    let t_compile = bench_internal.then(std::time::Instant::now);
    let compiled = compile_rules(rules);
    if let Some(t) = t_compile {
        eprintln!("[bench] regex compile ({} rules): {:?}", compiled.len(), t.elapsed());
    }

    run_compiled_rules(&compiled, &filetext, prev_newlines, out, filename, output_format, bench_internal)
}
