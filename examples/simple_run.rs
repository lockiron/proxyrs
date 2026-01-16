use proxyrs::ProxyGenerator;
use proxyrs::providers::cool_proxy::CoolProxy;
use proxyrs::providers::free_proxy_list::FreeProxyList;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let mut generator = ProxyGenerator::new();
    
    // Add providers
    generator.add_provider(CoolProxy::new());
    generator.add_provider(FreeProxyList::new());

    // Run generator
    generator.run().await;

    println!("Starting proxy search... (Wait a bit for proxies to be found)");

    // Try to get some proxies
    let mut count = 0;
    while count < 5 {
        if let Some(proxy) = generator.get().await {
            println!("Found valid proxy: {}", proxy);
            count += 1;
        } else {
             // Wait a bit if no proxy found yet
             tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
