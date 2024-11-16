use async_trait::async_trait;

use crate::{errors::ClientError, http_client::HttpClient, SearchRequest, Torrent};

pub mod knaben;
pub mod piratebay;

pub use knaben::Knaben;
pub use piratebay::PirateBay;

#[async_trait]
pub trait SearchProvider {
    async fn request_torrents(
        &self,
        client: &HttpClient,
        request: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, ClientError>;
    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError>;
}
