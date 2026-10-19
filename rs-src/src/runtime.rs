use reqwest::blocking::Client;
use reqwest::Proxy;
use std::sync::Mutex;
use std::time::Duration;
use url::Url;

use crate::error::{set_last_error, ERR_CLIENT_INIT, ERR_INVALID_PROXY};
use crate::proxy::{is_direct, ProxyServer, ProxySettings};

struct ClientState {
    client: Option<Client>,
    /// The proxy set with http_set_proxy, where empty detects it.
    /// `None`, the default, connects directly.
    proxy: Option<String>,
}

static STATE: Mutex<ClientState> = Mutex::new(ClientState {
    client: None,
    proxy: None,
});

fn lock_state() -> std::sync::MutexGuard<'static, ClientState> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Returns the shared blocking HTTP client.
/// The client is created on first call, choosing its proxy then, and reused for
/// all subsequent calls. Reusing the client allows connection pooling across requests.
pub fn get_client() -> Result<Client, i32> {
    let mut state = lock_state();
    if let Some(client) = &state.client {
        return Ok(client.clone());
    }
    let client = build_client(state.proxy.as_deref()).map_err(|e| {
        set_last_error(e);
        ERR_CLIENT_INIT
    })?;
    state.client = Some(client.clone());
    Ok(client)
}

/// Sets the proxy for later requests: an address, `direct`, or `None` or empty to
/// detect it. Requests already running finish on the old client.
pub fn set_proxy(proxy: Option<String>) -> Result<(), i32> {
    let proxy = match proxy.as_deref().map(str::trim) {
        Some(value) if is_direct(value) => None,
        Some(value) if !value.is_empty() => {
            ProxySettings::detect(Some(value)).map_err(|e| {
                set_last_error(e);
                ERR_INVALID_PROXY
            })?;
            Some(value.to_string())
        }
        _ => Some(String::new()),
    };

    let mut state = lock_state();
    state.proxy = proxy;
    let old_client = state.client.take();
    drop(state);
    drop(old_client);
    Ok(())
}

fn build_client(proxy: Option<&str>) -> Result<Client, String> {
    let builder = Client::builder().tcp_keepalive(Duration::from_secs(30));
    let builder = match proxy {
        // No proxy: also turn off reqwest's own lookup, which follows HTTPS_PROXY
        None => builder.no_proxy(),
        // Replaces reqwest's own lookup, so both Nominal libraries choose the same proxy
        Some(proxy) => {
            let settings = ProxySettings::detect(Some(proxy))?;
            builder.proxy(Proxy::custom(move |url| {
                settings
                    .for_destination(url.scheme(), url.host_str()?)
                    .and_then(proxy_url)
            }))
        }
    };
    builder
        .build()
        .map_err(|e| format!("Failed to initialise HTTP client: {}", e))
}

fn proxy_url(server: &ProxyServer) -> Option<Url> {
    let mut url = Url::parse(&format!("http://{}:{}", server.host, server.port)).ok()?;
    if let Some((user, password)) = &server.credentials {
        url.set_username(user).ok()?;
        url.set_password(Some(password)).ok()?;
    }
    Some(url)
}
