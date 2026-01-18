use std::fmt;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
    Unknown,
}

impl fmt::Display for ProxyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyType::Http => write!(f, "HTTP"),
            ProxyType::Https => write!(f, "HTTPS"),
            ProxyType::Socks4 => write!(f, "SOCKS4"),
            ProxyType::Socks5 => write!(f, "SOCKS5"),
            ProxyType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyMetadata {
    pub addr: String,
    pub kind: ProxyType,
    pub country: String,
}

#[derive(Debug, Clone)]
pub struct Proxy {
    pub addr: String,
    pub kind: ProxyType,
    pub country: String,
    pub provider: String,
    pub latency: Duration,
}

impl fmt::Display for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}, {}) - {:?} via {}",
            self.addr, self.kind, self.country, self.latency, self.provider
        )
    }
}
