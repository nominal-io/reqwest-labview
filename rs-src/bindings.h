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

int32_t http_get(const char *url,
                 const char *headers_json,
                 int32_t timeout_ms,
                 uint64_t **handle_out,
                 int32_t *response_len_out,
                 uint32_t *status_out);

int32_t http_post(const char *url,
                  const char *headers_json,
                  const uint8_t *body_ptr,
                  int32_t body_len,
                  int32_t timeout_ms,
                  uint64_t **handle_out,
                  int32_t *response_len_out,
                  uint32_t *status_out);

int32_t http_put(const char *url,
                 const char *headers_json,
                 const uint8_t *body_ptr,
                 int32_t body_len,
                 int32_t timeout_ms,
                 uint64_t **handle_out,
                 int32_t *response_len_out,
                 uint32_t *status_out);

int32_t http_patch(const char *url,
                   const char *headers_json,
                   const uint8_t *body_ptr,
                   int32_t body_len,
                   int32_t timeout_ms,
                   uint64_t **handle_out,
                   int32_t *response_len_out,
                   uint32_t *status_out);

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
 */
int32_t http_read_response(uint64_t *handle_ptr, uint8_t *buf_ptr, int32_t buf_len);

/**
 * Free a response handle without reading the body.
 * Call this in error-handling paths to avoid leaking the store entry and box.
 *
 * LabVIEW CLN wiring: handle -> "Pointer to Void" (adapt to type).
 */
int32_t http_free_response(uint64_t *handle_ptr);

int32_t http_get_last_error(uint8_t *buf_ptr, int32_t buf_len);

void http_shutdown(void);

/**
 * Returns the library version as a static null-terminated string (e.g. "0.1.0").
 * The pointer is valid for the lifetime of the process; do not free it.
 *
 * LabVIEW CLN wiring: return type -> "C String Pointer".
 */
const char *http_get_version(void);

#endif  /* REQWEST_LABVIEW_H */
