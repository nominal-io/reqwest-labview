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
/// Returns Err with an error code already set on failure.
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
/// A null pointer with length 0 is treated as an empty body.
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
    handle_out: *mut u64,
    response_len_out: *mut i32,
    status_out: *mut u32,
) -> i32 {
    let len = response.body.len() as i32;
    let status = response.status;
    let handle = insert_response(response.body, status);

    if !handle_out.is_null() {
        *handle_out = handle;
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

/// Perform an HTTP GET request.
///
/// @param url            Null-terminated UTF-8 URL string.
/// @param headers_json   Null-terminated JSON object of request headers, e.g.
///                       "{\"Authorization\": \"Bearer token\"}".
///                       Pass NULL for no headers.
/// @param timeout_ms     Request timeout in milliseconds. Pass 0 for no timeout.
/// @param handle_out     Receives an opaque handle identifying the stored response.
///                       Pass to http_read_response or http_free_response.
/// @param response_len_out  Receives the byte length of the response body.
///                          Use this to allocate the buffer before calling http_read_response.
/// @param status_out     Receives the HTTP status code (e.g. 200, 404).
/// @return               0 on success, negative error code on failure.
#[no_mangle]
pub extern "system" fn http_get(
    url: *const c_char,
    headers_json: *const c_char,
    timeout_ms: i32,
    handle_out: *mut u64,
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

/// Perform an HTTP POST request.
///
/// @param url            Null-terminated UTF-8 URL string.
/// @param headers_json   Null-terminated JSON object of request headers.
///                       Pass NULL for no headers.
/// @param body_ptr       Pointer to the raw request body bytes.
///                       Pass NULL for an empty body.
/// @param body_len       Length of the request body in bytes.
/// @param timeout_ms     Request timeout in milliseconds. Pass 0 for no timeout.
/// @param handle_out     Receives an opaque handle identifying the stored response.
/// @param response_len_out  Receives the byte length of the response body.
/// @param status_out     Receives the HTTP status code.
/// @return               0 on success, negative error code on failure.
#[no_mangle]
pub extern "system" fn http_post(
    url: *const c_char,
    headers_json: *const c_char,
    body_ptr: *const u8,
    body_len: i32,
    timeout_ms: i32,
    handle_out: *mut u64,
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

/// Perform an HTTP PUT request.
///
/// @param url            Null-terminated UTF-8 URL string.
/// @param headers_json   Null-terminated JSON object of request headers.
///                       Pass NULL for no headers.
/// @param body_ptr       Pointer to the raw request body bytes.
///                       Pass NULL for an empty body.
/// @param body_len       Length of the request body in bytes.
/// @param timeout_ms     Request timeout in milliseconds. Pass 0 for no timeout.
/// @param handle_out     Receives an opaque handle identifying the stored response.
/// @param response_len_out  Receives the byte length of the response body.
/// @param status_out     Receives the HTTP status code.
/// @return               0 on success, negative error code on failure.
#[no_mangle]
pub extern "system" fn http_put(
    url: *const c_char,
    headers_json: *const c_char,
    body_ptr: *const u8,
    body_len: i32,
    timeout_ms: i32,
    handle_out: *mut u64,
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

/// Perform an HTTP PATCH request.
///
/// @param url            Null-terminated UTF-8 URL string.
/// @param headers_json   Null-terminated JSON object of request headers.
///                       Pass NULL for no headers.
/// @param body_ptr       Pointer to the raw request body bytes.
///                       Pass NULL for an empty body.
/// @param body_len       Length of the request body in bytes.
/// @param timeout_ms     Request timeout in milliseconds. Pass 0 for no timeout.
/// @param handle_out     Receives an opaque handle identifying the stored response.
/// @param response_len_out  Receives the byte length of the response body.
/// @param status_out     Receives the HTTP status code.
/// @return               0 on success, negative error code on failure.
#[no_mangle]
pub extern "system" fn http_patch(
    url: *const c_char,
    headers_json: *const c_char,
    body_ptr: *const u8,
    body_len: i32,
    timeout_ms: i32,
    handle_out: *mut u64,
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

/// Perform an HTTP DELETE request.
///
/// @param url            Null-terminated UTF-8 URL string.
/// @param headers_json   Null-terminated JSON object of request headers.
///                       Pass NULL for no headers.
/// @param timeout_ms     Request timeout in milliseconds. Pass 0 for no timeout.
/// @param handle_out     Receives an opaque handle identifying the stored response.
/// @param response_len_out  Receives the byte length of the response body.
/// @param status_out     Receives the HTTP status code.
/// @return               0 on success, negative error code on failure.
#[no_mangle]
pub extern "system" fn http_delete(
    url: *const c_char,
    headers_json: *const c_char,
    timeout_ms: i32,
    handle_out: *mut u64,
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

/// Read and consume a stored response into a caller-supplied buffer.
///
/// The handle is consumed on success and cannot be used again.
/// If the buffer is too small (ERR_BUFFER_TOO_SMALL), the handle remains
/// valid and the call can be retried with a larger buffer.
///
/// @param handle     Handle returned by a previous http_* call.
/// @param buf_ptr    Pointer to a caller-allocated buffer to receive the response body.
/// @param buf_len    Size of the buffer in bytes. Must be >= the response_len
///                   returned by the originating http_* call.
/// @return           Number of bytes written on success, negative error code on failure.
#[no_mangle]
pub extern "system" fn http_read_response(
    handle: u64,
    buf_ptr: *mut u8,
    buf_len: i32,
) -> i32 {
    clear_last_error();
    read_and_free_response(handle, buf_ptr, buf_len)
}

/// Free a stored response without reading it.
///
/// Call this in error-handling paths where you have a handle but do not
/// intend to read the response body. Failing to call this or http_read_response
/// will leak the stored response for the lifetime of the process.
///
/// @param handle     Handle returned by a previous http_* call.
/// @return           0 on success, negative error code on failure.
#[no_mangle]
pub extern "system" fn http_free_response(handle: u64) -> i32 {
    clear_last_error();
    free_response(handle)
}

/// Retrieve the last error message as a null-terminated UTF-8 string.
///
/// Error messages are stored per-thread, so this must be called from the
/// same thread that made the failing http_* call.
///
/// @param buf_ptr    Pointer to a caller-allocated buffer.
/// @param buf_len    Size of the buffer in bytes.
/// @return           Number of bytes written (excluding null terminator),
///                   or a negative error code if buf_ptr is null.
#[no_mangle]
pub extern "system" fn http_get_last_error(buf_ptr: *mut u8, buf_len: i32) -> i32 {
    read_last_error(buf_ptr, buf_len)
}

/// Shut down the library.
///
/// Frees all stored responses. Should be called when your LabVIEW application
/// is closing or when you want to ensure all handles are released.
/// The HTTP client itself is tied to the process lifetime and is not freed.
#[no_mangle]
pub extern "system" fn http_shutdown() {
    clear_all_responses();
}
