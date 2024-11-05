use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub mod knaben;
pub mod piratebay;

pub use knaben::Knaben;
pub use piratebay::PirateBay;

#[allow(dead_code)]
#[derive(Default)]
pub struct Options {
    disable_knaben: bool,
    number_of_results: u16,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Magneto {
    providers: Vec<Box<dyn SearchProvider>>,
    options: Options,
}

impl Magneto {
    pub fn new(opts: Options) -> Self {
        Magneto {
            options: opts,
            providers: vec![Box::new(Knaben::new()), Box::new(PirateBay::new())],
        }
    }

    pub async fn search(&self, req: SearchRequest) -> Vec<Torrent> {
        let provider = if self.options.disable_knaben {
            &self.providers[1]
        } else {
            &self.providers[0]
        };

        provider.search(req).await.unwrap()
    }
}

#[allow(dead_code)]
pub struct Torrent {
    pub name: String,
    pub magnet_link: String,
    pub seeders: Option<u32>,
    pub leechers: Option<u32>,
    pub size_bytes: Option<u64>,
}

#[async_trait]
pub trait SearchProvider {
    async fn search(
        &self,
        req: SearchRequest,
    ) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>>;
}

#[derive(Serialize, Deserialize, Debug)]
enum SearchType {
    #[serde(rename = "score")]
    Score,
    Percentage(u8), // Only allow 0-100%
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
pub struct SearchRequest {
    search_type: SearchType,
    search_field: Option<String>,
    query: String,
    order_by: OrderBy,
    order_direction: OrderDirection,
    categories: Option<Vec<u32>>,
    from: u32,
    size: u32,
    hide_unsafe: bool,
    hide_xxx: bool,
}

impl SearchRequest {
    pub fn new(query: String, categories: Option<Vec<u32>>) -> Self {
        SearchRequest {
            search_type: SearchType::Score,
            search_field: None,
            query,
            order_by: OrderBy::Seeders,
            order_direction: OrderDirection::Desc,
            categories,
            from: 0,
            size: 50,
            hide_unsafe: true,
            hide_xxx: true,
        }
    }

    fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self)
    }
}
