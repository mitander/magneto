use std::error::Error;

use crate::search_providers::SearchProvider;
use crate::search_providers::Torrent;

use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::{Method, Request};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use serde::Deserialize;

use super::SearchRequest;

const URL: &str = "https://api.knaben.eu/v1";

pub struct Knaben {}

impl Knaben {
    pub fn new() -> Knaben {
        Knaben {}
    }
}

#[async_trait]
impl SearchProvider for Knaben {
    async fn search(
        &self,
        req: SearchRequest,
    ) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>> {
        let https = HttpsConnector::new();
        let client = Client::builder(TokioExecutor::new()).build::<_, Full<Bytes>>(https);

        let json = req.to_json()?;
        println!("{}", json);

        let request = Request::builder()
            .method(Method::POST)
            .uri(URL)
            .header("Content-Type", "application/json")
            .body(Full::from(json))?;

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
pub struct Total {
    relation: String,
    value: u16,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct EntryWrap {
    hits: Vec<Entry>,
    total: Total,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub hash: Option<String>,
    pub leechers: Option<u32>,
    pub seeders: Option<u32>,
    pub bytes: Option<u64>,
    pub date: String,
    pub tracker: String,
    pub category_id: Vec<u32>,
}

impl Entry {
    fn filter(self: &Entry) -> bool {
        self.hash.is_some()
    }
}

fn parse(content: &str) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>> {
    let entry_wrap: EntryWrap = serde_json::from_str(content)?;
    let entries = entry_wrap.hits;
    println!("{:?}", entries);
    let results = entries
        .iter()
        .filter(|entry| entry.filter())
        .map(|entry| {
            let magnet_link = format!("magnet:?xt=urn:btih:{}", entry.hash.to_owned().unwrap());
            Torrent {
                name: entry.title.clone(),
                magnet_link,
                seeders: entry.seeders,
                leechers: entry.leechers,
                size_bytes: entry.bytes,
            }
        })
        .collect();
    Ok(results)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse() {
        todo!()
    }

    #[test]
    fn test_parse_empty() {
        todo!()
    }
}
