use async_trait::async_trait;
use std::error::Error;

pub mod knaben;
pub mod piratebay;

pub use knaben::Knaben;
pub use piratebay::PirateBay;

pub struct Torrent {
    pub name: String,
    pub magnet_link: String,
    pub seeders: Option<u32>,
    pub leechers: Option<u32>,
    pub size_bytes: Option<u64>,
}

#[async_trait]
pub trait SearchProvider {
    async fn search(&self, query: &str) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>>;
}
