//! C FFI bindings for lyxgc.
//!
//! Build as cdylib or staticlib to produce a C-compatible library.
//! See `include/lyxgc.h` for the header.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// Check a .tex file. Returns malloc'd output string, or NULL on failure.
/// Caller must free with `lyxgc_free`.
#[no_mangle]
pub extern "C" fn lyxgc_check_file(
    path: *const c_char,
    lang: *const c_char,
    output_format: *const c_char,
    run_lacheck_chktex: i32,
) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let lang_str = if lang.is_null() {
        "en"
    } else {
        unsafe { CStr::from_ptr(lang) }.to_str().unwrap_or("en")
    };
    let fmt_str = if output_format.is_null() {
        "-v1"
    } else {
        unsafe { CStr::from_ptr(output_format) }.to_str().unwrap_or("-v1")
    };

    match crate::check_file(path_str, lang_str, fmt_str, run_lacheck_chktex != 0, false, false) {
        Ok((output, _)) => match CString::new(output) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Check LaTeX text in memory. Returns malloc'd output string, or NULL on failure.
/// Caller must free with `lyxgc_free`.
#[no_mangle]
pub extern "C" fn lyxgc_check_text(
    text: *const c_char,
    filename: *const c_char,
    lang: *const c_char,
    output_format: *const c_char,
    _run_lacheck_chktex: i32, // ignored: lacheck/chktex require a file path
) -> *mut c_char {
    if text.is_null() || filename.is_null() {
        return std::ptr::null_mut();
    }
    let text_str = match unsafe { CStr::from_ptr(text) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let filename_str = match unsafe { CStr::from_ptr(filename) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let lang_str = if lang.is_null() {
        "en"
    } else {
        unsafe { CStr::from_ptr(lang) }.to_str().unwrap_or("en")
    };
    let fmt_str = if output_format.is_null() {
        "-v1"
    } else {
        unsafe { CStr::from_ptr(output_format) }.to_str().unwrap_or("-v1")
    };

    // run_lacheck_chktex ignored: lacheck/chktex require a real file path
    match crate::check(text_str, filename_str, lang_str, fmt_str, false, false, false) {
        Ok((output, _)) => match CString::new(output) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string returned by `lyxgc_check_file` or `lyxgc_check_text`.
#[no_mangle]
pub extern "C" fn lyxgc_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = unsafe { CString::from_raw(ptr) };
    }
}
