use crate::proxy::ProxyMetadata;
use async_trait::async_trait;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn list(&mut self) -> anyhow::Result<Vec<ProxyMetadata>>;
    fn name(&self) -> &'static str;
    fn set_proxy(&mut self, proxy: String);
}
