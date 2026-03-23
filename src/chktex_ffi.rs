//! FFI bindings for ChkTeX C library (when chktex feature is enabled).

use std::os::raw::c_char;

#[cfg(feature = "chktex")]
extern "C" {
    pub fn chktex_check_file(path: *const c_char) -> *mut c_char;
    pub fn chktex_free(s: *mut c_char);
}
