use super::new_client;
use crate::provider::Provider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use scraper::{Html, Selector};
use std::time::{Duration, Instant};

const COOL_PROXY_URL: &str = "https://www.cool-proxy.net/proxies/http_proxy_list/sort:score/direction:desc";

pub struct CoolProxy {
    proxy: String,
    proxy_list: Vec<String>,
    last_update: Option<Instant>,
}

impl CoolProxy {
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
        let resp = client.get(COOL_PROXY_URL).send().await?;
        let body = resp.text().await?;

        let doc = Html::parse_document(&body);
        let ip_selector = Selector::parse(r#"#main table tr td:nth-child(1):not([colspan]) script"#).unwrap();
        let port_selector = Selector::parse(r#"#main table tr td:nth-child(2)"#).unwrap();

        let mut ips = Vec::new();
        for element in doc.select(&ip_selector) {
            ips.push(element.inner_html());
        }

        let mut ports = Vec::new();
        for element in doc.select(&port_selector) {
            ports.push(element.inner_html());
        }

        if ips.is_empty() {
            return Err(anyhow!("ip not found"));
        }
        if ips.len() != ports.len() {
             return Err(anyhow!("len port not equal ip"));
        }

        let re = Regex::new(r#""(.*?[^\\])""#)?;
        let mut result = Vec::new();

        for (i, ip_script) in ips.iter().enumerate() {
            if let Some(captures) = re.captures(ip_script) {
                if let Some(encoded) = captures.get(1) {
                     let rot13_decoded: String = encoded.as_str().chars().map(rot13).collect();
                     if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(rot13_decoded) {
                         if let Ok(ip) = String::from_utf8(decoded_bytes) {
                             result.push(format!("{}:{}", ip, ports[i]));
                         }
                     }
                }
            }
        }

        self.proxy_list = result.clone();
        self.last_update = Some(Instant::now());
        Ok(result)
    }
}

#[async_trait]
impl Provider for CoolProxy {
    async fn list(&mut self) -> Result<Vec<String>> {
        self.load_internal().await
    }

    fn name(&self) -> &'static str {
        "www.cool-proxy.net"
    }

    fn set_proxy(&mut self, proxy: String) {
        self.proxy = proxy;
    }
}

// Actual implementation that matches expected updated trait
impl CoolProxy {
     // helper to satisfy my plan to update trait
}

fn rot13(c: char) -> char {
    match c {
        'A'..='Z' => ((c as u8 - b'A' + 13) % 26 + b'A') as char,
        'a'..='z' => ((c as u8 - b'a' + 13) % 26 + b'a') as char,
        _ => c,
    }
}
