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

## Language JSON Cache

For languages whose `lang/data/{module}.json` file is not bundled with the binary, the checker downloads it on demand and caches it:

- Unix: `$XDG_CACHE_HOME/lyxgc` (fallback: `$HOME/.cache/lyxgc`)
- Windows: `%LOCALAPPDATA%\lyxgc`

Set `LYXGC_DATA` to a directory to override both reads and downloads (writes to `LYXGC_DATA/{module}.json`).

## Releases

Binaries are built on tag push (e.g. `v0.1.0`). See [Releases](https://github.com/gmatht/lyxgc.rs/releases).
