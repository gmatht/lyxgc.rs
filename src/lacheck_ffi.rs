//! FFI bindings for lacheck C library.

use std::os::raw::c_char;

extern "C" {
    /// Run lacheck on a file; returns malloc'd string or null.
    /// Caller must call lacheck_free on the result.
    pub fn lacheck_check_file(path: *const c_char) -> *mut c_char;

    /// Free string returned by lacheck_check_file.
    pub fn lacheck_free(s: *mut c_char);
}
