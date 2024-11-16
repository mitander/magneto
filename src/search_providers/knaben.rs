use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    errors::ClientError,
    http_client::{HttpClient, RequestType},
    SearchProvider, SearchRequest, Torrent,
};

#[derive(Default)]
pub struct Knaben;

impl Knaben {
    pub fn new() -> Knaben {
        Knaben {}
    }
}

#[async_trait]
impl SearchProvider for Knaben {
    async fn request_torrents(
        &self,
        client: &HttpClient,
        request: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, ClientError> {
        let knaben_request = KnabenRequest::from_search_request(request);
        let json = serde_json::to_value(&knaben_request)
            .map_err(|e| ClientError::DataParseError { source: e.into() })?;

        let raw_response = client
            .request("https://api.knaben.eu/v1", RequestType::Post(&json))
            .await?;

        self.parse_response(&raw_response)
    }

    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
        let response: Response = serde_json::from_str(response)
            .map_err(|e| ClientError::DataParseError { source: e.into() })?;

        let torrents = response
            .hits
            .iter()
            .filter(|entry| entry.hash.is_some() && entry.peers != 0)
            .map(|entry| Torrent {
                name: entry.title.to_owned(),
                magnet_link: format!("magnet:?xt=urn:btih:{}", entry.hash.to_owned().unwrap()),
                seeders: entry.seeders,
                peers: entry.peers,
                size_bytes: entry.bytes,
                provider: entry.tracker.to_owned(),
            })
            .collect();

        Ok(torrents)
    }
}

#[derive(Debug, Deserialize)]
struct Response {
    hits: Vec<Entry>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    id: String,
    title: String,
    hash: Option<String>,
    peers: u32,
    seeders: u32,
    bytes: u64,
    date: String,
    tracker: String,
    category_id: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SearchType {
    #[serde(rename = "score")]
    Score,
    Percentage(u8),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OrderBy {
    #[serde(rename = "seeders")]
    Seeders,
    #[serde(rename = "peers")]
    Peers,
    Other(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KnabenRequest {
    pub search_type: SearchType,
    pub search_field: String,
    pub query: String,
    pub order_by: OrderBy,
    pub order_direction: OrderDirection,
    pub categories: Option<Vec<String>>,
    pub from: u32,
    pub size: u32,
    pub hide_unsafe: bool,
    pub hide_xxx: bool,
    pub seconds_since_last_seen: u32,
}

impl KnabenRequest {
    pub fn from_search_request(req: SearchRequest<'_>) -> Self {
        Self {
            search_type: SearchType::Score,
            search_field: "title".to_string(),
            query: req.query.to_string(),
            order_by: OrderBy::Seeders,
            order_direction: OrderDirection::Desc,
            categories: req.categories,
            from: 0,
            size: 50,
            hide_unsafe: true,
            hide_xxx: true,
            seconds_since_last_seen: 86400, // 24hr
        }
    }
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
