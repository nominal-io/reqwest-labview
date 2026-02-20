#ifndef HTTP_RS_LABVIEW_H
#define HTTP_RS_LABVIEW_H

#ifdef __cplusplus
extern "C" {
#endif

int http_get(const char *url,
             const char *headers_json,
             int timeout_ms,
             void **handle_out,
             int *response_len_out,
             unsigned int *status_out);

int http_post(const char *url,
              const char *headers_json,
              const unsigned char *body_ptr,
              int body_len,
              int timeout_ms,
              void **handle_out,
              int *response_len_out,
              unsigned int *status_out);

int http_put(const char *url,
             const char *headers_json,
             const unsigned char *body_ptr,
             int body_len,
             int timeout_ms,
             void **handle_out,
             int *response_len_out,
             unsigned int *status_out);

int http_patch(const char *url,
               const char *headers_json,
               const unsigned char *body_ptr,
               int body_len,
               int timeout_ms,
               void **handle_out,
               int *response_len_out,
               unsigned int *status_out);

int http_delete(const char *url,
                const char *headers_json,
                int timeout_ms,
                void **handle_out,
                int *response_len_out,
                unsigned int *status_out);

int http_read_response(void *handle,
                       unsigned char *buf_ptr,
                       int buf_len);

int http_free_response(void *handle);

int http_get_last_error(unsigned char *buf_ptr,
                        int buf_len);

void http_shutdown(void);

#ifdef __cplusplus
}
#endif

#endif /* HTTP_RS_LABVIEW_H */