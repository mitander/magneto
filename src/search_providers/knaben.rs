use crate::{
    errors,
    http_client::{self, Client, RequestMethod},
    SearchProvider, SearchRequest, Torrent,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct Knaben {}

impl Knaben {
    pub fn new() -> Knaben {
        Knaben {}
    }
}

#[async_trait]
impl SearchProvider for Knaben {
    async fn execute_request(
        &self,
        req: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, errors::ClientError> {
        let client = Client::default();
        let knaben_req = KnabenRequest::from_search_request(req);

        let body = http_client::build_body(&knaben_req)?;
        let req = client.build_request("https://api.knaben.eu/v1", RequestMethod::POST(body))?;

        let res = client.send_request(req).await?;
        let res_data: Response = serde_json::from_slice(&res).unwrap();
        handle_response(res_data.hits)
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

fn handle_response(response: Vec<Entry>) -> Result<Vec<Torrent>, errors::ClientError> {
    Ok(response
        .iter()
        .filter(|entry| entry.hash.is_some() && entry.peers != 0)
        .map(|entry| Torrent {
            name: entry.title.clone(),
            magnet_link: format!("magnet:?xt=urn:btih:{}", entry.hash.to_owned().unwrap()),
            seeders: entry.seeders,
            peers: entry.peers,
            size_bytes: entry.bytes,
            provider: entry.tracker.clone(),
        })
        .collect())
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
