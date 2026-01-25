pub mod base;
pub mod cool_proxy;
pub mod free_proxy_list;
pub mod cybersyndrome;
pub mod proxyscrape;

// Common HTTP client construction if needed, or re-use logic
use reqwest::Client;
use std::time::Duration;

pub fn new_client(proxy: Option<&str>) -> anyhow::Result<Client> {
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(10));
    
    if let Some(p) = proxy {
         if !p.is_empty() {
             let proxy_url = reqwest::Proxy::http(&format!("http://{}", p))?;
             builder = builder.proxy(proxy_url);
         }
    }

    Ok(builder.build()?)
}
