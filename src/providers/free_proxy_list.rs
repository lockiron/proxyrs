use super::new_client;
use crate::provider::Provider;
use crate::proxy::{ProxyMetadata, ProxyType};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use scraper::{Html, Selector};
use std::time::{Duration, Instant};

const FREE_PROXY_LIST_URL: &str = "https://free-proxy-list.net/";

pub struct FreeProxyList {
    proxy: String,
    proxy_list: Vec<ProxyMetadata>,
    last_update: Option<Instant>,
}

impl FreeProxyList {
    pub fn new() -> Self {
        Self {
            proxy: String::new(),
            proxy_list: Vec::new(),
            last_update: None,
        }
    }

    async fn load_internal(&mut self) -> Result<Vec<ProxyMetadata>> {
        if let Some(last) = self.last_update {
             if last.elapsed() < Duration::from_secs(60 * 20) && !self.proxy_list.is_empty() {
                 return Ok(self.proxy_list.clone());
             }
        }

        let client = new_client(if self.proxy.is_empty() { None } else { Some(&self.proxy) })?;
        
        let resp = client.get(FREE_PROXY_LIST_URL)
            .header("Accept-Language", "en-US,en;q=0.8")
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36")
            .send().await?;
            
        let body = resp.text().await?;

        let doc = Html::parse_document(&body);
        let row_selector = Selector::parse(r#"#list table tbody tr"#).unwrap();
        let row_selector_alt = Selector::parse(r#"table tbody tr"#).unwrap();
        
        let mut rows: Vec<_> = doc.select(&row_selector).collect();
        if rows.is_empty() {
             rows = doc.select(&row_selector_alt).collect();
        }

        let mut result = Vec::new();
        let td_selector = Selector::parse("td").unwrap();

        for row in rows {
            let cols: Vec<_> = row.select(&td_selector).collect();
            if cols.len() >= 8 {
                let ip = cols[0].inner_html();
                let port = cols[1].inner_html();
                let country_code = cols[2].inner_html(); // 3rd column: Code
                let https = cols[6].inner_html();       // 7th column: Https (yes/no)

                let kind = if https.to_lowercase() == "yes" {
                    ProxyType::Https
                } else {
                    ProxyType::Http
                };

                result.push(ProxyMetadata {
                    addr: format!("{}:{}", ip, port),
                    kind,
                    country: country_code,
                });
            }
        }

        if result.is_empty() {
            return Err(anyhow!("proxies not found"));
        }

        self.proxy_list = result.clone();
        self.last_update = Some(Instant::now());
        Ok(result)
    }
}

#[async_trait]
impl Provider for FreeProxyList {
    async fn list(&mut self) -> Result<Vec<ProxyMetadata>> {
        self.load_internal().await
    }

    fn name(&self) -> &'static str {
        "free-proxy-list.net"
    }

    fn set_proxy(&mut self, proxy: String) {
        self.proxy = proxy;
    }
}
