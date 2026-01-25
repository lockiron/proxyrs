use clap::{Parser, ValueEnum};
use proxyrs::ProxyGenerator;
use proxyrs::filter::ProxyFilter;
use proxyrs::providers::cool_proxy::CoolProxy;
use proxyrs::providers::free_proxy_list::FreeProxyList;
use proxyrs::providers::cybersyndrome::Cybersyndrome;
use proxyrs::providers::proxyscrape::ProxyScrape;
use proxyrs::configuration::Settings;
use proxyrs::proxy::ProxyType;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Proxy types to include (e.g. http, https)
    #[arg(long, value_enum)]
    type_: Option<Vec<ProxyTypeArg>>,

    /// Countries to include (e.g. US, JP)
    #[arg(long)]
    country: Option<Vec<String>>,

    /// Countries to exclude
    #[arg(long)]
    exclude_country: Option<Vec<String>>,

    /// Maximum latency in milliseconds
    #[arg(long, default_value_t = 2000)]
    max_latency_ms: u64,

    /// Timeout in seconds for the entire search
    #[arg(long, default_value_t = 60)]
    timeout_s: u64,

    /// Number of unique proxies to find
    #[arg(long, default_value_t = 1)]
    limit: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProxyTypeArg {
    Http,
    Https,
    Socks4,
    Socks5,
}

impl From<ProxyTypeArg> for ProxyType {
    fn from(arg: ProxyTypeArg) -> Self {
        match arg {
            ProxyTypeArg::Http => ProxyType::Http,
            ProxyTypeArg::Https => ProxyType::Https,
            ProxyTypeArg::Socks4 => ProxyType::Socks4,
            ProxyTypeArg::Socks5 => ProxyType::Socks5,
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let mut generator = ProxyGenerator::new();
    
    // Load configuration
    match Settings::new() {
        Ok(settings) => {
             // Cybersyndrome
            if let Some(conf) = settings.providers.cybersyndrome {
                if conf.enabled {
                    generator.add_provider(Cybersyndrome::new());
                }
            }

            // ProxyScrape
            if let Some(conf) = settings.providers.proxyscrape {
                if conf.enabled {
                    generator.add_provider(ProxyScrape::new());
                }
            }

            // CoolProxy
            if let Some(conf) = settings.providers.cool_proxy {
                 if conf.enabled {
                    generator.add_provider(CoolProxy::new());
                }
            }

            // FreeProxyList
            if let Some(conf) = settings.providers.free_proxy_list {
                 if conf.enabled {
                    generator.add_provider(FreeProxyList::new());
                }
            }
        },
        Err(e) => {
             println!("Warning: Failed to load config.toml: {}. Using default providers.", e);
             generator.add_provider(Cybersyndrome::new());
             generator.add_provider(ProxyScrape::new());
             generator.add_provider(CoolProxy::new());
             generator.add_provider(FreeProxyList::new());
        }
    }

    // Build filter from args
    let mut filter = ProxyFilter::new()
        .with_max_latency(Duration::from_millis(args.max_latency_ms));

    if let Some(types) = args.type_ {
        filter = filter.with_proxy_types(types.into_iter().map(Into::into).collect());
    }

    if let Some(countries) = args.country {
        filter = filter.with_include_countries(countries);
    }

    if let Some(excludes) = args.exclude_country {
        filter = filter.with_exclude_countries(excludes);
    }

    println!("Applying filter: {:?}", filter);
    generator.set_filter(filter).await;

    // Run generator
    generator.run().await;

    println!("Starting proxy search... (Timeout: {}s, Limit: {})", args.timeout_s, args.limit);

    let search_future = async {
        let mut count = 0;
        let mut seen = std::collections::HashSet::new();
        while count < args.limit {
            if let Some(proxy) = generator.get().await {
                if seen.insert(proxy.addr.clone()) {
                    println!("Found valid proxy: {}", proxy);
                    count += 1;
                }
            } else {
                 tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    };

    if let Err(_) = tokio::time::timeout(Duration::from_secs(args.timeout_s), search_future).await {
        println!("Timeout reached! Could not find enough proxies matching your criteria within {} seconds.", args.timeout_s);
    }
}
