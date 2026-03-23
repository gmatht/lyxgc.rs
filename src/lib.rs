//! LyX/LaTeX grammar checker library.
//!
//! Rust API:
//!
//! ```ignore
//! use lyxgc::{check, check_file};
//!
//! let (output, n_errors) = check_file("document.tex", "en", "-v1", true)?;
//! ```
//!
//! C API: see `include/lyxgc.h` and `lyxgc_check_file`, `lyxgc_free`.

pub mod chktex_ffi;
pub mod chktex_parse;
pub mod engine;
pub mod lang;
pub mod lacheck_ffi;
pub mod lacheck_parse;
pub mod registry;
pub mod report;
pub mod rules;
pub mod tokenizer;

mod capi;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub use engine::{compile_rules, find_errors, find_errors_compiled, CompiledRule};

lazy_static! {
    static ref REGEX_CACHE: Mutex<HashMap<String, Arc<Vec<CompiledRule>>>> =
        Mutex::new(HashMap::new());
}
pub use lang::load_language;
pub use registry::resolve_language;

use std::io::Cursor;

/// Run the full check pipeline on text.
///
/// Returns `(output_string, error_count)`.
/// If `bench_internal` is true, prints timing breakdown to stderr.
/// If `cache_regex` is true, compiled regexes are cached per language (faster on repeat runs).
pub fn check(
    filetext: &str,
    filename: &str,
    lang_spec: &str,
    output_format: &str,
    run_lacheck_chktex: bool,
    bench_internal: bool,
    cache_regex: bool,
) -> std::io::Result<(String, usize)> {
    let rule_module = registry::resolve_language(lang_spec)
        .or_else(|| {
            let low = lang_spec.to_lowercase();
            if low == "c" || low.starts_with("c.") {
                Some("en".to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "en".to_string());

    let t0 = bench_internal.then(std::time::Instant::now);
    let error_types = lang::load_language(&rule_module);
    if let Some(t) = t0 {
        eprintln!("[bench] load_language (JSON parse + placeholders): {:?}", t.elapsed());
    }

    let mut out = Cursor::new(Vec::new());
    let mut n_errors = if cache_regex {
        let compiled_arc: Arc<Vec<CompiledRule>> = {
            let mut cache = REGEX_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(&rule_module) {
                if bench_internal {
                    eprintln!("[bench] regex compile ({} rules): 0ns (cached)", cached.len());
                }
                Arc::clone(cached)
            } else {
                drop(cache);
                let t_compile = bench_internal.then(std::time::Instant::now);
                let compiled = engine::compile_rules(&error_types);
                if let Some(t) = t_compile {
                    eprintln!("[bench] regex compile ({} rules): {:?}", compiled.len(), t.elapsed());
                }
                let arc = Arc::new(compiled);
                REGEX_CACHE.lock().unwrap().insert(rule_module.clone(), Arc::clone(&arc));
                arc
            }
        };
        engine::find_errors_compiled(
            &compiled_arc[..],
            &mut out,
            filetext,
            filename,
            output_format,
            bench_internal,
        )?
    } else {
        engine::find_errors(
            &error_types,
            &mut out,
            filetext,
            filename,
            output_format,
            bench_internal,
        )?
    };

    if run_lacheck_chktex {
        let t1 = bench_internal.then(std::time::Instant::now);
        n_errors += lacheck_parse::parse_lacheck_output(&filename, &mut out, output_format)?;
        if let Some(t) = t1 {
            eprintln!("[bench] lacheck FFI: {:?}", t.elapsed());
        }
        let t2 = bench_internal.then(std::time::Instant::now);
        n_errors += chktex_parse::parse_chktex_output(&filename, &mut out, output_format)?;
        if let Some(t) = t2 {
            eprintln!("[bench] chktex FFI: {:?}", t.elapsed());
        }
    }

    let mut output = String::from_utf8(out.into_inner()).unwrap_or_default();
    if n_errors == 0 {
        output.push_str("X:1:1: All OK (^_^)\n");
    }

    Ok((output, n_errors))
}

/// Run the full check pipeline on a file.
///
/// Returns `(output_string, error_count)`. Reads file from disk.
pub fn check_file(
    path: &str,
    lang_spec: &str,
    output_format: &str,
    run_lacheck_chktex: bool,
    bench_internal: bool,
    cache_regex: bool,
) -> std::io::Result<(String, usize)> {
    let path_buf = std::path::Path::new(path)
        .canonicalize()
        .unwrap_or_else(|_| std::path::Path::new(path).to_path_buf());
    let filetext = std::fs::read_to_string(&path_buf).unwrap_or_default();
    let mut filename_display = path_buf.to_string_lossy().into_owned();
    if filename_display.starts_with(r"\\?\") {
        filename_display.drain(..4);
    }

    check(
        &filetext,
        &filename_display,
        lang_spec,
        output_format,
        run_lacheck_chktex,
        bench_internal,
        cache_regex,
    )
}

#[cfg(test)]
#[test]
fn test_check_api() {
    let (_, n) = check(
        r"\begin{document}Test.\end{document}",
        "t.tex",
        "en",
        "-v0",
        false,
        false,
        false,
    )
    .unwrap();
    assert!(n >= 0);
}
