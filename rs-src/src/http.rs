use std::time::Duration;
use reqwest::header::HeaderMap;

use crate::error::{set_last_error, ERR_REQUEST_FAILED};
use crate::runtime::get_client;

pub struct HttpResponse {
    pub status: u32,
    pub body: Vec<u8>,
}

/// Internal helper: attach headers and timeout to a RequestBuilder, then execute.
fn execute(
    builder: reqwest::blocking::RequestBuilder,
    headers: HeaderMap,
    timeout_ms: i32,
) -> Result<HttpResponse, i32> {
    let builder = builder.headers(headers);

    let builder = if timeout_ms > 0 {
        builder.timeout(Duration::from_millis(timeout_ms as u64))
    } else {
        builder
    };

    let response = builder.send().map_err(|e| {
        set_last_error(format!("Request failed: {}", e));
        ERR_REQUEST_FAILED
    })?;

    let status = response.status().as_u16() as u32;

    let body = response.bytes().map_err(|e| {
        set_last_error(format!("Failed to read response body: {}", e));
        ERR_REQUEST_FAILED
    })?;

    Ok(HttpResponse {
        status,
        body: body.to_vec(),
    })
}

pub fn get(url: &str, headers: HeaderMap, timeout_ms: i32) -> Result<HttpResponse, i32> {
    let client = get_client()?;
    execute(client.get(url), headers, timeout_ms)
}

pub fn post(
    url: &str,
    headers: HeaderMap,
    body: Vec<u8>,
    timeout_ms: i32,
) -> Result<HttpResponse, i32> {
    let client = get_client()?;
    execute(client.post(url).body(body), headers, timeout_ms)
}

pub fn put(
    url: &str,
    headers: HeaderMap,
    body: Vec<u8>,
    timeout_ms: i32,
) -> Result<HttpResponse, i32> {
    let client = get_client()?;
    execute(client.put(url).body(body), headers, timeout_ms)
}

pub fn patch(
    url: &str,
    headers: HeaderMap,
    body: Vec<u8>,
    timeout_ms: i32,
) -> Result<HttpResponse, i32> {
    let client = get_client()?;
    execute(client.patch(url).body(body), headers, timeout_ms)
}

pub fn delete(url: &str, headers: HeaderMap, timeout_ms: i32) -> Result<HttpResponse, i32> {
    let client = get_client()?;
    execute(client.delete(url), headers, timeout_ms)
}
