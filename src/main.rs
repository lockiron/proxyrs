use proxyrs::ProxyGenerator;
use proxyrs::providers::cool_proxy::CoolProxy;
use proxyrs::providers::free_proxy_list::FreeProxyList;
use proxyrs::providers::cybersyndrome::Cybersyndrome;
use proxyrs::providers::proxyscrape::ProxyScrape;
use proxyrs::configuration::Settings;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn) // Default warn
        .filter_module("proxyrs", log::LevelFilter::Debug) // proxyrs debug
        .init();

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
            log::warn!("Failed to load config.toml: {}. Using default providers.", e);
             // Default fallback
            generator.add_provider(Cybersyndrome::new());
            generator.add_provider(ProxyScrape::new());
            generator.add_provider(CoolProxy::new());
            generator.add_provider(FreeProxyList::new());
        }
    }

    // Start the generator
    generator.run().await;

    // Get a proxy (blocks until one is found)
    if let Some(proxy) = generator.get().await {
        println!("{}", proxy);
    }
}
