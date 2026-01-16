use crate::provider::Provider;

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
    cache: Cache<String, bool>,
    providers: Vec<Arc<Mutex<dyn Provider>>>,
    proxy_tx: Sender<String>,
    proxy_rx: Arc<Mutex<Receiver<String>>>,
    job_tx: Sender<String>,
    last_valid_proxy: Arc<Mutex<String>>,
}

impl ProxyGenerator {
    pub fn new() -> Self {
        let (proxy_tx, proxy_rx) = mpsc::channel(100);
        let (job_tx, mut job_rx) = mpsc::channel(100);

        let generator = Self {
            cache: Cache::builder()
                .time_to_live(Duration::from_secs(20 * 60))
                .build(),
            providers: Vec::new(),
            proxy_tx,
            proxy_rx: Arc::new(Mutex::new(proxy_rx)),
            job_tx,
            last_valid_proxy: Arc::new(Mutex::new(String::new())),
        };

        // Spawn workers
        let cache_clone = generator.cache.clone();
        let proxy_tx_clone = generator.proxy_tx.clone();
        
        tokio::spawn(async move {
            while let Some(proxy) = job_rx.recv().await {
                let cache = cache_clone.clone();
                let tx = proxy_tx_clone.clone();
                tokio::spawn(async move {
                    if verify_with_cache(cache, proxy.clone()).await {
                        let _ = tx.send(proxy).await;
                    }
                });
            }
        });

        generator
    }

    pub fn add_provider<P: Provider + 'static>(&mut self, provider: P) {
        self.providers.push(Arc::new(Mutex::new(provider)));
    }

    pub async fn run(&self) {
        let providers = self.providers.clone();
        let job_tx = self.job_tx.clone();
        let last_valid_proxy = self.last_valid_proxy.clone();

        tokio::spawn(async move {
            loop {
                for provider in &providers {
                    let mut provider_guard = provider.lock().await;
                    let last_valid = last_valid_proxy.lock().await.clone();
                    if !last_valid.is_empty() {
                         provider_guard.set_proxy(last_valid);
                    }

                    match provider_guard.list().await {
                        Ok(mut ips) => {
                            info!("{} found ips {}", provider_guard.name(), ips.len());
                            {
                                let mut rng = rand::thread_rng();
                                ips.shuffle(&mut rng);
                            }
                            
                            // Limit to 10 proxies to avoid overwhelming
                            ips.truncate(10);
                            
                            for ip in ips {
                                if let Err(e) = job_tx.send(ip).await {
                                     error!("failed to send job: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                             error!("cannot load list of proxy {} err:{}", provider_guard.name(), e);
                             let mut last_valid = last_valid_proxy.lock().await;
                             *last_valid = String::new();
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(60)).await; // avoid tight loop if all fail
            }
        });
    }

    pub async fn get(&self) -> Option<String> {
        let mut rx = self.proxy_rx.lock().await;
        if let Some(proxy) = rx.recv().await {
             let mut last_valid = self.last_valid_proxy.lock().await;
             *last_valid = proxy.clone();
             return Some(proxy);
        }
        None
    }
}

async fn verify_with_cache(cache: Cache<String, bool>, proxy: String) -> bool {
    if let Some(val) = cache.get(&proxy).await {
        return val;
    }
    
    // Use the verification logic from verification mod
    let res = crate::verification::verify_proxy(&proxy).await;
    cache.insert(proxy, res).await;
    res
}
