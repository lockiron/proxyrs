# proxyrs

A Rust library to generate free proxies, inspired by and ported from the Go library [freeproxy](https://github.com/soluchok/freeproxy).

It scrapes free proxy lists, verifies them against a test target (default: `httpbin.org`), and provides a stream of valid proxies.

## Features

- **Async**: Built on `tokio` and `reqwest` for high-performance concurrent checking.
- **Providers**: Includes `FreeProxyList` and `CoolProxy` (more can be added easily).
- **Verification**: Automatically verifies proxies before returning them, measuring latency.
- **Filtering**: Filter by proxy type, country, and maximum latency.
- **Detailed Metadata**: Returns provider name, country, proxy type, and latency.
- **Caching**: implement TTL caching to avoid re-verifying recently checked proxies.

## Usage

### As a Library

Add `proxyrs` to your `Cargo.toml`.

```rust
use proxyrs::ProxyGenerator;
use proxyrs::providers::cool_proxy::CoolProxy;
use proxyrs::providers::free_proxy_list::FreeProxyList;
use proxyrs::filter::ProxyFilter;
use proxyrs::proxy::ProxyType;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mut generator = ProxyGenerator::new();
    
    // Add providers
    generator.add_provider(CoolProxy::new());
    generator.add_provider(FreeProxyList::new());

    // Optional: Set a filter
    generator.set_filter(
        ProxyFilter::new()
            .with_proxy_types(vec![ProxyType::Https]) // Only HTTPS
            .with_exclude_countries(vec!["CN".to_owned()]) // Exclude China
            .with_max_latency(Duration::from_secs(2)) // Max 2s latency
    ).await;

    // Start the background generator task
    generator.run().await;

    println!("Searching for proxies...");

    // Get a valid proxy
    if let Some(proxy) = generator.get().await {
        // Prints detailed info: "1.2.3.4:8080 (HTTPS, US) - 123ms via free-proxy-list.net"
        println!("Found proxy: {}", proxy); 
        
        // Access specific fields
        println!("Address: {}", proxy.addr);
        println!("Type: {}", proxy.kind);
        println!("Country: {}", proxy.country);
    }
}
```

### Running Standalone

You can run the included binary to fetch and print a valid proxy:

```bash
# Get any proxy
cargo run --example simple_run

# Get an HTTPS proxy from US
cargo run --example simple_run -- --type https --country US

# Get a proxy excluding US with max latency 500ms
cargo run --example simple_run -- --exclude-country US --max-latency-ms 500

# Set a timeout of 10 seconds
cargo run --example simple_run -- --timeout-s 10

# Get 3 unique proxies
cargo run --example simple_run -- --limit 3
```

This will output something like:
```text
[INFO  proxyrs::proxy_generator] free-proxy-list.net found ips 300
Applying filter: ProxyFilter { proxy_types: Some([Https]), include_countries: Some(["US"]), exclude_countries: None, max_latency: Some(2s) }
Found proxy: 154.3.236.202:3128 (HTTPS, US) - 823.401708ms via free-proxy-list.net
```

## Detailed Information

The `Proxy` object returned by the generator contains:
- `addr`: IP:Port string.
- `kind`: `ProxyType` (HTTP, HTTPS, SOCKS4, SOCKS5).
- `country`: Country code (e.g., "US", "JP").
- `provider`: The name of the provider that found this proxy.
- `latency`: `Duration` representing the response time during verification.

## Supported Proxy Types

When using `ProxyFilter`, you can specify the following types:
- `ProxyType::Http`
- `ProxyType::Https`
- `ProxyType::Socks4`
- `ProxyType::Socks5`
- `ProxyType::Unknown`

## Configuration

The verification timeout is set to **2 seconds** to ensure fast response times, filtering out slow proxies. It checks purely for HTTP connectivity using `http://httpbin.org/ip`.

## License

MIT