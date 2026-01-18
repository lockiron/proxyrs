use crate::proxy::{Proxy, ProxyMetadata, ProxyType};
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct ProxyFilter {
    pub proxy_types: Option<Vec<ProxyType>>,
    pub include_countries: Option<Vec<String>>,
    pub exclude_countries: Option<Vec<String>>,
    pub max_latency: Option<Duration>,
}

impl ProxyFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_proxy_types(mut self, types: Vec<ProxyType>) -> Self {
        self.proxy_types = Some(types);
        self
    }

    pub fn with_include_countries(mut self, countries: Vec<String>) -> Self {
        self.include_countries = Some(countries);
        self
    }

    pub fn with_exclude_countries(mut self, countries: Vec<String>) -> Self {
        self.exclude_countries = Some(countries);
        self
    }

    pub fn with_max_latency(mut self, latency: Duration) -> Self {
        self.max_latency = Some(latency);
        self
    }

    pub fn filter_metadata(&self, meta: &ProxyMetadata) -> bool {
        if let Some(types) = &self.proxy_types {
            if !types.contains(&meta.kind) {
                return false;
            }
        }

        if let Some(includes) = &self.include_countries {
            // Case-insensitive comparison? Let's assume input matches implementation (usually uppercase codes)
            // But to be safe, exact match for now.
            if !includes.iter().any(|c| c.eq_ignore_ascii_case(&meta.country)) {
                return false;
            }
        }

        if let Some(excludes) = &self.exclude_countries {
            if excludes.iter().any(|c| c.eq_ignore_ascii_case(&meta.country)) {
                return false;
            }
        }

        true
    }

    pub fn filter_proxy(&self, proxy: &Proxy) -> bool {
        // Metadata filters
        let meta = ProxyMetadata {
            addr: proxy.addr.clone(),
            kind: proxy.kind.clone(),
            country: proxy.country.clone(),
        };

        if !self.filter_metadata(&meta) {
            return false;
        }

        // Latency filter
        if let Some(max) = self.max_latency {
            if proxy.latency > max {
                return false;
            }
        }

        true
    }
}
