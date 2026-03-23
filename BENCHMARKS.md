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

3. **No caching by default**: Compiling regexes could be cached (e.g. per language module), but unless `--cache-regex` is enabled, each `chktex` invocation compiles all rules. With `--cache-regex`, compiled rules are cached for the lifetime of the process, so it only helps when the same process performs multiple checks.

## Regex caching (`--cache-regex`)

`--cache-regex` enables an in-process cache of compiled rules (keyed by language module). This is useful for scenarios where the checker is called repeatedly from the *same* long-lived process.

For “real” `chktex` wall-clock timing (fresh process per check, so startup time is included), benchmark without `--repeat` and do one invocation per timing sample (see `py/benchmark.py`).

### Benchmark commands

```bash
# In-process warm benchmark (single process, uses chktex's `--repeat`)
# Without cache: every loop iteration recompiles the ~266 regexes.
./rs/target/release/chktex --bench-internal --rules-only --repeat 20 py/tests/fixtures/simple_errors.tex -o /dev/null

# With cache: compiled rules are reused for iterations 2–20 within the same process.
./rs/target/release/chktex --bench-internal --rules-only --cache-regex --repeat 20 py/tests/fixtures/simple_errors.tex -o /dev/null
```

On Windows, replace `/dev/null` with `NUL`.

### Cache-regex expectations (what to benchmark)

- **Per-invocation wall-clock timing** (fresh process per run; no `--repeat`): `--cache-regex` should be close to no-cache.
- **In-process warm timing** (single process with `--repeat`): `--cache-regex` can be much faster.

**~10× speedup** per run when using the cache: first run pays the compile cost (~100 ms); subsequent runs skip it and report `0ns (cached)` for regex compile.

## Binary size (download backends)

When a language JSON file is missing, it can be downloaded from lyx-gc.py. Two backends are available: `curl` (default; shell out, smaller binary) or `ureq` (pure Rust, portable). Build with different features to choose:

```bash
cd rs

# Baseline: no download (smallest)
cargo build --release --no-default-features --features chktex

# curl (default; requires curl on PATH)
cargo build --release --features chktex,download-curl

# ureq (portable, no external tools)
cargo build --release --no-default-features --features chktex,download-ureq
```

On Windows, `curl` uses `curl.exe` to avoid the PowerShell alias. Only enable one download feature; if both are enabled, curl takes precedence.

### Size comparison (Windows x64, release)

| Config           | Size (bytes) | Size (MB) |
|------------------|--------------|-----------|
| no download      | 1 739 776    | 1.70      |
| download-curl    | 1 830 912    | 1.79      |
| download-ureq    | 3 257 344    | 3.11      |

ureq adds ~1.5 MB (rustls, webpki, etc.). curl adds ~90 KB (std::process::Command + URL handling).

## Rust release profiles

Six release profiles allow trade-offs between binary size and speed:

| Profile              | opt-level | Use case                                               |
|----------------------|-----------|--------------------------------------------------------|
| `release` (default)  | 3 (regex-automata) / z (others) | **Default.** Small binary, competitive speed; regex-automata hot, rest size-optimized |
| `release-opt2`       | 2 (all)   | Moderate optimization; smaller than opt3, slightly slower |
| `release-opt3`       | 3 (all)   | Full speed; explicit opt-level 3 everywhere            |
| `release-fast-regex` | 3 (extended regex) / s (others) | Same speed as opt3; extends regex hot path  |
| `release-size`       | s (all)   | Smallest binary; slowest runtime                       |
| `release-regex-automata-hot-cold-z` | Same as `release` | Alias; kept for compatibility |

### Why automata=opt3, all-other=opt-z is the default

We chose `regex-automata` at opt-level 3 and all other crates at opt-level `z` as the default `release` profile after per-crate benchmarking:

1. **regex-automata dominates the hot path.** Internal timing shows ~62% of no-cache time in regex compilation; per-crate analysis found regex-automata alone at opt-3 gives the best speed/size trade-off (~23 ms faster for ~69 KB vs all-opt-s).

2. **opt-z yields a smaller binary than opt-s.** Cold paths at opt-z shrink the binary ~50 KB vs opt-s, with competitive or indistinguishable speed (benchmark variance is high; differences are within noise).

3. **Single hot crate minimizes codegen footprint.** Bumping other crates (regex, fancy-regex, etc.) from z to s did not improve speed and sometimes slowed runs; keeping only regex-automata hot avoids extra code bloat.

4. **Good balance.** The default profile is ~50 KB smaller than release-hot-regex-automata (all-s cold) and within a few ms in speed. Use `release-opt3` for maximum speed when binary size does not matter.

