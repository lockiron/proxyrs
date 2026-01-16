use super::new_client;
use crate::provider::Provider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use scraper::{Html, Selector};
use std::time::{Duration, Instant};

const FREE_PROXY_LIST_URL: &str = "https://free-proxy-list.net/";

pub struct FreeProxyList {
    proxy: String,
    proxy_list: Vec<String>,
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

    async fn load_internal(&mut self) -> Result<Vec<String>> {
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
        let ip_selector = Selector::parse(r#"#list table tbody tr td:nth-child(1)"#).unwrap();
        // Fallback selector for different layout which sometimes happens on this site
        let ip_selector_alt = Selector::parse(r#"table tbody tr td:nth-child(1)"#).unwrap();
        
        let mut ips = Vec::new();
        // Try precise selector first
        for element in doc.select(&ip_selector) {
            ips.push(element.inner_html());
        }
        if ips.is_empty() {
             for element in doc.select(&ip_selector_alt) {
                ips.push(element.inner_html());
            }
        }

        let port_selector = Selector::parse(r#"#list table tbody tr td:nth-child(2)"#).unwrap();
        let port_selector_alt = Selector::parse(r#"table tbody tr td:nth-child(2)"#).unwrap();
        
        let mut ports = Vec::new();
        for element in doc.select(&port_selector) {
            ports.push(element.inner_html());
        }
        if ports.is_empty() {
             for element in doc.select(&port_selector_alt) {
                ports.push(element.inner_html());
            }
        }

        if ips.is_empty() {
            return Err(anyhow!("ip not found"));
        }
        if ips.len() != ports.len() {
             // Sometimes table structure is weird, let's just take min length
             // or error. Go code errors.
             return Err(anyhow!("len port not equal ip: {} vs {}", ips.len(), ports.len()));
        }

        let mut result = Vec::new();
        for (i, ip) in ips.iter().enumerate() {
            result.push(format!("{}:{}", ip, ports[i]));
        }

        self.proxy_list = result.clone();
        self.last_update = Some(Instant::now());
        Ok(result)
    }
}

#[async_trait]
impl Provider for FreeProxyList {
    async fn list(&mut self) -> Result<Vec<String>> {
        self.load_internal().await
    }

    fn name(&self) -> &'static str {
        "free-proxy-list.net"
    }

    fn set_proxy(&mut self, proxy: String) {
        self.proxy = proxy;
    }
}
