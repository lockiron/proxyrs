use serde::Deserialize;
use std::fs;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub providers: Providers,
}

#[derive(Debug, Deserialize)]
pub struct Providers {
    pub cool_proxy: Option<ProviderConfig>,
    pub free_proxy_list: Option<ProviderConfig>,
    pub cybersyndrome: Option<CybersyndromeConfig>,
    pub proxyscrape: Option<ProviderConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CybersyndromeConfig {
    pub enabled: bool,
    pub url: String,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let config_data = fs::read_to_string("config.toml").unwrap_or_else(|_| "".to_string());
        
        // If config file is missing or empty, use default values (mocked by deserializing partial or using defaults if I implied Default trait, but here I'll just check existence)
        // Actually, let's just default if missing.
        if config_data.is_empty() {
             return Ok(Settings {
                 providers: Providers {
                     cool_proxy: Some(ProviderConfig { enabled: true, url: None }),
                     free_proxy_list: Some(ProviderConfig { enabled: true, url: None }),
                     cybersyndrome: Some(CybersyndromeConfig { enabled: true, url: "https://www.cybersyndrome.net/plr6.html".to_string() }),
                     proxyscrape: Some(ProviderConfig { enabled: true, url: None }),
                 }
             });
        }

        let settings: Settings = toml::from_str(&config_data)?;
        Ok(settings)
    }
}
