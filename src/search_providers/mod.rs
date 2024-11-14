use crate::{errors::ClientError, SearchRequest, Torrent};
use async_trait::async_trait;

pub mod knaben;
pub mod piratebay;

pub use knaben::Knaben;
pub use piratebay::PirateBay;

#[async_trait]
pub trait SearchProvider {
    async fn search(&self, req: SearchRequest<'_>) -> Result<Vec<Torrent>, ClientError>;
}
