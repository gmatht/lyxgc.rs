//! Error reporting for LyX-compatible output formats.

use regex::Regex;
use std::io::Write;

use crate::tokenizer::tokens_to_user;

fn embed_error_tags(rule_context: &str, rule_ptrs: &str, s_tag: &str, e_tag: &str) -> String {
    if let (Some(f), Some(l)) = (rule_ptrs.find('^'), rule_ptrs.rfind('^')) {
        if f <= l {
            format!(
                "{}{}{}{}{}",
                &rule_context[..f],
                s_tag,
                &rule_context[f..=l],
                e_tag,
                &rule_context[l + 1..]
            )
        } else {
            rule_context.to_string()
        }
    } else {
        rule_context.to_string()
    }
}

pub fn report_error(
    out: &mut dyn Write,
    line_num: usize,
    col_num: usize,
    rule_id: &str,
    rule_name: &str,
    rule_description: &str,
    _rule_trigger: &str,
    rule_context: &str,
    rule_ptrs: &str,
    error_filename: &str,
    output_format: &str,
) -> std::io::Result<()> {
    let rule_name = rule_name.trim();
    let rule_context = rule_context.trim();
    let rule_id_str = format!("{}; {}", rule_id, rule_name);

    let line = if output_format == "-v0" || output_format == "-v3" {
        let mut error_text = String::new();
        if !rule_description.is_empty() {
            error_text.push_str(&rule_description.trim());
            error_text.push_str(".\n\n");
        }
        if !rule_context.is_empty() {
            let rule_context_flat = rule_context.replace('\n', " ").replace('\r', " ");
            let rule_context_flat = Regex::new(r"^\s*")
                .unwrap()
                .replace(&rule_context_flat, " ");
            error_text.push_str("> ");
            error_text.push_str(&embed_error_tags(
                &rule_context_flat,
                &format!(" {}", rule_ptrs),
                ">>",
                "<<",
            ));
            error_text.push_str(".\n\n  ");
        }
        error_text = error_text.trim().to_string();
        if !error_text.is_empty() {
            let newline_hack = "  ";
            let par_hack = "  ";
            error_text = error_text.replace("\n\n", par_hack);
            error_text = error_text.replace('\n', newline_hack);
            error_text = error_text.replace(':', "<COLON/>");
        }
        let rule_id_safe = rule_id_str.replace(':', "<COLON/>");
        let error_text = tokens_to_user(&error_text);
        let error_filename_safe = error_filename.replace(':', "<COLON/>");

        if output_format == "-v0" {
            format!(
                "{}:{}:{}:{}:{}\n",
                error_filename_safe, line_num, col_num, rule_id_safe, error_text
            )
        } else {
            format!(
                "\"{}\", line {}: {}\n",
                error_filename, line_num, error_text
            )
        }
    } else {
        let mut error_text = (rule_description).replace('\n', " ");
        if !error_text.is_empty() {
            error_text.push_str(".  ");
        }
        let rule_context_flat = rule_context.replace('\n', " ").replace('\r', " ");
        let rule_ptrs_flat = rule_ptrs.replace('\n', " ").replace('\r', " ");
        let error_text = tokens_to_user(&error_text);
        format!(
            "Warning {} in {} line {}: {}\n{}\n{}\n",
            rule_id_str, error_filename, line_num, error_text, rule_context_flat, rule_ptrs_flat
        )
    };

    out.write_all(line.as_bytes())
}
