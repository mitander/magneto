use crate::{http_client, SearchProvider, SearchRequest, Torrent};
use serde::Deserialize;

use async_trait::async_trait;
use std::error::Error;

pub struct PirateBay {}

impl Default for PirateBay {
    fn default() -> Self {
        Self::new()
    }
}

impl PirateBay {
    pub fn new() -> PirateBay {
        PirateBay {}
    }
}

#[async_trait]
impl SearchProvider for PirateBay {
    async fn search(
        &self,
        req: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>> {
        let client = http_client::HttpClient::new();
        let url = "https://apibay.org/q.php?q=".parse().unwrap();
        let body = client.send_get_request(url, req.query).await.unwrap();
        let response: Vec<Entry> = serde_json::from_str(&body)?;
        handle_response(response)
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub info_hash: String,
    pub leechers: String,
    pub seeders: String,
    pub num_files: String,
    pub size: String,
    pub username: String,
    pub added: String,
    pub status: String,
    pub category: String,
    pub imdb: String,
}

impl Entry {
    fn filter(self: &Entry) -> bool {
        const EMPTY_ID: &str = "0";
        const EMPTY_NAME: &str = "No results returned";
        const EMPTY_HASH: &str = "0000000000000000000000000000000000000000";
        self.id != EMPTY_ID && self.name != EMPTY_NAME && self.info_hash != EMPTY_HASH
    }
}

fn handle_response(response: Vec<Entry>) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>> {
    Ok(response
        .iter()
        .filter(|entry| entry.filter())
        .map(|entry| Torrent {
            name: entry.name.clone(),
            magnet_link: format!("magnet:?xt=urn:btih:{}", entry.info_hash),
            seeders: entry.seeders.parse().ok(),
            leechers: entry.leechers.parse().ok(),
            size_bytes: entry.size.parse().ok(),
        })
        .collect())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse() {
        todo!();
    }

    #[test]
    fn test_parse_empty() {
        todo!();
    }
}
