use super::base::BaseProvider;
use super::new_client;
use crate::provider::Provider;
use crate::proxy::{ProxyMetadata, ProxyType};
use anyhow::Result;
use async_trait::async_trait;

pub struct ProxyScrape {
    base: BaseProvider,
}

impl ProxyScrape {
    pub fn new() -> Self {
        Self {
            base: BaseProvider::new(),
        }
    }

    async fn load_internal(&mut self) -> Result<Vec<ProxyMetadata>> {
        if !self.base.should_update() {
            return Ok(self.base.cached_list());
        }

        let client = new_client(if self.base.proxy_upstream.is_empty() {
            None
        } else {
            Some(&self.base.proxy_upstream)
        })?;

        // ProxyScrape API for HTTP proxies
        let url = "https://api.proxyscrape.com/v2/?request=getproxies&protocol=http&timeout=10000&country=all&ssl=all&anonymity=all";
        let resp = client.get(url).send().await?;
        let body = resp.text().await?;

        let mut proxies = Vec::new();
        for line in body.lines() {
            let line = line.trim();
            if !line.is_empty() && line.contains(':') {
                proxies.push(ProxyMetadata {
                    addr: line.to_string(),
                    kind: ProxyType::Http,
                    country: "Unknown".to_string(), // ProxyScrape simple API doesn't return country in this format
                });
            }
        }

        self.base.update_cache(proxies.clone());
        Ok(proxies)
    }
}

#[async_trait]
impl Provider for ProxyScrape {
    async fn list(&mut self) -> Result<Vec<ProxyMetadata>> {
        self.load_internal().await
    }

    fn name(&self) -> &'static str {
        "api.proxyscrape.com"
    }

    fn set_proxy(&mut self, proxy: String) {
        self.base.proxy_upstream = proxy;
    }
}
