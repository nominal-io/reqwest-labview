/// Error codes returned by all public functions.
/// Positive values are HTTP status codes (200, 404, etc.) stored separately.
/// Negative values are library-level errors.
pub const ERR_OK: i32 = 0;
pub const ERR_NULL_PTR: i32 = -1;
pub const ERR_INVALID_UTF8: i32 = -2;
pub const ERR_INVALID_HEADERS: i32 = -3;
pub const ERR_REQUEST_FAILED: i32 = -4;
pub const ERR_INVALID_HANDLE: i32 = -5;
pub const ERR_BUFFER_TOO_SMALL: i32 = -6;
pub const ERR_CLIENT_INIT: i32 = -7;

use std::cell::RefCell;

// Thread-local storage for the last error message.
// Using thread-local means concurrent calls from different LabVIEW threads
// never clobber each other's error strings.
thread_local! {
    static LAST_ERROR: RefCell<String> = RefCell::new(String::new());
}

/// Store an error message for retrieval via http_get_last_error.
pub fn set_last_error(msg: impl Into<String>) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = msg.into();
    });
}

/// Clear the last error.
pub fn clear_last_error() {
    LAST_ERROR.with(|e| {
        e.borrow_mut().clear();
    });
}

/// Copy the last error string into a caller-supplied buffer.
/// Returns the number of bytes written, or a negative error code.
pub fn read_last_error(buf_ptr: *mut u8, buf_len: i32) -> i32 {
    if buf_ptr.is_null() || buf_len <= 0 {
        return ERR_NULL_PTR;
    }

    LAST_ERROR.with(|e| {
        let error = e.borrow();
        let bytes = error.as_bytes();
        // Leave room for a null terminator
        let copy_len = bytes.len().min((buf_len as usize).saturating_sub(1));
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf_ptr, copy_len);
            // Write null terminator
            *buf_ptr.add(copy_len) = 0;
        }
        copy_len as i32
    })
}
