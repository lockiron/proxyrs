use crate::filter::ProxyFilter;
use crate::provider::Provider;
use crate::proxy::{Proxy, ProxyMetadata};

use log::{error, info};
use moka::future::Cache;
use rand::seq::SliceRandom;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use std::time::Duration;

use futures::future::BoxFuture;

pub type VerifyFn = fn(&str) -> BoxFuture<'static, bool>;

pub struct ProxyGenerator {
    cache: Cache<String, Option<Duration>>, // Cache stores latency if valid
    filter: Arc<Mutex<ProxyFilter>>,
    providers: Vec<Arc<Mutex<dyn Provider>>>,
    proxy_tx: Sender<Proxy>,
    proxy_rx: Arc<Mutex<Receiver<Proxy>>>,
    job_tx: Sender<(ProxyMetadata, String)>, // Metadata and Provider Name
    last_valid_proxy: Arc<Mutex<Option<Proxy>>>,
}

impl ProxyGenerator {
    pub fn new() -> Self {
        let (proxy_tx, proxy_rx) = mpsc::channel(100);
        let (job_tx, mut job_rx) = mpsc::channel::<(ProxyMetadata, String)>(100);

        let generator = Self {
            cache: Cache::builder()
                .time_to_live(Duration::from_secs(20 * 60))
                .build(),
            filter: Arc::new(Mutex::new(ProxyFilter::default())),
            providers: Vec::new(),
            proxy_tx,
            proxy_rx: Arc::new(Mutex::new(proxy_rx)),
            job_tx,
            last_valid_proxy: Arc::new(Mutex::new(None)),
        };

        // Spawn workers
        let cache_clone = generator.cache.clone();
        let proxy_tx_clone = generator.proxy_tx.clone();
        let filter_clone = generator.filter.clone();
        
        tokio::spawn(async move {
            while let Some((metadata, provider_name)) = job_rx.recv().await {
                let cache = cache_clone.clone();
                let tx = proxy_tx_clone.clone();
                let filter = filter_clone.clone();
                tokio::spawn(async move {
                    if let Some(latency) = verify_with_cache(cache, &metadata.addr).await {
                         let proxy = Proxy {
                             addr: metadata.addr,
                             kind: metadata.kind,
                             country: metadata.country,
                             provider: provider_name,
                             latency,
                         };
                         
                         // Post-verification filter (e.g. Latency)
                         let filter = filter.lock().await;
                         if filter.filter_proxy(&proxy) {
                            let _ = tx.send(proxy).await;
                         }
                    }
                });
            }
        });

        generator
    }

    pub async fn set_filter(&self, filter: ProxyFilter) {
        let mut f = self.filter.lock().await;
        *f = filter;
    }

    pub fn add_provider<P: Provider + 'static>(&mut self, provider: P) {
        self.providers.push(Arc::new(Mutex::new(provider)));
    }

    pub async fn run(&self) {
        let providers = self.providers.clone();
        let job_tx = self.job_tx.clone();
        let last_valid_proxy = self.last_valid_proxy.clone();
        let filter_mutex = self.filter.clone();

        tokio::spawn(async move {
            loop {
                for provider in &providers {
                    let mut provider_guard = provider.lock().await;
                    let last_valid = last_valid_proxy.lock().await.clone();
                    if let Some(valid_proxy) = last_valid {
                         provider_guard.set_proxy(valid_proxy.addr);
                    }

                    match provider_guard.list().await {
                        Ok(mut proxies) => {
                            // Pre-verification filter (Type, Country)
                            {
                                let filter = filter_mutex.lock().await;
                                proxies.retain(|meta| filter.filter_metadata(meta));
                            }
                        
                            info!("{} found ips {}", provider_guard.name(), proxies.len());
                            {
                                let mut rng = rand::thread_rng();
                                proxies.shuffle(&mut rng);
                            }
                            
                            // Limit to 10 proxies to avoid overwhelming
                            proxies.truncate(10);
                            
                            for proxy in proxies {
                                if let Err(e) = job_tx.send((proxy, provider_guard.name().to_string())).await {
                                     error!("failed to send job: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                             error!("cannot load list of proxy {} err:{}", provider_guard.name(), e);
                             let mut last_valid = last_valid_proxy.lock().await;
                             *last_valid = None;
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await; // Loop frequently to replenish jobs
            }
        });
    }

    pub async fn get(&self) -> Option<Proxy> {
        let mut rx = self.proxy_rx.lock().await;
        if let Some(proxy) = rx.recv().await {
             let mut last_valid = self.last_valid_proxy.lock().await;
             *last_valid = Some(proxy.clone());
             return Some(proxy);
        }
        None
    }
}

async fn verify_with_cache(cache: Cache<String, Option<Duration>>, proxy: &str) -> Option<Duration> {
    if let Some(val) = cache.get(proxy).await {
        return val;
    }
    
    // Use the verification logic from verification mod
    let res = crate::verification::verify_proxy(proxy).await;
    // Cache the result (Some(duration) or None). 
    // If None (invalid), we might not want to cache it forever, but for now we do to avoid retrying bad proxies immediately.
    // Actually if it's bad, maybe we don't cache correct latency, but 'None' implies bad.
    // However, the cache type is Option<Duration>.
    cache.insert(proxy.to_string(), res).await;
    res
}
