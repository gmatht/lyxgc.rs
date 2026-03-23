# lyx-gc Benchmarks

Benchmarks for Python, Perl, and Rust implementations of the lyx-gc LaTeX grammar checker.

## How to Run

From the `py/` directory:

```bash
cd py
python benchmark.py -n 20              # Full mode: py, pl, rs (full), rs (rules-only)
python benchmark.py -n 20 --rules-only  # Rules-only: all impls skip ChkTeX/lacheck
```

### Running in WSL (recommended on Windows)

The easiest way to get real ChkTeX and lacheck on Windows is via WSL. From PowerShell or CMD:

```bash
wsl -e bash -c "cd /mnt/d/GitHub/lyx-gc/py && python3 benchmark.py -n 20"
```

Replace `d` with your Windows drive letter if the repo is on another drive.

- **Full mode**: Python and Perl run with ChkTeX/lacheck enabled; Rust is benchmarked both full and rules-only.
- **Rules-only mode**: All implementations run with ChkTeX and lacheck disabled (only internal rules).

Rust must be built first:

- **Native Linux (WSL)**: `wsl -e bash -c "cd /mnt/d/GitHub/lyx-gc/rs && cargo build --release"` — produces native `chktex` binary, ~2× faster than Windows exe in WSL.
- **Windows**: `cd rs && cargo build --release` — produces `chktex.exe`.

Internal timing (where Rust spends time): add `--bench-internal` to any chktex invocation. Output goes to stderr.

For cache benchmarks: use `--repeat N` (e.g. `--repeat 20`) to run N times in one process; combine with `--cache-regex` to measure warm runs.

Perl requires `path/chktex.pl`. For full ChkTeX/lacheck in Python/Perl, install them in WSL: `sudo apt install chktex lacheck`.

## Modes

| Mode        | ChkTeX | lacheck | LanguageTool | Internal rules |
|-------------|--------|---------|--------------|----------------|
| **Full**    | yes    | yes     | yes (Python) | yes            |
| **Rules-only** | no  | no      | no           | yes            |

- **Python full**: Uses LanguageTool when available; ChkTeX/lacheck via external tools.
- **Rust**: ChkTeX and lacheck; no LanguageTool. Use `--rules-only` to skip external tools.
- **Perl**: ChkTeX and lacheck. Set `LYXGC_RULES_ONLY=1` for rules-only.

## Results (n=20, simple_errors.tex)

### WSL (Linux, native Rust binary)

| Implementation | Avg (ms) | Min | Max |
|----------------|----------|-----|-----|
| Python rules engine (find_errors) | ~3.3 | 2.4 | 4.9 |
| Python CLI (full) | 361 | 270 | 444 |
| Rust CLI (full) | 138 | 87 | 365 |
| Rust CLI (rules-only) | 99 | 77 | 163 |
| Perl (full) | 41 | 32 | 53 |

### Rules-only mode (WSL)

| Implementation | Avg (ms) | Min | Max |
|----------------|----------|-----|-----|
| Python rules engine (find_errors) | ~3.4 | 2.8 | 4.7 |
| Python CLI (rules-only) | 265 | 212 | 335 |
| Rust CLI (rules-only) | 99 | 77 | 163 |
| Perl (rules-only) | 45 | 35 | 67 |

### Windows (native)

#### Full mode

| Implementation | Avg (ms) | Min | Max |
|----------------|----------|-----|-----|
| Python rules engine (find_errors) | ~3.9 | 2.9 | 5.2 |
| Python CLI (full) | 275 | 240 | 421 |
| Rust CLI (full) | 116 | 92 | 289 |
| Rust CLI (rules-only) | 112 | 99 | 134 |
| Perl (full) | 136 | 128 | 145 |

#### Rules-only mode

