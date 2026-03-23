//! Parse ChkTeX output and report errors (ported from Python chktex_parse.py).

use regex::Regex;
use std::ffi::CString;
use std::io::Write;

use crate::report::report_error;

#[cfg(feature = "chktex")]
use crate::chktex_ffi;

lazy_static::lazy_static! {
    static ref MSG_RE: Regex = Regex::new(
        r"(Warning|Error|Message) ([^ ]+) in (.*) line ([1-9][0-9]*): (.*)"
    )
    .unwrap();
}

/// Parse ChkTeX output string and report findings to `out`. Returns number of errors reported.
/// Used for testing without FFI; production code calls this via parse_chktex_output.
pub fn parse_chktex_output_from_str(
    output: &str,
    path: &str,
    out: &mut dyn Write,
    output_format: &str,
) -> std::io::Result<usize> {
    let mut n_errors = 0usize;
    let mut error_pos = 1u32;
    let mut line_num = 0usize;
    let mut rule_id = "";
    let mut error_name = "";
    let mut error_context = "";
    let mut error_ptr = "";

    for line in output.lines() {
        if error_pos == 1 {
            if let Some(caps) = MSG_RE.captures(line) {
                rule_id = caps.get(2).map_or("", |m| m.as_str());
                line_num = caps.get(4).map_or(0, |m| m.as_str().parse().unwrap_or(0));
                error_name = caps.get(5).map_or("", |m| m.as_str().trim_end_matches('.'));
                error_pos = 2;
            }
        } else if error_pos == 2 {
            error_context = line.trim_end_matches('.');
            error_pos = 3;
        } else if error_pos == 3 {
            error_ptr = line;
            report_error(
                out,
                line_num,
                1,
                rule_id,
                error_name,
                "",
                "",
                error_context,
                error_ptr,
                path,
                output_format,
            )?;
            n_errors += 1;
            error_pos = 1;
        }
    }
    Ok(n_errors)
}

/// Run ChkTeX on `path` and report findings to `out`. Returns number of errors reported.
#[cfg(feature = "chktex")]
pub fn parse_chktex_output(
    path: &str,
    out: &mut dyn Write,
    output_format: &str,
) -> std::io::Result<usize> {
    let path_fixed = path.replace('\\', "/");
    let c_path = match CString::new(path_fixed.as_str()) {
        Ok(p) => p,
        Err(_) => return Ok(0),
    };

    let ptr = unsafe { chktex_ffi::chktex_check_file(c_path.as_ptr()) };
    if ptr.is_null() {
        return Ok(0);
    }

    let output = unsafe {
        let s = std::ffi::CStr::from_ptr(ptr)
            .to_string_lossy()
            .into_owned();
        chktex_ffi::chktex_free(ptr as *mut std::os::raw::c_char);
        s
    };

    parse_chktex_output_from_str(&output, path, out, output_format)
}

#[cfg(not(feature = "chktex"))]
pub fn parse_chktex_output(
    _path: &str,
    _out: &mut dyn Write,
    _output_format: &str,
) -> std::io::Result<usize> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_chktex_output_from_str_empty() {
        let mut out = Cursor::new(Vec::new());
        let n = parse_chktex_output_from_str("", "file.tex", &mut out, "-v0").unwrap();
        assert_eq!(n, 0);
        assert!(out.into_inner().is_empty());
    }

    #[test]
    fn parse_chktex_output_from_str_single_warning() {
        let chktex_output = r#"Warning 1 in /path/to/file.tex line 3: Command terminated with space.
Some context line.
   ^
"#;
        let mut out = Cursor::new(Vec::new());
        let n = parse_chktex_output_from_str(chktex_output, "file.tex", &mut out, "-v0").unwrap();
        assert_eq!(n, 1);
        let s = String::from_utf8(out.into_inner()).unwrap();
        assert!(s.contains("1; Command terminated with space"));
        assert!(s.contains("file.tex:3:1"));
    }

    #[test]
    fn parse_chktex_output_from_str_multiple_warnings() {
        let chktex_output = r#"Warning 1 in file.tex line 2: First error.
ctx1
  ^
Warning 2 in file.tex line 4: Second error.
ctx2
   ^
"#;
        let mut out = Cursor::new(Vec::new());
        let n = parse_chktex_output_from_str(chktex_output, "file.tex", &mut out, "-v0").unwrap();
        assert_eq!(n, 2);
        let s = String::from_utf8(out.into_inner()).unwrap();
        assert!(s.contains("First error"));
        assert!(s.contains("Second error"));
    }
}
