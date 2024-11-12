use crate::{SearchRequest, Torrent};
use async_trait::async_trait;
use std::error::Error;

pub mod knaben;
pub mod piratebay;

pub use knaben::Knaben;
pub use piratebay::PirateBay;

#[async_trait]
pub trait SearchProvider {
    async fn search(
        &self,
        req: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>>;
}
