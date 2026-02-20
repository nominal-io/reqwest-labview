use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{set_last_error, ERR_INVALID_HANDLE, ERR_NULL_PTR, ERR_BUFFER_TOO_SMALL};

/// A stored HTTP response waiting to be read by the caller.
pub struct StoredResponse {
    pub body: Vec<u8>,
    pub status: u32,
}

static RESPONSES: OnceLock<Mutex<HashMap<u64, StoredResponse>>> = OnceLock::new();

// Starts at 1 so that 0 can serve as a sentinel "no handle" value in LabVIEW
static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

fn response_store() -> &'static Mutex<HashMap<u64, StoredResponse>> {
    RESPONSES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Insert a response into the store and return its handle.
pub fn insert_response(body: Vec<u8>, status: u32) -> u64 {
    let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    response_store()
        .lock()
        .unwrap()
        .insert(handle, StoredResponse { body, status });
    handle
}

/// Copy the response body into a caller-supplied buffer, then free the handle.
/// Returns the number of bytes written, or a negative error code.
/// The handle is consumed on success - it cannot be read twice.
pub fn read_and_free_response(handle: u64, buf_ptr: *mut u8, buf_len: i32) -> i32 {
    if buf_ptr.is_null() {
        set_last_error("Response buffer pointer is null");
        return ERR_NULL_PTR;
    }

    let mut store = response_store().lock().unwrap();
    let Some(resp) = store.remove(&handle) else {
        set_last_error(format!("Invalid or already-consumed handle: {}", handle));
        return ERR_INVALID_HANDLE;
    };

    let available = buf_len as usize;
    if resp.body.len() > available {
        // Put it back so the caller can retry with a larger buffer
        store.insert(handle, resp);
        set_last_error(format!(
            "Buffer too small: need {} bytes, got {}",
            store[&handle].body.len(),
            available
        ));
        return ERR_BUFFER_TOO_SMALL;
    }

    let copy_len = resp.body.len();
    unsafe {
        std::ptr::copy_nonoverlapping(resp.body.as_ptr(), buf_ptr, copy_len);
    }
    copy_len as i32
}

/// Free a response handle without reading it.
/// Call this in error-handling paths where you received a handle but
/// do not intend to read the response.
pub fn free_response(handle: u64) -> i32 {
    let removed = response_store().lock().unwrap().remove(&handle);
    if removed.is_none() {
        set_last_error(format!("Invalid or already-freed handle: {}", handle));
        return ERR_INVALID_HANDLE;
    }
    0
}

/// Clear all stored responses. Called from http_shutdown.
pub fn clear_all_responses() {
    response_store().lock().unwrap().clear();
}

/// Returns the number of responses currently in the store.
/// Useful for detecting handle leaks during development.
pub fn pending_response_count() -> usize {
    response_store().lock().unwrap().len()
}
