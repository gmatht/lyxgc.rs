//! Parse lacheck output and report errors (ported from Python lacheck_parse.py).

use regex::Regex;
use std::ffi::CString;
use std::io::Write;
use std::os::raw::c_char;

use crate::lacheck_ffi;
use crate::report::report_error;

lazy_static::lazy_static! {
    static ref LINE_RE: Regex = Regex::new(r#""([^"]+)", line (\d+): (.*)"#).unwrap();
    static ref IGNORE_RE: Regex = Regex::new(
        r"possible unwanted space at|Could not open|Whitespace before punctation|bad character in label|unmatched"
    )
    .unwrap();
}

/// Parse lacheck output string and report findings to `out`. Returns number of errors reported.
/// Used for testing without FFI; production code calls this via parse_lacheck_output.
pub fn parse_lacheck_output_from_str(
    output: &str,
    path: &str,
    out: &mut dyn Write,
    output_format: &str,
) -> std::io::Result<usize> {
    let mut n_errors = 0usize;
    for line in output.lines() {
        if let Some(caps) = LINE_RE.captures(line) {
            let error_filename = caps.get(1).map_or(path, |m| m.as_str());
            let line_num: usize = caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
            let error_name = caps.get(3).map_or("", |m| m.as_str());
            if !IGNORE_RE.is_match(error_name) {
                report_error(
                    out,
                    line_num,
                    1,
                    "lacheck",
                    error_name,
                    "",
                    "",
                    "",
                    "",
                    error_filename,
                    output_format,
                )?;
                n_errors += 1;
            }
        }
    }
    Ok(n_errors)
}

/// Run lacheck on `path` and report findings to `out`. Returns number of errors reported.
/// Path is converted to forward slashes so C fopen works on all platforms.
pub fn parse_lacheck_output(
    path: &str,
    out: &mut dyn Write,
    output_format: &str,
) -> std::io::Result<usize> {
    let path_fixed = path.replace('\\', "/");
    let c_path = match CString::new(path_fixed.as_str()) {
        Ok(p) => p,
        Err(_) => return Ok(0),
    };

    let ptr = unsafe { lacheck_ffi::lacheck_check_file(c_path.as_ptr()) };
    if ptr.is_null() {
        return Ok(0);
    }

    let output = unsafe {
        let s = std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned();
        lacheck_ffi::lacheck_free(ptr as *mut c_char);
        s
    };

    parse_lacheck_output_from_str(&output, path, out, output_format)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_lacheck_output_from_str_empty() {
        let mut out = Cursor::new(Vec::new());
        let n = parse_lacheck_output_from_str("", "file.tex", &mut out, "-v0").unwrap();
        assert_eq!(n, 0);
        assert!(out.into_inner().is_empty());
    }

    #[test]
    fn parse_lacheck_output_from_str_single() {
        let lacheck_output = r#""/path/to/file.tex", line 3: perhaps you should insert a `~' before "\ref"
"#;
        let mut out = Cursor::new(Vec::new());
        let n = parse_lacheck_output_from_str(lacheck_output, "file.tex", &mut out, "-v0").unwrap();
        assert_eq!(n, 1);
        let s = String::from_utf8(out.into_inner()).unwrap();
        assert!(s.contains("lacheck"));
        assert!(s.contains("perhaps you should insert"));
        assert!(s.contains(":3:1"));
    }

    #[test]
    fn parse_lacheck_output_from_str_ignores_unmatched() {
        let lacheck_output = r#""file.tex", line 2: unmatched `)'
"#;
        let mut out = Cursor::new(Vec::new());
        let n = parse_lacheck_output_from_str(lacheck_output, "file.tex", &mut out, "-v0").unwrap();
        assert_eq!(n, 0, "unmatched errors should be ignored");
    }

    #[test]
    fn parse_lacheck_output_from_str_multiple() {
        let lacheck_output = r#""file.tex", line 2: insert a `~' before "\ref"
"other.tex", line 5: another warning
"#;
        let mut out = Cursor::new(Vec::new());
        let n = parse_lacheck_output_from_str(lacheck_output, "file.tex", &mut out, "-v0").unwrap();
        assert_eq!(n, 2);
    }
}
