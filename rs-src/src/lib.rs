mod error;
mod headers;
mod http;
mod runtime;
mod store;

use std::ffi::{CStr, c_void};
use std::os::raw::c_char;
use std::slice;

use error::{
    clear_last_error, read_last_error, set_last_error, ERR_NULL_PTR, ERR_INVALID_UTF8, ERR_OK,
};
use headers::parse_headers;
use store::{clear_all_responses, free_response, insert_response, read_and_free_response};

// ---------------------------------------------------------------------------
// Calling convention
// On Windows, LabVIEW's Call Library Node defaults to __stdcall.
// `extern "system"` resolves to stdcall on Windows and cdecl everywhere else.
// ---------------------------------------------------------------------------

// The handle is a u64 key stored as a pointer-sized value (void*).
// This matches LabVIEW's Instance Data Pointer type.
fn handle_to_ptr(handle: u64) -> *mut c_void {
    handle as usize as *mut c_void
}

fn ptr_to_handle(ptr: *mut c_void) -> u64 {
    ptr as usize as u64
}

/// Helper: convert a *const c_char URL to a &str.
unsafe fn url_to_str<'a>(url: *const c_char) -> Result<&'a str, i32> {
    if url.is_null() {
        set_last_error("URL pointer is null");
        return Err(ERR_NULL_PTR);
    }
    CStr::from_ptr(url).to_str().map_err(|_| {
        set_last_error("URL contains invalid UTF-8");
        ERR_INVALID_UTF8
    })
}

/// Helper: convert a raw body pointer + length into a Vec<u8>.
unsafe fn body_to_vec(body_ptr: *const u8, body_len: i32) -> Vec<u8> {
    if body_ptr.is_null() || body_len <= 0 {
        Vec::new()
    } else {
        slice::from_raw_parts(body_ptr, body_len as usize).to_vec()
    }
}

/// Helper: write outputs after a successful request.
unsafe fn write_response_outputs(
    response: crate::http::HttpResponse,
    handle_out: *mut *mut c_void,
    response_len_out: *mut i32,
    status_out: *mut u32,
) -> i32 {
    let len = response.body.len() as i32;
    let status = response.status;
    let handle = insert_response(response.body, status);

    if !handle_out.is_null() {
        *handle_out = handle_to_ptr(handle);
    }
    if !response_len_out.is_null() {
        *response_len_out = len;
    }
    if !status_out.is_null() {
        *status_out = status;
    }

    ERR_OK
}

// ---------------------------------------------------------------------------
// Public FFI functions
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn http_get(
    url: *const c_char,
    headers_json: *const c_char,
    timeout_ms: i32,
    handle_out: *mut *mut c_void,
    response_len_out: *mut i32,
    status_out: *mut u32,
) -> i32 {
    clear_last_error();
    unsafe {
        let url_str = match url_to_str(url) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let headers = match parse_headers(headers_json) {
            Ok(h) => h,
            Err(e) => return e,
        };
        match http::get(url_str, headers, timeout_ms) {
            Ok(resp) => write_response_outputs(resp, handle_out, response_len_out, status_out),
            Err(e) => e,
        }
    }
}

#[no_mangle]
pub extern "system" fn http_post(
    url: *const c_char,
    headers_json: *const c_char,
    body_ptr: *const u8,
    body_len: i32,
    timeout_ms: i32,
    handle_out: *mut *mut c_void,
    response_len_out: *mut i32,
    status_out: *mut u32,
) -> i32 {
    clear_last_error();
    unsafe {
        let url_str = match url_to_str(url) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let headers = match parse_headers(headers_json) {
            Ok(h) => h,
            Err(e) => return e,
        };
        let body = body_to_vec(body_ptr, body_len);
        match http::post(url_str, headers, body, timeout_ms) {
            Ok(resp) => write_response_outputs(resp, handle_out, response_len_out, status_out),
            Err(e) => e,
        }
    }
}

#[no_mangle]
pub extern "system" fn http_put(
    url: *const c_char,
    headers_json: *const c_char,
    body_ptr: *const u8,
    body_len: i32,
    timeout_ms: i32,
    handle_out: *mut *mut c_void,
    response_len_out: *mut i32,
    status_out: *mut u32,
) -> i32 {
    clear_last_error();
    unsafe {
        let url_str = match url_to_str(url) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let headers = match parse_headers(headers_json) {
            Ok(h) => h,
            Err(e) => return e,
        };
        let body = body_to_vec(body_ptr, body_len);
        match http::put(url_str, headers, body, timeout_ms) {
            Ok(resp) => write_response_outputs(resp, handle_out, response_len_out, status_out),
            Err(e) => e,
        }
    }
}

#[no_mangle]
pub extern "system" fn http_patch(
    url: *const c_char,
    headers_json: *const c_char,
    body_ptr: *const u8,
    body_len: i32,
    timeout_ms: i32,
    handle_out: *mut *mut c_void,
    response_len_out: *mut i32,
    status_out: *mut u32,
) -> i32 {
    clear_last_error();
    unsafe {
        let url_str = match url_to_str(url) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let headers = match parse_headers(headers_json) {
            Ok(h) => h,
            Err(e) => return e,
        };
        let body = body_to_vec(body_ptr, body_len);
        match http::patch(url_str, headers, body, timeout_ms) {
            Ok(resp) => write_response_outputs(resp, handle_out, response_len_out, status_out),
            Err(e) => e,
        }
    }
}

#[no_mangle]
pub extern "system" fn http_delete(
    url: *const c_char,
    headers_json: *const c_char,
    timeout_ms: i32,
    handle_out: *mut *mut c_void,
    response_len_out: *mut i32,
    status_out: *mut u32,
) -> i32 {
    clear_last_error();
    unsafe {
        let url_str = match url_to_str(url) {
            Ok(s) => s,
            Err(e) => return e,
        };
        let headers = match parse_headers(headers_json) {
            Ok(h) => h,
            Err(e) => return e,
        };
        match http::delete(url_str, headers, timeout_ms) {
            Ok(resp) => write_response_outputs(resp, handle_out, response_len_out, status_out),
            Err(e) => e,
        }
    }
}

#[no_mangle]
pub extern "system" fn http_read_response(
    handle: *mut c_void,
    buf_ptr: *mut u8,
    buf_len: i32,
) -> i32 {
    clear_last_error();
    read_and_free_response(ptr_to_handle(handle), buf_ptr, buf_len)
}

#[no_mangle]
pub extern "system" fn http_free_response(handle: *mut c_void) -> i32 {
    clear_last_error();
    free_response(ptr_to_handle(handle))
}

#[no_mangle]
pub extern "system" fn http_get_last_error(buf_ptr: *mut u8, buf_len: i32) -> i32 {
    read_last_error(buf_ptr, buf_len)
}

#[no_mangle]
pub extern "system" fn http_shutdown() {
    clear_all_responses();
}