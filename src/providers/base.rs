use crate::proxy::ProxyMetadata;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct BaseProvider {
    pub proxy_upstream: String, // Upstream proxy for scraping requests
    pub proxy_list: Vec<ProxyMetadata>,
    pub last_update: Option<Instant>,
    pub ttl: Duration,
}

impl BaseProvider {
    pub fn new() -> Self {
        Self {
            proxy_upstream: String::new(),
            proxy_list: Vec::new(),
            last_update: None,
            ttl: Duration::from_secs(60 * 20), // Default 20 min TTL
        }
    }

    pub fn should_update(&self) -> bool {
        if let Some(last) = self.last_update {
            if last.elapsed() < self.ttl && !self.proxy_list.is_empty() {
                return false;
            }
        }
        true
    }

    pub fn update_cache(&mut self, list: Vec<ProxyMetadata>) {
        self.proxy_list = list;
        self.last_update = Some(Instant::now());
    }
    
    pub fn cached_list(&self) -> Vec<ProxyMetadata> {
        self.proxy_list.clone()
    }
}

impl Default for BaseProvider {
    fn default() -> Self {
        Self::new()
    }
}
