//! C FFI bindings for aam-rs.
//!
//! Compile with `--features ffi` (implied when building as `cdylib`).

#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::aam::AAM;
use crate::error::AamlError;
use crate::pipeline::formatter::FormattingOptions as FormatterRules;

fn first_error(errors: Vec<AamlError>) -> AamlError {
    errors.into_iter().next().unwrap_or(AamlError::ParseError {
        line: 1,
        content: String::new(),
        details: "unexpected empty parse error list".to_string(),
        diagnostics: None,
    })
}

// ── Opaque handle ────────────────────────────────────────────────────────────

/// Opaque handle to an AAM parser instance.
pub struct AamHandle {
    inner: AAM,
    last_error: Option<CString>,
}

#[deprecated(since = "1.0.0", note = "Use AamHandle instead")]
pub type AamlHandle = AamHandle;

impl AamHandle {
    fn set_error(&mut self, err: impl ToString) {
        let msg = err.to_string().replace('\0', "<NUL>");
        self.last_error = CString::new(msg).ok();
    }

    fn clear_error(&mut self) {
        self.last_error = None;
    }
}

// ── Lifecycle ────────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn aam_new() -> *mut AamHandle {
    Box::into_raw(Box::new(AamHandle {
        inner: AAM::new(),
        last_error: None,
    }))
}

#[deprecated(since = "1.0.0", note = "Use aam_new instead")]
#[unsafe(no_mangle)]
pub extern "C" fn aaml_new() -> *mut AamHandle {
    aam_new()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_free(handle: *mut AamHandle) {
    if !handle.is_null() {
        unsafe { drop(Box::from_raw(handle)) };
    }
}

// ── Parsing & Formatting ─────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_parse(handle: *mut AamHandle, content: *const c_char) -> i32 {
    if handle.is_null() || content.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };

    let content = match unsafe { CStr::from_ptr(content) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            handle.set_error(e);
            return -1;
        }
    };

    match AAM::parse(content) {
        Ok(aam) => {
            handle.inner = aam;
            handle.clear_error();
            0
        }
        Err(e) => {
            handle.set_error(first_error(e));
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_load(handle: *mut AamHandle, path: *const c_char) -> i32 {
    if handle.is_null() || path.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };

    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            handle.set_error(e);
            return -1;
        }
    };

    match AAM::load(path) {
        Ok(aam) => {
            handle.inner = aam;
            handle.clear_error();
            0
        }
        Err(e) => {
            handle.set_error(first_error(e));
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_merge(handle: *mut AamHandle, content: *const c_char) -> i32 {
    if handle.is_null() || content.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };

    let content = match unsafe { CStr::from_ptr(content) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            handle.set_error(e);
            return -1;
        }
    };

    match handle.inner.merge_content(content) {
        Ok(()) => {
            handle.clear_error();
            0
        }
        Err(e) => {
            handle.set_error(e);
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_recover_simple(
    handle: *mut AamHandle,
    content: *const c_char,
) -> i32 {
    if handle.is_null() || content.is_null() {
        return -1;
    }
    let handle = unsafe { &mut *handle };

    let content = match unsafe { CStr::from_ptr(content) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            handle.set_error(e);
            return -1;
        }
    };

    let report = AAM::recover_simple(content);
    handle.inner = report.recovered;
    handle.clear_error();
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_format(handle: *mut AamHandle, content: *const c_char) -> *mut c_char {
    if handle.is_null() || content.is_null() {
        return std::ptr::null_mut();
    }
    let handle_ref = unsafe { &mut *handle };

    let content_str = match unsafe { CStr::from_ptr(content) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            handle_ref.set_error(e);
            return std::ptr::null_mut();
        }
    };

    let rules = FormatterRules::default();
    match handle_ref.inner.format(content_str, &rules) {
        Ok(formatted) => to_c_string(&formatted),
        Err(e) => {
            handle_ref.set_error(e);
            std::ptr::null_mut()
        }
    }
}

// ── Lookup ───────────────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_find_obj(handle: *const AamHandle, key: *const c_char) -> *mut c_char {
    if handle.is_null() || key.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let key = match unsafe { CStr::from_ptr(key) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match handle.inner.find_obj(key) {
        Some(v) => to_c_string(v.as_str()),
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_find_key(
    handle: *const AamHandle,
    value: *const c_char,
) -> *mut c_char {
    if handle.is_null() || value.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let value = match unsafe { CStr::from_ptr(value) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match handle.inner.find_key(value) {
        Some(v) => to_c_string(v.as_str()),
        None => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_find_deep(
    handle: *const AamHandle,
    key: *const c_char,
) -> *mut c_char {
    if handle.is_null() || key.is_null() {
        return std::ptr::null_mut();
    }
    let handle = unsafe { &*handle };

    let key = match unsafe { CStr::from_ptr(key) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match handle.inner.find_deep(key) {
        Some(v) => to_c_string(v.as_str()),
        None => std::ptr::null_mut(),
    }
}

// ── Memory management ────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_string_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}

// ── Error reporting ──────────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn aam_last_error(handle: *const AamHandle) -> *const c_char {
    if handle.is_null() {
        return std::ptr::null();
    }
    let handle = unsafe { &*handle };
    match &handle.last_error {
        Some(cs) => cs.as_ptr(),
        None => std::ptr::null(),
    }
}

// ── Private helpers ──────────────────────────────────────────────────────────

fn to_c_string(s: &str) -> *mut c_char {
    let safe = s.replace('\0', "<NUL>");
    CString::new(safe).unwrap().into_raw()
}
