use async_trait::async_trait;
use serde::Deserialize;

use crate::{errors::ClientError, SearchProvider, SearchRequest, Torrent};

use reqwest::{Client, Request};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ResponseEntry {
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

pub struct PirateBay {
    api_url: String,
}

impl PirateBay {
    pub fn new() -> Self {
        Self {
            api_url: "https://apibay.org/q.php?".to_string(),
        }
    }
}

impl Default for PirateBay {
    fn default() -> Self {
        PirateBay::new()
    }
}

#[async_trait]
impl SearchProvider for PirateBay {
    fn build_request(
        &self,
        client: &Client,
        request: SearchRequest<'_>,
    ) -> Result<Request, ClientError> {
        let query = &[("q", request.query)];

        client
            .get(self.api_url.clone())
            .query(query)
            .build()
            .map_err(|e| ClientError::RequestBuildError {
                source: e.into(),
                url: self.api_url.clone(),
            })
    }

    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
        let response: Vec<ResponseEntry> = serde_json::from_str(response)
            .map_err(|e| ClientError::DataParseError { source: e.into() })?;

        let torrents = response
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
            .collect();

        Ok(torrents)
    }
    fn id(&self) -> String {
        self.api_url.clone()
    }
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
