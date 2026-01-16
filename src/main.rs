use proxyrs::ProxyGenerator;
use proxyrs::providers::cool_proxy::CoolProxy;
use proxyrs::providers::free_proxy_list::FreeProxyList;

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn) // Default warn
        .filter_module("proxyrs", log::LevelFilter::Debug) // proxyrs debug
        .init();

    let mut generator = ProxyGenerator::new();
    
    // In Go, New() adds providers by default. Here we add them explicitly.
    generator.add_provider(CoolProxy::new());
    generator.add_provider(FreeProxyList::new());

    // Start the generator
    generator.run().await;

    // Get a proxy (blocks until one is found)
    if let Some(proxy) = generator.get().await {
        println!("{}", proxy);
    }
}
