use log::{debug, error};
use reqwest::Client;
use serde::Deserialize;
use std::net::{IpAddr, ToSocketAddrs};
use std::time::Duration;

#[derive(Deserialize)]
struct CheckIP {
    origin: String,
}

fn is_safe_ip(ip: IpAddr) -> bool {
    // Check for loopback and other unsafe ranges
    if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
        return false;
    }

    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // private networks
            // 10.0.0.0/8
            if octets[0] == 10 { return false; }
            // 172.16.0.0/12
            if octets[0] == 172 && (16..=31).contains(&octets[1]) { return false; }
            // 192.168.0.0/16
            if octets[0] == 192 && octets[1] == 168 { return false; }
            // link-local 169.254.0.0/16
            if octets[0] == 169 && octets[1] == 254 { return false; }
            // broadcast
            if ipv4.is_broadcast() { return false; }
            
            true
        },
        IpAddr::V6(ipv6) => {
            // unique local (fc00::/7)
            if (ipv6.segments()[0] & 0xfe00) == 0xfc00 { return false; }
            // link-local (fe80::/10)
            if (ipv6.segments()[0] & 0xffc0) == 0xfe80 { return false; }
            
            true
        }
    }
}

pub async fn verify_proxy(proxy: &str) -> Option<Duration> {
    // strict parsing
    let addr = match proxy.to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(a) => a,
            None => {
                error!("cannot resolve proxy address {}", proxy);
                return None;
            }
        },
        Err(e) => {
            error!("cannot parse proxy address {}: {}", proxy, e);
            return None;
        }
    };

    if !is_safe_ip(addr.ip()) {
        error!("unsafe proxy ip refused: {}", addr.ip());
        return None;
    }

    let proxy_url = match reqwest::Proxy::http(&format!("http://{}", proxy)) {
        Ok(url) => url,
        Err(e) => {
            error!("cannot parse proxy {}: {}", proxy, e);
            return None;
        }
    };

    let client = match Client::builder()
        .proxy(proxy_url)
        .timeout(Duration::from_secs(2))
        .pool_max_idle_per_host(0) // Disable pooling for one-off requests
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            error!("cannot build client for verify {}: {}", proxy, e);
            return None;
        }
    };

    let start = std::time::Instant::now();
    // Use http instead of https to support HTTP-only proxies
    let resp = match client.get("http://httpbin.org/ip").send().await {
        Ok(resp) => resp,
        Err(e) => {
            debug!("cannot verify proxy {}: {}", proxy, e);
            return None;
        }
    };

    if !resp.status().is_success() {
        return None;
    }

    let check: CheckIP = match resp.json().await {
        Ok(c) => c,
        Err(e) => {
            error!("cannot unmarshal checkIP for {}: {}", proxy, e);
            return None;
        }
    };

    // httpbin returns "IP1, IP2" if multiple, or just "IP".
    // We just check if our proxy IP is contained in the origin string.
    // The previous logic was checking prefix.
    if check.origin.contains(proxy.split(':').next().unwrap_or("")) {
        return Some(start.elapsed());
    }
    None
}