### Build commands

```bash
cd rs

# Default (regex-automata=opt3, rest=opt-z)
cargo build --release
# Output: target/release/chktex[.exe]

# Opt-level 2 (moderate optimization)
cargo build --profile release-opt2
# Output: target/release-opt2/chktex[.exe]

# Explicit opt-level 3 (full speed)
cargo build --profile release-opt3
# Output: target/release-opt3/chktex[.exe]

# Extended regex hot path (same speed as opt3, same size)
cargo build --profile release-fast-regex
# Output: target/release-fast-regex/chktex[.exe]

# Size-optimized
cargo build --profile release-size
# Output: target/release-size/chktex[.exe]

# Same as release (alias)
cargo build --profile release-regex-automata-hot-cold-z
# Output: target/release-regex-automata-hot-cold-z/chktex[.exe]
```

Use the same feature flags as above (`--no-default-features --features chktex` for no download, etc.).

### Benchmarking profiles

```bash
cd py
# Single profile (no-cache and cached)
python benchmark.py --rs-profile release -n 20 --rules-only

# All profiles with no-cache and cached modes
python benchmark.py --all-profiles --rules-only -n 20
```

The benchmark reports both no-cache (20 subprocess invocations) and cached (single process with `--cache-regex --repeat 20`) timings, plus binary size.

### Size comparison (Windows x64, no download)

| Profile              | Size (bytes) | Size (MB) |
|----------------------|--------------|-----------|
| release (default)    | ~1 510 000   | ~1.44     |
| release-opt2         | 1 667 072    | 1.59      |
| release-opt3         | 1 739 776    | 1.70      |
| release-fast-regex   | 1 739 776    | 1.70      |
| release-size         | 1 496 064    | 1.43      |

`release-size` is the smallest. Default `release` (automata=3, rest=z) is ~50 KB larger than release-size, ~230 KB smaller than release-opt3.

### Speed comparison (rules-only, simple_errors.tex, n=20, Windows)

#### No cache (20 subprocess invocations per run)

| Profile              | Avg (ms) | Min | Max |
|----------------------|----------|-----|-----|
| release (default)    | ~80–90   | ~75 | ~95 |
| release-opt2         | 83       | 77  | 93  |
| release-opt3         | 78       | 74  | 94  |
| release-fast-regex   | 77       | 74  | 84  |
| release-size         | 102      | 99  | 111 |

`release-opt3` and `release-fast-regex` are fastest (~78 ms). Default `release` (automata=3, rest=z) is competitive; variance is high, often within ~5 ms of release-hot-regex-automata.

#### Cached (single process, `--cache-regex --repeat 20`)

| Profile              | Avg (ms) | Min | Max |
|----------------------|----------|-----|-----|
| release (default)    | ~7–8     | -   | -   |
| release-opt2         | 7.4      | 7.4 | 7.4 |
| release-opt3         | 7.2      | 7.2 | 7.2 |
| release-fast-regex   | 7.2      | 7.2 | 7.2 |
| release-size         | 9.4      | 9.4 | 9.4 |

With regex caching, `release-opt3` and `release-fast-regex` are fastest. Default `release` is typically ~7–8 ms per run. Use `--cache-regex` when LyX or another client invokes the checker many times in one process.

## Hot crate per-crate analysis

Each profile compiles **only one** hot crate at opt-level 3; all others at opt-level s. Baseline: `release-size` (all opt-s).

```bash
cd py
python benchmark.py --hot-crate-analysis -n 20
```

### Results (rules-only, simple_errors.tex, n=20, Windows, rerun)

| Hot crate (only opt-3) | Size (MB) | vs baseline | No-cache (ms) | vs baseline | Cached (ms) | vs baseline |
|------------------------|-----------|-------------|---------------|-------------|-------------|--------------|
| (baseline: release-size) | 1.43 | - | 103 | - | 10.1 | - |
| regex | 1.43 | +0 | 104 | +1.5 | 9.7 | -0.4 |
| fancy-regex | 1.43 | +0 | 103 | +0.1 | 9.5 | -0.6 |
| aho-corasick | 1.43 | +0 | 104 | +1.4 | 9.6 | -0.5 |
| memchr | 1.43 | +0 | 110 | +7.5 | 15.1 | +5.0 |
| regex-syntax | 1.48 | +56 KB | 105 | +2.9 | 9.3 | -0.8 |
| regex-automata | 1.49 | +71 KB | 79 | -23 | 7.5 | -2.6 |
| bit-set | 1.43 | +0 | 105 | +2.5 | 10.4 | +0.3 |
| bit-vec | 1.43 | -0.5 KB | 106 | +3.8 | 9.8 | -0.3 |

