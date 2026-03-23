//! ChkTeX replacement - grammar checker for LyX/LaTeX (Rust frontend).

mod engine;
mod lang;
mod registry;
mod report;
mod rules;
mod tokenizer;

use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

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
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let output_format = format!("-v{}", args.verbose);
    let filename = args.input_file.or(args.filename).unwrap_or_else(|| "stdin".to_string());

    let filetext = if filename == "stdin" {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        s
    } else {
        let path = Path::new(&filename);
        let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        fs::read_to_string(&abs_path).unwrap_or_default()
    };

    let filename_display = if filename == "stdin" {
        filename.clone()
    } else {
        std::fs::canonicalize(&filename)
            .map(|p| {
                let s = p.to_string_lossy().into_owned();
                if s.starts_with(r"\\?\") {
                    s[4..].to_string()
                } else {
                    s
                }
            })
            .unwrap_or(filename)
    };

    let lang_spec = args
        .lang
        .or_else(|| std::env::var("LYX_LANGUAGE").ok())
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_else(|| "en".to_string());

    let rule_module = registry::resolve_language(&lang_spec)
        .or_else(|| {
            let low = lang_spec.to_lowercase();
            if low == "c" || low.starts_with("c.") {
                Some("en".to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "en".to_string());

    let error_types = lang::load_language(&rule_module);

    let mut out: Box<dyn Write> = if let Some(ref path) = args.output {
        Box::new(fs::File::create(path)?)
    } else {
        Box::new(io::stdout())
    };

    let n_errors = engine::find_errors(
        &error_types,
        out.as_mut(),
        &filetext,
        &filename_display,
        &output_format,
    )?;

    out.flush()?;

    if let Some(ref path) = args.output {
        if n_errors == 0 {
            let mut f = fs::OpenOptions::new().append(true).open(path)?;
            writeln!(f, "X:1:1: All OK (^_^)")?;
        }
    }

    std::process::exit(if n_errors > 0 { 1 } else { 0 });
}
