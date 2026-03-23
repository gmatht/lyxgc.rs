# lyxgc - Rust library & C API

LyX/LaTeX grammar checker. Built as Rust library and C-compatible shared/static library.

## Rust library

```toml
[dependencies]
lyxgc = { path = "../rs" }
```

```rust
use lyxgc::{check, check_file};

// Check a file (reads from disk, runs lacheck/chktex)
let (output, n_errors) = check_file("document.tex", "en", "-v1", true)?;

// Check text in memory (rules only; lacheck/chktex need a file)
let (output, n_errors) = check("\\documentclass{article}\\begin{document}Hello.\\end{document}", 
    "stdin", "en", "-v1", false)?;
```

## C library

Build produces `lyxgc.dll` (Windows), `liblyxgc.so` (Linux), `liblyxgc.dylib` (macOS), and `liblyxgc.a` (static).

```bash
cargo build --release
# target/release/liblyxgc.dll (Windows) or liblyxgc.so (Unix)
# target/release/liblyxgc.a (static)
```

### C API

Include `include/lyxgc.h`:

```c
#include "lyxgc.h"

char *out = lyxgc_check_file("document.tex", "en", "-v1", 1);
if (out) {
    printf("%s", out);
    lyxgc_free(out);
}
```

| Function | Description |
|----------|-------------|
| `lyxgc_check_file(path, lang, format, run_lacheck_chktex)` | Check file. Returns malloc'd string. |
| `lyxgc_check_text(text, filename, lang, format, _)` | Check text in memory. `run_lacheck_chktex` ignored. |
| `lyxgc_free(ptr)` | Free result. |