### Findings

- **regex-automata** gives the best no-cache speed/size trade-off: ~23 ms faster for ~69 KB (0.33 ms/KB).
- **memchr** alone at opt-3 is counterproductive: slower in both modes (especially cached +5 ms); likely different inlining/codegen when isolated.
- **regex-syntax** adds 56 KB and small cached gain (-0.8 ms); modest benefit.
- Most single-crate optimizations add negligible size (PE alignment) but don't improve no-cache speed; only regex-automata shows a large no-cache gain.

## regex-automata hot with cold paths at opt-z

Profile `release-regex-automata-hot-cold-z`: regex-automata at opt-level 3, all other crates at opt-level `z` (size-oriented). Shrinks cold-path code more than opt-level s.

```bash
cd rs
cargo build --profile release-regex-automata-hot-cold-z --no-default-features --features chktex
# Output: target/release-regex-automata-hot-cold-z/chktex[.exe]
```

### Results (rules-only, simple_errors.tex, n=20, Windows)

| Profile                              | Size (MB) | No-cache (ms) | Cached (ms) |
|-------------------------------------|-----------|---------------|-------------|
| release-size (baseline)             | 1.43      | 103           | 10.1        |
| release-hot-regex-automata (opt-s cold) | 1.49   | 79            | 7.5         |
| **release-regex-automata-hot-cold-z**   | 1.44   | 82–86         | 8.5         |

### Trade-off vs release-hot-regex-automata

- **~50 KB smaller** (1.44 vs 1.49 MB) from cold paths at opt-z instead of opt-s
- **~3–4 ms slower** no-cache (82 vs 79 ms)
- **~1 ms slower** cached (8.5 vs 7.5 ms)

Use this profile when you want most of the regex-automata speed benefit with a smaller binary.

## automata=opt3, one hot crate=s, rest=z

Profiles with regex-automata at opt-3 and exactly one other hot crate at opt-s (rest at opt-z). Tests whether bumping any cold crate from z to s improves speed.

```bash
cd py
python benchmark.py --automata3-s-rest-z-analysis -n 20 --rules-only
```

### Results (rules-only, simple_errors.tex, n=20, Windows)

| Profile                 | Size (MB) | vs base | No-cache (ms) | vs base | Cached (ms) | vs base |
|-------------------------|-----------|---------|---------------|---------|-------------|---------|
| (baseline all-z)        | 1.44      | -       | 134.22        | -       | 113.06      | -       |
| regex=s                 | 1.43      | -3 KB   | 115.50        | -18.7   | 106.42      | -6.64   |
| fancy-regex=s           | 1.43      | -10 KB  | 134.26        | +0.0    | 102.79      | -10.27  |
| aho-corasick=s          | 1.43      | -12 KB  | 112.39        | -21.8   | 105.59      | -7.47   |
| memchr=s                | 1.44      | -0.5 KB | 108.97        | -25.3   | 132.15      | +19.09  |
| regex-syntax=s          | 1.45      | +15 KB  | 114.31        | -19.9   | 106.27      | -6.78   |
| bit-set=s               | 1.44      | +0      | 108.31        | -25.9   | 104.15      | -8.91   |
| bit-vec=s               | 1.44      | +0      | 115.58        | -18.6   | 105.00      | -8.06   |

### Findings

- **Size rule still holds:** `regex-syntax=s` is the only variant that clearly increases binary size (+15 KB).
- For this rerun, **most z->s variants are faster than all-z** in no-cache timing.
- `memchr=s` is an outlier: fastest no-cache here, but much worse cached timing (+19 ms vs baseline).
- Practical policy for default `release`: set z->s for `regex`, `fancy-regex`, `aho-corasick`, `memchr`, `bit-set`, `bit-vec`; keep `regex-syntax` at z.

## Updated default `--release` report (Windows, rules-only)

`rs/Cargo.toml` default `release` profile was updated accordingly:

- Keep `regex-automata=3`
- Keep `regex-syntax=z`
- Set `regex`, `fancy-regex`, `aho-corasick`, `memchr`, `bit-set`, `bit-vec` to `s`

Measured with:

```bash
cd py
python benchmark.py --rs-only --rs-profile release --rules-only -n 20
```

Observed results (two immediate runs):

| Run | No-cache avg (ms) | Cached avg (ms) | Size |
|-----|-------------------|-----------------|------|
| 1   | 176.41            | 146.16          | 1.62 MB |
| 2   | 122.12            | 136.54          | 1.62 MB |
