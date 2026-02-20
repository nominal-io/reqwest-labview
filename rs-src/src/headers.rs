use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::ffi::CStr;
use std::os::raw::c_char;

use crate::error::{set_last_error, ERR_INVALID_HEADERS, ERR_INVALID_UTF8};

/// Parse a null-terminated JSON string of the form {"Key": "Value", ...}
/// into a reqwest HeaderMap.
///
/// Returns Ok(HeaderMap) on success, or a negative error code on failure.
/// Passing a null pointer returns an empty HeaderMap (no headers).
pub fn parse_headers(headers_json: *const c_char) -> Result<HeaderMap, i32> {
    // Null pointer means no headers - that's fine
    if headers_json.is_null() {
        return Ok(HeaderMap::new());
    }

    let json_str = unsafe { CStr::from_ptr(headers_json) }
        .to_str()
        .map_err(|_| {
            set_last_error("Headers JSON string contains invalid UTF-8");
            ERR_INVALID_UTF8
        })?;

    // Empty string also means no headers
    if json_str.trim().is_empty() {
        return Ok(HeaderMap::new());
    }

    let map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(json_str).map_err(|e| {
            set_last_error(format!("Failed to parse headers JSON: {}", e));
            ERR_INVALID_HEADERS
        })?;

    let mut header_map = HeaderMap::new();

    for (key, value) in map {
        let header_name = HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
            set_last_error(format!("Invalid header name '{}': {}", key, e));
            ERR_INVALID_HEADERS
        })?;

        let value_str = value.as_str().ok_or_else(|| {
            set_last_error(format!("Header value for '{}' must be a string", key));
            ERR_INVALID_HEADERS
        })?;

        let header_value = HeaderValue::from_str(value_str).map_err(|e| {
            set_last_error(format!("Invalid header value for '{}': {}", key, e));
            ERR_INVALID_HEADERS
        })?;

        header_map.insert(header_name, header_value);
    }

    Ok(header_map)
}
