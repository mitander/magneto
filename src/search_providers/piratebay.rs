use std::error::Error;

use crate::search_providers::SearchProvider;
use crate::search_providers::Torrent;

use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::Empty;
use hyper::Request;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use serde::Deserialize;

use super::SearchRequest;

const EMPTY_ID: &str = "0";
const EMPTY_NAME: &str = "No results returned";
const EMPTY_HASH: &str = "0000000000000000000000000000000000000000";
const URL: &str = "https://apibay.org/q.php?q=";

pub struct PirateBay {}

impl PirateBay {
    pub fn new() -> PirateBay {
        PirateBay {}
    }
}

#[async_trait]
impl SearchProvider for PirateBay {
    async fn search(
        &self,
        req: SearchRequest,
    ) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>> {
        let https = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build::<_, Empty<Bytes>>(https);

        let request = Request::get(URL.to_string() + &req.query)
            .body(Empty::new())
            .expect("Request builder");

        let mut response = client.request(request).await?;
        println!("status: {}", response.status());

        let mut content = Vec::new();
        while let Some(frame) = response.body_mut().frame().await {
            let frame = frame?;
            if let Some(data) = frame.data_ref() {
                content.extend(data);
            }
        }
        let body = String::from_utf8(content)?;
        parse(&body)
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
        self.id != EMPTY_ID && self.name != EMPTY_NAME && self.info_hash != EMPTY_HASH
    }
}

fn parse(content: &str) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>> {
    let entries: Vec<Entry> = serde_json::from_str(content)?;
    println!("{:?}", entries);
    let results = entries
        .iter()
        .filter(|entry| entry.filter())
        .map(|entry| Torrent {
            name: entry.name.clone(),
            magnet_link: format!("magnet:?xt=urn:btih:{}", entry.info_hash),
            seeders: entry.seeders.parse().ok(),
            leechers: entry.leechers.parse().ok(),
            size_bytes: entry.size.parse().ok(),
        })
        .collect();
    Ok(results)
}

#[cfg(test)]
mod test {
    static TEST_DATA: &str = include_str!("../../assets/response.json");
    static TEST_DATA_EMPTY: &str = include_str!("../../assets/empty.json");

    #[test]
    fn test_parse() {
        let torrents = super::parse(TEST_DATA).unwrap();
        assert_eq!(torrents.len(), 2);
        for torrent in torrents.iter() {
            assert!(torrent.magnet_link.starts_with("magnet:?"));
            assert!(torrent.seeders.is_some());
            assert!(torrent.leechers.is_some());
            assert!(torrent.size_bytes.is_some());
        }
    }

    #[test]
    fn test_parse_empty() {
        let torrents = super::parse(TEST_DATA_EMPTY).unwrap();
        assert_eq!(torrents.len(), 0);
    }
}
