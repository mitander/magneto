use crate::{
    errors::ClientError,
    http_client::{Client, RequestMethod},
    SearchProvider, SearchRequest, Torrent,
};
use serde::Deserialize;

use async_trait::async_trait;

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
    async fn search(&self, req: SearchRequest<'_>) -> Result<Vec<Torrent>, ClientError> {
        let client = Client::default();

        let query = req.query.to_string();
        let req = client
            .build_request("https://apibay.org/q.php?=q", RequestMethod::GET(query))
            .unwrap();

        let res = client.send_request(req).await?;
        handle_response(serde_json::from_slice(&res).unwrap())
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

fn handle_response(response: Vec<Entry>) -> Result<Vec<Torrent>, ClientError> {
    Ok(response
        .iter()
        .filter(|entry| {
            println!("entry: {:?}", entry);
            entry.filter()
        })
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
