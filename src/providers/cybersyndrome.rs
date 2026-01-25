use super::base::BaseProvider;
use super::new_client;
use crate::provider::Provider;
use crate::proxy::{ProxyMetadata, ProxyType};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub struct Cybersyndrome {
    base: BaseProvider,
}

impl Cybersyndrome {
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

        // URL for the Speed Ranking list
        let url = "https://www.cybersyndrome.net/plr6.html";
        let resp = client.get(url).send().await?;
        let body = resp.text().await?;

        // 1. Parse Countries from HTML Table
        // IDs are n1, n2, ... corresponds to index 0, 1...
        let doc = Html::parse_document(&body);
        let tr_selector = Selector::parse("tr").unwrap();
        let td_selector = Selector::parse("td").unwrap();
        
        let mut country_map: HashMap<usize, String> = HashMap::new();

        for tr in doc.select(&tr_selector) {
            let tds: Vec<_> = tr.select(&td_selector).collect();
            // Expected row: <td>Rank</td><td id="n1"></td>...<td>Country</td>
            // Length should be at least 5.
            if tds.len() >= 5 {
                // Check 2nd td for id
                if let Some(id_attr) = tds[1].value().attr("id") {
                    if id_attr.starts_with('n') {
                        if let Ok(idx) = id_attr[1..].parse::<usize>() {
                             // Country is in the 5th td (index 4)
                             let country = tds[4].text().collect::<Vec<_>>().join("").trim().to_string();
                             // ID n1 corresponds to index 0
                             if idx > 0 {
                                 country_map.insert(idx - 1, country);
                             }
                        }
                    }
                }
            }
        }

        // 2. Parse JS variables
        // var as=[...];
        // var ps=[...];
        // var n=(...)%120;

        let re_as = Regex::new(r"var\s+as\s*=\s*\[([^\]]+)\];")?;
        let re_ps = Regex::new(r"var\s+ps\s*=\s*\[([^\]]+)\];")?;
        let re_n = Regex::new(r"var\s+n\s*=\s*\(([^)]+)\)%120;")?;

        let as_str = re_as
            .captures(&body)
            .and_then(|c| c.get(1))
            .ok_or_else(|| anyhow!("Failed to find 'as' array"))?
            .as_str();
        
        let ps_str = re_ps
            .captures(&body)
            .and_then(|c| c.get(1))
            .ok_or_else(|| anyhow!("Failed to find 'ps' array"))?
            .as_str();
            
        let n_expr = re_n
            .captures(&body)
            .and_then(|c| c.get(1))
            .ok_or_else(|| anyhow!("Failed to find 'n' expression"))?
            .as_str();

        // Parse arrays
        let mut as_vec: Vec<i32> = as_str
            .split(',')
            .map(|s| s.trim().parse().unwrap_or(0))
            .collect();
        
        let ps_vec: Vec<i32> = ps_str
            .split(',')
            .map(|s| s.trim().parse().unwrap_or(0))
            .collect();

        // Calculate n
        let n = eval_n(n_expr, &ps_vec)?;
        let n = n % 120;

        if n < as_vec.len() as i32 { // Valid n
             as_vec.rotate_left(n as usize);
        }

        let mut proxies = Vec::new();
        let num_ips = as_vec.len() / 4;

        for j in 0..proxies_len_limit(num_ips, ps_vec.len()) {
            let p1 = as_vec[j * 4];
            let p2 = as_vec[j * 4 + 1];
            let p3 = as_vec[j * 4 + 2];
            let p4 = as_vec[j * 4 + 3];
            let port = ps_vec[j];
            
            let ip = format!("{}.{}.{}.{}", p1, p2, p3, p4);
            let addr = format!("{}:{}", ip, port);

            // Lookup country
            let country = country_map.get(&j).cloned().unwrap_or_else(|| "Unknown".to_string());

            proxies.push(ProxyMetadata {
                addr,
                kind: ProxyType::Http, // Cybersyndrome lists HTTP proxies usually
                country,
            });
        }

        self.base.update_cache(proxies.clone());
        Ok(proxies)
    }
}

fn proxies_len_limit(ips: usize, ports: usize) -> usize {
    if ips < ports { ips } else { ports }
}

fn eval_n(expr: &str, ps: &[i32]) -> Result<i32> {
    // Expression format: "123+456*ps[7]+890"
    // Split by '+'
    let parts: Vec<&str> = expr.split('+').collect();
    let mut sum = 0;

    for part in parts {
        let part = part.trim();
        if part.contains("*ps[") {
             // e.g. "291*ps[22]"
             let subparts: Vec<&str> = part.split('*').collect();
             if subparts.len() != 2 { continue; }
             let coeff: i32 = subparts[0].trim().parse().unwrap_or(0);
             
             // Extract index from "ps[22]"
             let idx_str = subparts[1].trim();
             // Removing "ps[" and "]"
             let start = idx_str.find('[').unwrap_or(0) + 1;
             let end = idx_str.find(']').unwrap_or(idx_str.len());
             if start >= end { continue; }
             
             let idx: usize = idx_str[start..end].parse().unwrap_or(0);
             let val = if idx < ps.len() { ps[idx] } else { 0 };
             
             sum += coeff * val;
        } else {
             // just a number
             let val: i32 = part.parse().unwrap_or(0);
             sum += val;
        }
    }
    Ok(sum)
}

#[async_trait]
impl Provider for Cybersyndrome {
    async fn list(&mut self) -> Result<Vec<ProxyMetadata>> {
        self.load_internal().await
    }

    fn name(&self) -> &'static str {
        "www.cybersyndrome.net"
    }

    fn set_proxy(&mut self, proxy: String) {
        self.base.proxy_upstream = proxy;
    }
}
