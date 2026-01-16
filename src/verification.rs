use log::{debug, error};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct CheckIP {
    origin: String,
}

pub async fn verify_proxy(proxy: &str) -> bool {
    // ... proxy setup matches ...
    let proxy_url = match reqwest::Proxy::http(&format!("http://{}", proxy)) {
        Ok(url) => url,
        Err(e) => {
            error!("cannot parse proxy {}: {}", proxy, e);
            return false;
        }
    };

    let client = match Client::builder()
        .proxy(proxy_url)
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            error!("cannot build client for verify {}: {}", proxy, e);
            return false;
        }
    };

    // Use http instead of https to support HTTP-only proxies
    let resp = match client.get("http://httpbin.org/ip").send().await {
        Ok(resp) => resp,
        Err(e) => {
            debug!("cannot verify proxy {}: {}", proxy, e);
            return false;
        }
    };

    if !resp.status().is_success() {
        return false;
    }

    let check: CheckIP = match resp.json().await {
        Ok(c) => c,
        Err(e) => {
            error!("cannot unmarshal checkIP for {}: {}", proxy, e);
            return false;
        }
    };

    // httpbin returns "IP1, IP2" if multiple, or just "IP".
    // We just check if our proxy IP is contained in the origin string.
    // The previous logic was checking prefix.
    check.origin.contains(proxy.split(':').next().unwrap_or(""))
}
