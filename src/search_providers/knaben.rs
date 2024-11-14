use crate::{
    errors,
    http_client::{self, Client, RequestMethod},
    SearchProvider, SearchRequest, Torrent,
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
enum SearchType {
    #[serde(rename = "score")]
    Score,
    Percentage(u8),
}
impl fmt::Display for SearchType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SearchType::Score => write!(f, "score"),
            SearchType::Percentage(val) => write!(f, "{}%", val),
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
enum OrderBy {
    #[serde(rename = "seeders")]
    Seeders,
    #[serde(rename = "peers")]
    Peers,
    Other(String),
}
impl fmt::Display for OrderBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderBy::Seeders => write!(f, "seeders"),
            OrderBy::Peers => write!(f, "peers"),
            OrderBy::Other(other) => write!(f, "{}", other),
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
enum OrderDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KnabenRequest {
    search_type: SearchType,
    search_field: Option<String>,
    query: String,
    order_by: OrderBy,
    order_direction: OrderDirection,
    categories: Option<Vec<String>>,
    from: u32,
    size: u32,
    hide_unsafe: bool,
    hide_xxx: bool,
}

impl KnabenRequest {
    pub fn new(req: SearchRequest<'_>) -> Self {
        Self {
            search_type: SearchType::Score,
            search_field: Some("title".to_string()),
            query: req.query.to_string(),
            order_by: OrderBy::Seeders,
            order_direction: OrderDirection::Desc,
            categories: req.categories,
            from: 0,
            size: 50,
            hide_unsafe: true,
            hide_xxx: true,
        }
    }
}

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
    async fn search(&self, req: SearchRequest<'_>) -> Result<Vec<Torrent>, errors::ClientError> {
        let client = Client::default();
        let knaben_req = KnabenRequest::new(req);

        let body = http_client::build_body(&knaben_req)?;
        let req = client
            .build_request("https://api.knaben.eu/v1", RequestMethod::POST(body))
            .unwrap();

        let res = client.send_request(req).await?;
        let knaben_res: Response = serde_json::from_slice(&res).unwrap();
        handle_response(knaben_res.hits)
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

fn handle_response(response: Vec<Entry>) -> Result<Vec<Torrent>, errors::ClientError> {
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
