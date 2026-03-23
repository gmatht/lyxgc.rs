//! Build lacheck and chktex as static libraries.
//!
//! - **lacheck**: Always built. Uses YY_NO_UNISTD_H on Windows (MSVC).
//! - **chktex**: Built when `chktex` feature is enabled (default).
//!   On Windows: uses vendored getopt, config.h stubs for MSVC.
//!   On Unix: uses system getopt, config.h with POSIX headers.

use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor = manifest.join("vendor");

    // Phase 1: lacheck
    let lacheck_dir = vendor.join("lacheck");
    cc::Build::new()
        .file(lacheck_dir.join("lacheck.c"))
        .file(lacheck_dir.join("lacheck_lib.c"))
        .define("LACHECK_LIB", None)
        .define("YY_NO_UNISTD_H", None) // skip unistd.h on Windows (MSVC)
        .compile("lacheck");

    // Phase 2: chktex (when "chktex" feature is enabled)
    if std::env::var("CARGO_FEATURE_CHKTEX").is_ok() {
        build_chktex(&vendor);
    }
}

fn build_chktex(vendor: &PathBuf) {
    let chktex_dir = vendor.join("chktex").join("chktex");
    let getopt_dir = vendor.join("chktex").join("getopt");

    let mut build = cc::Build::new();
    build
        .define("HAVE_CONFIG_H", None)
        .define("CHKTEX_LIB", None)
        .include(&chktex_dir)
        .file(chktex_dir.join("ChkTeX.c"))
        .file(chktex_dir.join("FindErrs.c"))
        .file(chktex_dir.join("Utility.c"))
        .file(chktex_dir.join("Resource.c"))
        .file(chktex_dir.join("OpSys.c"))
        .file(vendor.join("chktex").join("chktex_lib.c"));

    let is_windows = std::env::var("TARGET").map(|t| t.contains("windows")).unwrap_or(false);
    if !is_windows {
        build.file(vendor.join("chktex").join("strlwr_compat.c"));
    }
    if is_windows {
        build
            .define("_WIN32", None)
            .define("__MSDOS__", None)  /* OpSys.h needs SLASH, DIRCHARS */
            .include(&getopt_dir)
            .file(getopt_dir.join("getopt.c"));
    }

    build.compile("chktex");
    println!("cargo:rustc-cfg=chktex_available");
}
