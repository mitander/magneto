use crate::{http_client, SearchProvider, SearchRequest, Torrent};

use async_trait::async_trait;
use serde::Deserialize;
use std::error::Error;

pub struct Knaben {}

impl Default for Knaben {
    fn default() -> Self {
        Self::new()
    }
}

impl Knaben {
    pub fn new() -> Knaben {
        Knaben {}
    }
}

#[async_trait]
impl SearchProvider for Knaben {
    async fn search(
        &self,
        req: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>> {
        let client = http_client::HttpClient::new();
        let url = "http://api.knaben.eu/v1".parse().unwrap();
        let body = client.send_post_request(url, req.query).await.unwrap();
        let response: Response = serde_json::from_str(&body)?;
        handle_response(response.hits)
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Response {
    total: Total,
    hits: Vec<Entry>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Total {
    relation: String,
    value: u16,
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
    fn filter(&self) -> bool {
        self.hash.is_some()
    }
}

fn handle_response(response: Vec<Entry>) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>> {
    Ok(response
        .iter()
        .filter(|entry| entry.filter())
        .map(|entry| Torrent {
            name: entry.title.clone(),
            magnet_link: format!("magnet:?xt=urn:btih:{}", entry.hash.to_owned().unwrap()),
            seeders: entry.seeders,
            leechers: entry.leechers,
            size_bytes: entry.bytes,
        })
        .collect())
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
