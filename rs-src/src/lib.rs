mod error;
mod headers;
mod http;
mod proxy;
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
    let handle = insert_response(response.body);

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

/// # Safety
/// `url` and `headers_json` must each be null or a valid null-terminated
/// string, and each output pointer must be null or valid for writes.
#[no_mangle]
pub unsafe extern "C" fn http_get(
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

/// # Safety
/// `url` and `headers_json` must each be null or a valid null-terminated
/// string, `body_ptr` must be null or valid for `body_len` bytes, and each
/// output pointer must be null or valid for writes.
#[no_mangle]
pub unsafe extern "C" fn http_post(
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

/// # Safety
/// `url` and `headers_json` must each be null or a valid null-terminated
/// string, `body_ptr` must be null or valid for `body_len` bytes, and each
/// output pointer must be null or valid for writes.
#[no_mangle]
pub unsafe extern "C" fn http_put(
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

/// # Safety
/// `url` and `headers_json` must each be null or a valid null-terminated
/// string, `body_ptr` must be null or valid for `body_len` bytes, and each
/// output pointer must be null or valid for writes.
#[no_mangle]
pub unsafe extern "C" fn http_patch(
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

/// # Safety
/// `url` and `headers_json` must each be null or a valid null-terminated
/// string, and each output pointer must be null or valid for writes.
#[no_mangle]
pub unsafe extern "C" fn http_delete(
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
///
/// # Safety
/// `handle_ptr` must be null or a handle from a request function that has not
/// been read or freed, and `buf_ptr` must be null or valid for `buf_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn http_read_response(
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
///
/// # Safety
/// `handle_ptr` must be null or a handle from a request function that has not
/// been read or freed.
#[no_mangle]
pub unsafe extern "C" fn http_free_response(handle_ptr: *mut u64) -> i32 {
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

/// Set the proxy for later requests.
///
/// `proxy` is `http://host:port`, optionally with `user:password@` for Basic
/// authentication, or `direct` for no proxy. Null or empty detects it again from
/// HTTPS_PROXY / HTTP_PROXY / ALL_PROXY (honoring NO_PROXY), then the Windows
/// proxy setting (honoring its bypass list). Without this call, requests connect
/// directly. PAC scripts and NTLM authentication are not supported.
///
/// LabVIEW CLN wiring: proxy -> "C String Pointer".
///
/// # Safety
/// `proxy` must be null or a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn http_set_proxy(proxy: *const c_char) -> i32 {
    clear_last_error();
    let proxy = if proxy.is_null() {
        None
    } else {
        match unsafe { CStr::from_ptr(proxy) }.to_str() {
            Ok(s) => Some(s.to_string()),
            Err(_) => {
                set_last_error("Proxy contains invalid UTF-8");
                return ERR_INVALID_UTF8;
            }
        }
    };
    match runtime::set_proxy(proxy) {
        Ok(()) => ERR_OK,
        Err(e) => e,
    }
}

/// Returns the library version as a static null-terminated string (e.g. "0.1.0").
/// The pointer is valid for the lifetime of the process; do not free it.
///
/// LabVIEW CLN wiring: return type -> "C String Pointer".
#[no_mangle]
pub extern "C" fn http_get_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ERR_INVALID_PROXY;
    use std::ffi::CString;
    use std::sync::mpsc::{channel, Receiver};
    use std::sync::Mutex;
    use std::time::Duration;

    /// The client and its proxy are shared by the whole process
    static CLIENT_LOCK: Mutex<()> = Mutex::new(());

    /// Accepts connections, answers each with 403, and sends back each request head
    fn recording_server() -> (String, Receiver<String>) {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            for connection in listener.incoming() {
                let Ok(mut connection) = connection else {
                    return;
                };
                let mut head = Vec::new();
                let mut byte = [0u8; 1];
                while !head.ends_with(b"\r\n\r\n") && matches!(connection.read(&mut byte), Ok(1)) {
                    head.push(byte[0]);
                }
                let _ =
                    connection.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n");
                if tx
                    .send(String::from_utf8_lossy(&head).into_owned())
                    .is_err()
                {
                    return;
                }
            }
        });
        (address, rx)
    }

    fn set_proxy(proxy: &str) -> i32 {
        let proxy = CString::new(proxy).unwrap();
        unsafe { http_set_proxy(proxy.as_ptr()) }
    }

    /// Sends a GET and returns the first request head the server receives
    fn first_request(url: &str, rx: &Receiver<String>) -> String {
        let url = CString::new(url).unwrap();
        let mut handle: *mut u64 = std::ptr::null_mut();
        let mut len = 0;
        let mut status = 0;
        let result = unsafe {
            http_get(
                url.as_ptr(),
                std::ptr::null(),
                5000,
                &mut handle,
                &mut len,
                &mut status,
            )
        };
        if result == ERR_OK {
            unsafe { http_free_response(handle) };
        }
        rx.recv_timeout(Duration::from_secs(10))
            .expect("no request reached the server")
    }

    fn last_error() -> String {
        let mut buffer = [0u8; 512];
        http_get_last_error(buffer.as_mut_ptr(), buffer.len() as i32);
        unsafe { CStr::from_ptr(buffer.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn test_https_request_tunnels_through_proxy() {
        let _lock = CLIENT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (address, rx) = recording_server();
        assert_eq!(set_proxy(&format!("http://{}", address)), ERR_OK);

        let head = first_request("https://example.invalid/path", &rx);
        assert!(
            head.starts_with("CONNECT example.invalid:443 HTTP/1.1\r\n"),
            "{}",
            head
        );
        assert!(
            !head.to_ascii_lowercase().contains("proxy-authorization"),
            "{}",
            head
        );
        set_proxy("direct");
    }

    #[test]
    fn test_http_request_goes_through_proxy() {
        let _lock = CLIENT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (address, rx) = recording_server();
        assert_eq!(set_proxy(&address), ERR_OK);

        let head = first_request("http://example.invalid/path?q=1", &rx);
        assert!(
            head.starts_with("GET http://example.invalid/path?q=1 HTTP/1.1\r\n"),
            "{}",
            head
        );
        set_proxy("direct");
    }

    #[test]
    fn test_proxy_credentials_are_sent() {
        let _lock = CLIENT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (address, rx) = recording_server();
        assert_eq!(set_proxy(&format!("http://user:pass@{}", address)), ERR_OK);

        let head = first_request("https://example.invalid/path", &rx);
        // base64 of "user:pass"
        assert!(
            head.to_ascii_lowercase()
                .contains("proxy-authorization: basic dxnlcjpwyxnz\r\n"),
            "{}",
            head
        );
        set_proxy("direct");
    }

    #[test]
    fn test_direct_skips_proxy() {
        let _lock = CLIENT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (address, rx) = recording_server();
        assert_eq!(set_proxy("direct"), ERR_OK);

        // A direct request names only the path; a proxied one names the whole URL
        let head = first_request(&format!("http://{}/path", address), &rx);
        assert!(head.starts_with("GET /path HTTP/1.1\r\n"), "{}", head);
        set_proxy("direct");
    }

    #[test]
    fn test_direct_ignores_proxy_environment() {
        let _lock = CLIENT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (proxy_address, proxy_rx) = recording_server();
        let (origin_address, origin_rx) = recording_server();
        std::env::set_var("HTTP_PROXY", format!("http://{}", proxy_address));

        assert_eq!(set_proxy("direct"), ERR_OK);
        let head = first_request(&format!("http://{}/path", origin_address), &origin_rx);
        std::env::remove_var("HTTP_PROXY");

        assert!(head.starts_with("GET /path HTTP/1.1\r\n"), "{}", head);
        assert!(proxy_rx.try_recv().is_err());
    }

    #[test]
    fn test_empty_proxy_detects_proxy_environment() {
        let _lock = CLIENT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let (address, rx) = recording_server();
        std::env::set_var("HTTP_PROXY", format!("http://{}", address));

        assert_eq!(set_proxy(""), ERR_OK);
        let head = first_request("http://example.invalid/path", &rx);
        std::env::remove_var("HTTP_PROXY");
        set_proxy("direct");

        assert!(
            head.starts_with("GET http://example.invalid/path HTTP/1.1\r\n"),
            "{}",
            head
        );
    }

    #[test]
    fn test_invalid_proxy_is_rejected() {
        let _lock = CLIENT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(set_proxy("socks5://127.0.0.1:1080"), ERR_INVALID_PROXY);
        assert!(last_error().contains("only http://"), "{}", last_error());
    }
}
