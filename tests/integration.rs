//! Integration tests for ChkTeX and lacheck FFI - run the binary on fixtures.

use std::path::PathBuf;
use std::process::Command;

fn chktex_exe() -> Option<PathBuf> {
    let target_dir = std::env::var("CARGO_TARGET_DIR").ok()
        .or_else(|| Some("target".to_string()))
        .unwrap();
    let exe_name = if cfg!(target_os = "windows") {
        "chktex.exe"
    } else {
        "chktex"
    };
    for subdir in ["release", "debug"] {
        let p = PathBuf::from(&target_dir).join(subdir).join(exe_name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

#[test]
fn test_chktex_rules_only_on_simple_errors() {
    let exe = match chktex_exe() {
        Some(p) => p,
        None => {
            // Build first
            let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
                .args(["build"])
                .current_dir(env!("CARGO_MANIFEST_DIR"))
                .status()
                .expect("cargo build");
            assert!(status.success(), "cargo build failed");
            chktex_exe().expect("chktex binary not found after build")
        }
    };

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("py")
        .join("tests")
        .join("fixtures")
        .join("simple_errors.tex");

    if !fixture.exists() {
        return;
    }

    let output = Command::new(&exe)
        .args(["-v0", "-l", "English", "--rules-only", fixture.to_str().unwrap()])
        .env("LANG", "en_US.UTF-8")
        .env("LYX_LANGUAGE", "English")
        .output()
        .expect("chktex run");

    let out = String::from_utf8_lossy(&output.stdout);
    let err = String::from_utf8_lossy(&output.stderr);

    // Rules engine should report grammar errors
    assert!(
        out.contains("666") || out.contains("we that") || out.contains("spelt"),
        "Expected rules output. stdout: {:?} stderr: {:?}",
        out,
        err
    );
}

#[test]
fn test_lacheck_ffi_on_fixture() {
    let exe = match chktex_exe() {
        Some(p) => p,
        None => return,
    };

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("py")
        .join("tests")
        .join("fixtures")
        .join("lacheck_errors.tex");

    if !fixture.exists() {
        return;
    }

    let output = Command::new(&exe)
        .args(["-v0", "-l", "English", fixture.to_str().unwrap()])
        .env("LANG", "en_US.UTF-8")
        .output()
        .expect("chktex run");

    let out = String::from_utf8_lossy(&output.stdout);

    // lacheck should report the \ref spacing warning (not ignored by IGNORE_RE)
    assert!(
        out.contains("lacheck") && out.contains("perhaps you should insert"),
        "Expected lacheck FFI output. stdout: {:?}",
        out
    );
}

