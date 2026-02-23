mod error;
mod headers;
mod http;
mod runtime;
mod store;

use std::ffi::CStr;
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
///
/// The handle is heap-boxed so LabVIEW always receives a native-width pointer
/// (32-bit on 32-bit LabVIEW, 64-bit on 64-bit LabVIEW). This avoids the
/// calling-convention pitfalls of passing u64 across the FFI boundary on
/// 32-bit x86 __stdcall.
///
/// LabVIEW CLN wiring: handle_out -> "Pointer to Pointer to Void" (adapt to type).
unsafe fn write_response_outputs(
    response: crate::http::HttpResponse,
    handle_out: *mut *mut u64,
    response_len_out: *mut i32,
    status_out: *mut u32,
) -> i32 {
    let len = response.body.len() as i32;
    let status = response.status;
    let handle = insert_response(response.body, status);

    if !handle_out.is_null() {
        // Box the u64 store key and give LabVIEW a native-width pointer to it.
        // The caller must eventually pass this pointer back to http_read_response
        // or http_free_response, which will drop the box.
        *handle_out = Box::into_raw(Box::new(handle));
    }
    if !response_len_out.is_null() {
        *response_len_out = len;
    }
    if !status_out.is_null() {
        *status_out = status;
    }

    ERR_OK
}

/// Dereference a handle pointer and return the inner store key.
/// Returns Err(ERR_NULL_PTR) if the pointer is null.
unsafe fn deref_handle(handle_ptr: *mut u64) -> Result<u64, i32> {
    if handle_ptr.is_null() {
        set_last_error("Handle pointer is null");
        return Err(ERR_NULL_PTR);
    }
    Ok(*handle_ptr)
}

// ---------------------------------------------------------------------------
// Public FFI functions
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn http_get(
    url: *const c_char,
    headers_json: *const c_char,
    timeout_ms: i32,
    handle_out: *mut *mut u64,
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
pub extern "C" fn http_post(
    url: *const c_char,
    headers_json: *const c_char,
    body_ptr: *const u8,
    body_len: i32,
    timeout_ms: i32,
    handle_out: *mut *mut u64,
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
pub extern "C" fn http_put(
    url: *const c_char,
    headers_json: *const c_char,
    body_ptr: *const u8,
    body_len: i32,
    timeout_ms: i32,
    handle_out: *mut *mut u64,
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
pub extern "C" fn http_patch(
    url: *const c_char,
    headers_json: *const c_char,
    body_ptr: *const u8,
    body_len: i32,
    timeout_ms: i32,
    handle_out: *mut *mut u64,
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
pub extern "C" fn http_delete(
    url: *const c_char,
    headers_json: *const c_char,
    timeout_ms: i32,
    handle_out: *mut *mut u64,
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

/// Read the response body into the caller-supplied buffer, then free both the
/// store entry and the heap-boxed handle pointer.
///
/// LabVIEW CLN wiring: handle -> "Pointer to Void" (adapt to type).
///
/// Note on ERR_BUFFER_TOO_SMALL: the store entry is put back so you can retry
/// with a larger buffer, but the box is always freed here. Do not call
/// http_read_response or http_free_response again after this returns
/// ERR_BUFFER_TOO_SMALL - allocate a buffer of at least response_len_out bytes
/// upfront to avoid this situation.
#[no_mangle]
pub extern "C" fn http_read_response(
    handle_ptr: *mut u64,
    buf_ptr: *mut u8,
    buf_len: i32,
) -> i32 {
    clear_last_error();
    unsafe {
        let handle = match deref_handle(handle_ptr) {
            Ok(h) => h,
            Err(e) => return e,
        };
        let result = read_and_free_response(handle, buf_ptr, buf_len);
        drop(Box::from_raw(handle_ptr));
        result
    }
}

/// Free a response handle without reading the body.
/// Call this in error-handling paths to avoid leaking the store entry and box.
///
/// LabVIEW CLN wiring: handle -> "Pointer to Void" (adapt to type).
#[no_mangle]
pub extern "C" fn http_free_response(handle_ptr: *mut u64) -> i32 {
    clear_last_error();
    unsafe {
        let handle = match deref_handle(handle_ptr) {
            Ok(h) => h,
            Err(e) => return e,
        };
        let result = free_response(handle);
        drop(Box::from_raw(handle_ptr));
        result
    }
}

#[no_mangle]
pub extern "C" fn http_get_last_error(buf_ptr: *mut u8, buf_len: i32) -> i32 {
    read_last_error(buf_ptr, buf_len)
}

#[no_mangle]
pub extern "C" fn http_shutdown() {
    clear_all_responses();
}
