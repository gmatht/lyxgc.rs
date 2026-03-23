# lyxgc - ChkTeX replacement (Rust)

Grammar checker for LyX/LaTeX. Drop-in replacement for ChkTeX with internal rules, vendored lacheck, and ChkTeX C library.

## Quick start

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/gmatht/lyxgc.rs/master/quickstart.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/gmatht/lyxgc.rs/master/quickstart.ps1 | iex
```

These fetch the latest release binary and run it on a sample file. No Rust toolchain required.

## Build from source

```bash
cargo build --release
# Binary: target/release/chktex
```

The default release profile uses opt-level z for all crates except regex-automata (hot path at 3).

## Usage

```bash
chktex yourfile.tex
chktex -v0 -o output.txt yourfile.tex
chktex -l fr yourfile.tex
chktex --rules-only yourfile.tex   # Skip ChkTeX/lacheck
```

## Releases

Binaries are built on tag push (e.g. `v0.1.0`). See [Releases](https://github.com/gmatht/lyxgc.rs/releases).
