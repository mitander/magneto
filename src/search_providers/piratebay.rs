use crate::{
    errors::ClientError,
    http_client::{Client, RequestMethod},
    SearchProvider, SearchRequest, Torrent,
};

use async_trait::async_trait;
use serde::Deserialize;

#[derive(Default)]
pub struct PirateBay {}

impl PirateBay {
    pub fn new() -> PirateBay {
        PirateBay {}
    }
}

#[async_trait]
impl SearchProvider for PirateBay {
    async fn execute_request(&self, req: SearchRequest<'_>) -> Result<Vec<Torrent>, ClientError> {
        let client = Client::default();

        // Url.parse() removes the '=q' suffix when parsed, add it to the query
        let query = "q=".to_string() + req.query;
        let req = client.build_request("https://apibay.org/q.php", RequestMethod::GET(query))?;

        let res = client.send_request(req).await?;
        let res_data = serde_json::from_slice(&res).unwrap();
        parse_response(res_data)
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

fn parse_response(response: Vec<Entry>) -> Result<Vec<Torrent>, ClientError> {
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