| Implementation | Avg (ms) | Min | Max |
|----------------|----------|-----|-----|
| Python rules engine (find_errors) | ~3.7 | 3.0 | 5.0 |
| Python CLI (rules-only) | 206 | 161 | 340 |
| Rust CLI (rules-only) | 141 | 116 | 257 |
| Perl (rules-only) | 150 | 132 | 204 |

## Summary

- **Rules engine**: Python `find_errors` is ~2-4 ms per run (in-process, no subprocess overhead).
- **CLI (full)**: Perl is fastest in WSL (41 ms) with native ChkTeX/lacheck. Rust (138 ms) and Python (361 ms) follow.
- **CLI (rules-only)**: Perl (45 ms) is fastest in WSL; Rust (99 ms) with native Linux binary, then Python (265 ms).
- **Windows (native)**: Rust (116 ms full, 141 ms rules-only) and Perl (136/150 ms) are similar; Python is slower due to interpreter startup and optional LanguageTool.
- **Variance**: Rust max times can spike; likely due to external tools or system load.

Fixture: `py/tests/fixtures/simple_errors.tex`. Benchmarks run with `ORIG_CHKTEX` unset and `LANGUAGETOOL_PATH` set to avoid Java/LanguageTool startup for fair comparison.

## Rust internal timing (rules-only, simple_errors.tex)

Run with `--bench-internal` to see where the Rust binary spends time:

```bash
./rs/target/release/chktex --bench-internal --rules-only -v0 -o /dev/null py/tests/fixtures/simple_errors.tex
```

Typical breakdown (WSL, n=5 runs):

| Phase | Avg (ms) | % |
|-------|----------|---|
| load_language (JSON parse + placeholders) | ~11 | 10% |
| comment_remove + tokenize | ~25 | 22% |
| **regex compile (266 rules)** | **~70** | **62%** |
| rule matching | ~9 | 8% |

*(With miniserde + lexopt, load_language is ~1.4 ms; exe ~1.7 MB vs ~2.0 MB with clap+serde.)*

**Regex compilation dominates.** All 266 rules are compiled from JSON on every invocation. Perl embeds its regexes and compiles them once at script startup.

### Why isn't Rust regex as fast as Perl?

1. **Per-compile cost**: Rust uses `fancy-regex` for lookahead/lookbehind. Each `FancyRegex::new()` parses and compiles the pattern. We do this 266 times per run. Perl compiles regexes once when the script loads.

2. **Engine design**: Perl's regex engine (Oniguruma-derived) is C code optimized over decades for common text-processing patterns. `fancy-regex` is a hybrid: it uses the standard `regex` crate for simple patterns and falls back to backtracking for fancy features. The rules use `(?i)`, `(?<!...)`, `(?!...)` etc., so they go through the backtracking path, which has different performance characteristics.

3. **No caching**: Compiling regexes could be cached (e.g. per language module), but currently each `find_errors` call recompiles all rules. Perl keeps compiled regexes in memory across the run.

## Regex caching (`--cache-regex`)

Compiled rules are cached per language. Use `--cache-regex` when running multiple checks in the same process (e.g. LyX calling the checker many times) to avoid recompiling the ~266 regexes on every run.

### Benchmark commands

```bash
# Without cache (cold compile every run)
./rs/target/release/chktex --bench-internal --rules-only --repeat 20 py/tests/fixtures/simple_errors.tex -o /dev/null

# With cache (compile once, reuse for runs 2–20)
./rs/target/release/chktex --bench-internal --rules-only --cache-regex --repeat 20 py/tests/fixtures/simple_errors.tex -o /dev/null
```

On Windows, replace `/dev/null` with `NUL`.

### Observed cache benefit (Windows, n=20)

| Mode   | Total (20 runs) | Avg/run |
|--------|------------------|---------|
| No cache | ~2.08 s         | ~104 ms |
| `--cache-regex` | ~208 ms   | ~10 ms  |

**~10× speedup** per run when using the cache: first run pays the compile cost (~100 ms); subsequent runs skip it and report `0ns (cached)` for regex compile.
