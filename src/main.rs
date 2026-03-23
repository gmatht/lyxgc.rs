//! ChkTeX replacement - grammar checker for LyX/LaTeX (Rust frontend).

use clap::Parser;
use std::fs;
use std::io::{self, Read};

#[derive(Parser)]
#[command(name = "chktex")]
struct Args {
    /// .tex file to check
    filename: Option<String>,

    /// Output file
    #[arg(short, long)]
    output: Option<String>,

    /// Input file (LyX uses -x file.tex)
    #[arg(short = 'x')]
    input_file: Option<String>,

    /// Verbosity: 0, 1, or 3
    #[arg(short, long, default_value = "1")]
    verbose: String,

    /// Language: LyX name or locale (e.g. en_US, fr)
    #[arg(short, long)]
    lang: Option<String>,

    /// Rules only: skip ChkTeX and lacheck
    #[arg(long)]
    rules_only: bool,

    /// Print internal timing breakdown to stderr (for benchmarking)
    #[arg(long)]
    bench_internal: bool,

    /// Cache compiled regexes per language (faster on repeat runs in same process)
    #[arg(long)]
    cache_regex: bool,

    /// Repeat check N times (for benchmarking cache; reports total time)
    #[arg(long, default_value = "1")]
    repeat: usize,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let output_format = format!("-v{}", args.verbose);
    let filename = args
        .input_file
        .or(args.filename)
        .unwrap_or_else(|| "stdin".to_string());
    let is_stdin = filename == "stdin";

    let lang_spec = args
        .lang
        .or_else(|| std::env::var("LYX_LANGUAGE").ok())
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_else(|| "en".to_string());

    let run_lacheck_chktex = !is_stdin && !args.rules_only;

    let bench_internal = args.bench_internal;
    let cache_regex = args.cache_regex;
    let repeat = args.repeat;

    let stdin_text = if is_stdin {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        Some(s)
    } else {
        None
    };

    let (mut output, mut n_errors) = (String::new(), 0usize);
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
        output = result.0;
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

    if let Some(ref path) = args.output {
        fs::write(path, output)?;
    } else {
        print!("{}", output);
    }

    std::process::exit(if n_errors > 0 { 1 } else { 0 });
}
