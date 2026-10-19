#ifndef REQWEST_LABVIEW_H
#define REQWEST_LABVIEW_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdint.h>

/**
 * Error codes returned by all public functions.
 * Positive values are HTTP status codes (200, 404, etc.) stored separately.
 * Negative values are library-level errors.
 */
#define ERR_OK 0

#define ERR_NULL_PTR -1

#define ERR_INVALID_UTF8 -2

#define ERR_INVALID_HEADERS -3

#define ERR_REQUEST_FAILED -4

#define ERR_INVALID_HANDLE -5

#define ERR_BUFFER_TOO_SMALL -6

#define ERR_CLIENT_INIT -7

#define ERR_INVALID_PROXY -8

/**
 * # Safety
 * `url` and `headers_json` must each be null or a valid null-terminated
 * string, and each output pointer must be null or valid for writes.
 */
int32_t http_get(const char *url,
                 const char *headers_json,
                 int32_t timeout_ms,
                 uint64_t **handle_out,
                 int32_t *response_len_out,
                 uint32_t *status_out);

/**
 * # Safety
 * `url` and `headers_json` must each be null or a valid null-terminated
 * string, `body_ptr` must be null or valid for `body_len` bytes, and each
 * output pointer must be null or valid for writes.
 */
int32_t http_post(const char *url,
                  const char *headers_json,
                  const uint8_t *body_ptr,
                  int32_t body_len,
                  int32_t timeout_ms,
                  uint64_t **handle_out,
                  int32_t *response_len_out,
                  uint32_t *status_out);

/**
 * # Safety
 * `url` and `headers_json` must each be null or a valid null-terminated
 * string, `body_ptr` must be null or valid for `body_len` bytes, and each
 * output pointer must be null or valid for writes.
 */
int32_t http_put(const char *url,
                 const char *headers_json,
                 const uint8_t *body_ptr,
                 int32_t body_len,
                 int32_t timeout_ms,
                 uint64_t **handle_out,
                 int32_t *response_len_out,
                 uint32_t *status_out);

/**
 * # Safety
 * `url` and `headers_json` must each be null or a valid null-terminated
 * string, `body_ptr` must be null or valid for `body_len` bytes, and each
 * output pointer must be null or valid for writes.
 */
int32_t http_patch(const char *url,
                   const char *headers_json,
                   const uint8_t *body_ptr,
                   int32_t body_len,
                   int32_t timeout_ms,
                   uint64_t **handle_out,
                   int32_t *response_len_out,
                   uint32_t *status_out);

/**
 * # Safety
 * `url` and `headers_json` must each be null or a valid null-terminated
 * string, and each output pointer must be null or valid for writes.
 */
int32_t http_delete(const char *url,
                    const char *headers_json,
                    int32_t timeout_ms,
                    uint64_t **handle_out,
                    int32_t *response_len_out,
                    uint32_t *status_out);

/**
 * Read the response body into the caller-supplied buffer, then free both the
 * store entry and the heap-boxed handle pointer.
 *
 * LabVIEW CLN wiring: handle -> "Pointer to Void" (adapt to type).
 *
 * Note on ERR_BUFFER_TOO_SMALL: the store entry is put back so you can retry
 * with a larger buffer, but the box is always freed here. Do not call
 * http_read_response or http_free_response again after this returns
 * ERR_BUFFER_TOO_SMALL - allocate a buffer of at least response_len_out bytes
 * upfront to avoid this situation.
 *
 * # Safety
 * `handle_ptr` must be null or a handle from a request function that has not
 * been read or freed, and `buf_ptr` must be null or valid for `buf_len` bytes.
 */
int32_t http_read_response(uint64_t *handle_ptr, uint8_t *buf_ptr, int32_t buf_len);

/**
 * Free a response handle without reading the body.
 * Call this in error-handling paths to avoid leaking the store entry and box.
 *
 * LabVIEW CLN wiring: handle -> "Pointer to Void" (adapt to type).
 *
 * # Safety
 * `handle_ptr` must be null or a handle from a request function that has not
 * been read or freed.
 */
int32_t http_free_response(uint64_t *handle_ptr);

int32_t http_get_last_error(uint8_t *buf_ptr, int32_t buf_len);

void http_shutdown(void);

/**
 * Set the proxy for later requests.
 *
 * `proxy` is `http://host:port`, optionally with `user:password@` for Basic
 * authentication, or `direct` for no proxy. Null or empty detects it again from
 * HTTPS_PROXY / HTTP_PROXY / ALL_PROXY (honoring NO_PROXY), then the Windows
 * proxy setting (honoring its bypass list). Without this call, requests connect
 * directly. PAC scripts and NTLM authentication are not supported.
 *
 * LabVIEW CLN wiring: proxy -> "C String Pointer".
 *
 * # Safety
 * `proxy` must be null or a valid null-terminated string.
 */
int32_t http_set_proxy(const char *proxy);

/**
 * Returns the library version as a static null-terminated string (e.g. "0.1.0").
 * The pointer is valid for the lifetime of the process; do not free it.
 *
 * LabVIEW CLN wiring: return type -> "C String Pointer".
 */
const char *http_get_version(void);

#endif  /* REQWEST_LABVIEW_H */
