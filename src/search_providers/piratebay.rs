use crate::{SearchProvider, SearchRequest, Torrent};

use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::Request;
use hyper_tls::HttpsConnector;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use serde::Deserialize;
use std::error::Error;

const URL: &str = "https://apibay.org/q.php?q=";

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
        let https = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build::<_, Empty<Bytes>>(https);

        let request = Request::get(URL.to_string() + req.query)
            .body(Empty::new())
            .expect("Request builder");

        let mut response = client.request(request).await?;
        println!("status: {}", response.status());

        let mut content = Vec::new();
        while let Some(Ok(frame)) = response.body_mut().frame().await {
            if let Some(data) = frame.data_ref() {
                content.extend(data);
            }
        }
        let body = String::from_utf8(content)?;
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
