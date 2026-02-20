use once_cell::sync::OnceCell;
use reqwest::blocking::Client;
use std::time::Duration;

use crate::error::{set_last_error, ERR_CLIENT_INIT};

static CLIENT: OnceCell<Client> = OnceCell::new();

/// Returns a reference to the shared blocking HTTP client.
/// The client is initialised on first call and reused for all subsequent calls.
/// Reusing the client allows connection pooling across requests.
pub fn get_client() -> Result<&'static Client, i32> {
    CLIENT.get_or_try_init(|| {
        Client::builder()
            .use_rustls_tls()           // No OpenSSL dependency
            .tcp_keepalive(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                set_last_error(format!("Failed to initialise HTTP client: {}", e));
                ERR_CLIENT_INIT
            })
    })
}

/// Attempt to reinitialise the client. Only succeeds if the client has not
/// yet been initialised (i.e. after http_shutdown clears it).
/// In practice, shutdown drops the static - see store.rs for shutdown logic.
pub fn reset_client() {
    // OnceCell doesn't support reset directly; the process must reinitialise.
    // This is intentional - the client is tied to the process lifetime.
    // Documented here for clarity.
}
