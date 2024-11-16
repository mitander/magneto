use async_trait::async_trait;
use serde::Deserialize;

use crate::{
    errors::ClientError,
    http_client::{HttpClient, RequestType},
    SearchProvider, SearchRequest, Torrent,
};

#[derive(Default)]
pub struct PirateBay;

impl PirateBay {
    pub fn new() -> PirateBay {
        PirateBay {}
    }
}

#[async_trait]
impl SearchProvider for PirateBay {
    async fn request_torrents(
        &self,
        client: &HttpClient,
        request: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, ClientError> {
        let query = &[("q", request.query)];
        let raw_response = client
            .request("https://apibay.org/q.php?", RequestType::Get(query))
            .await?;
        self.parse_response(&raw_response)
    }

    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
        let response: Vec<Entry> = serde_json::from_str(response)
            .map_err(|e| ClientError::DataParseError { source: e.into() })?;
        Ok(response
            .iter()
            .filter(|entry| {
                entry.id != "0"
                    && entry.name != "No results returned"
                    && entry.info_hash != "0000000000000000000000000000000000000000"
            })
            .map(|entry| Torrent {
                name: entry.name.clone(),
                magnet_link: format!("magnet:?xt=urn:btih:{}", entry.info_hash),
                seeders: entry.seeders.parse().unwrap_or(0),
                peers: entry.leechers.parse().unwrap_or(0),
                size_bytes: entry.size.parse().unwrap_or(0),
                provider: "piratebay".to_string(),
            })
            .collect())
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
