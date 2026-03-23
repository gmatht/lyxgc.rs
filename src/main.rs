//! ChkTeX replacement - grammar checker for LyX/LaTeX (Rust frontend).

use lexopt::prelude::*;
use std::fs;
use std::io::{self, Read};

fn parse_args() -> Result<
    (
        String,
        Option<String>,
        String,
        Option<String>,
        bool,
        bool,
        bool,
        usize,
    ),
    lexopt::Error,
> {
    let mut output = None::<String>;
    let mut input_file = None::<String>;
    let mut verbose = "1".to_string();
    let mut lang = None::<String>;
    let mut rules_only = false;
    let mut bench_internal = false;
    let mut cache_regex = true;
    let mut repeat = 1usize;
    let mut filename = None::<String>;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('o') | Long("output") => {
                output = Some(parser.value()?.string()?);
            }
            Short('x') => {
                input_file = Some(parser.value()?.string()?);
            }
            Short('v') | Long("verbose") => {
                verbose = parser.value()?.string()?;
            }
            Short('l') | Long("lang") => {
                lang = Some(parser.value()?.string()?);
            }
            Long("rules-only") => rules_only = true,
            Long("bench-internal") => bench_internal = true,
            Long("cache-regex") => cache_regex = true,
            Long("no-cache-regex") => cache_regex = false,
            Long("repeat") => {
                repeat = parser.value()?.parse()?;
            }
            Short('h') | Long("help") => {
                eprintln!("Usage: chktex [OPTIONS] [FILE]
  -o, --output <FILE>    Write output to FILE
  -x <FILE>              Input file (LyX uses -x file.tex)
  -v, --verbose <0|1|3>  Verbosity (default: 1)
  -l, --lang <LANG>      Language (LyX name or locale, e.g. en_US, fr)
  --rules-only           Skip ChkTeX and lacheck
  --bench-internal       Print timing breakdown to stderr
  --cache-regex          Cache compiled regexes per language (default)
  --no-cache-regex       Disable regex caching
  --repeat <N>           Run check N times (for benchmarking)");
                std::process::exit(0);
            }
            Value(val) if filename.is_none() => {
                filename = Some(val.string()?);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    let resolved_filename = input_file
        .or(filename)
        .unwrap_or_else(|| "stdin".to_string());

    let output_format = format!("-v{}", verbose);

    Ok((
        output_format,
        output,
        resolved_filename,
        lang,
        rules_only,
        bench_internal,
        cache_regex,
        repeat,
    ))
}

fn main() -> std::io::Result<()> {
    let args = parse_args().unwrap_or_else(|e| {
        eprintln!("chktex: {}", e);
        std::process::exit(2);
    });

    let (
        output_format,
        output_path,
        filename,
        lang,
        rules_only,
        bench_internal,
        cache_regex,
        repeat,
    ) = args;

    let is_stdin = filename == "stdin";

    let lang_spec = lang
        .or_else(|| std::env::var("LYX_LANGUAGE").ok())
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_else(|| "en".to_string());

    let run_lacheck_chktex = !is_stdin && !rules_only;

    let stdin_text = if is_stdin {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        Some(s)
    } else {
        None
    };

    let (mut result_text, mut n_errors) = (String::new(), 0usize);
    let t_total = (bench_internal || repeat > 1).then(std::time::Instant::now);
    for iter in 0..repeat {
        let show_phase = bench_internal && (repeat == 1 || iter == 0);
        let result = if let Some(ref filetext) = stdin_text {
            lyxgc::check(
                filetext,
                &filename,
                &lang_spec,
                &output_format,
                false, // no lacheck/chktex on stdin
                show_phase,
                cache_regex,
            )?
        } else {
            lyxgc::check_file(
                &filename,
                &lang_spec,
                &output_format,
                run_lacheck_chktex,
                show_phase,
                cache_regex,
            )?
        };
        result_text = result.0;
        n_errors = result.1;
    }
    if let Some(t) = t_total {
        eprintln!(
            "[bench] total {} run(s): {:?} (avg {:?}/run)",
            repeat,
            t.elapsed(),
            t.elapsed() / repeat.max(1) as u32
        );
    }

    if let Some(ref path) = output_path {
        fs::write(path, &result_text)?;
    } else {
        print!("{}", result_text);
    }

    std::process::exit(if n_errors > 0 { 1 } else { 0 });
}
