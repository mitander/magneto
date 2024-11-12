use async_trait::async_trait;
use knaben::Knaben;
use piratebay::PirateBay;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub mod knaben;
pub mod piratebay;

#[allow(dead_code)]
pub enum ProviderID {
    PirateBay,
}

impl ProviderID {
    fn provider(&self) -> Box<dyn SearchProvider> {
        match self {
            ProviderID::PirateBay => Box::new(PirateBay::new()),
        }
    }
}

#[allow(dead_code)]
pub enum Provider {
    Knaben,
    PirateBay,
    Multiple(Vec<ProviderID>),
}

pub struct Magneto {
    provider: Provider,
}

impl Magneto {
    pub fn new(provider: Provider) -> Self {
        Magneto { provider }
    }

    pub async fn search(
        &self,
        req: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>> {
        match &self.provider {
            Provider::Knaben => Knaben::new().search(req).await,
            Provider::PirateBay => PirateBay::new().search(req).await,
            Provider::Multiple(providers) => {
                let mut results = Vec::new();
                for id in providers {
                    results.extend(id.provider().search(req.clone()).await?);
                }
                Ok(results)
            }
        }
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
        req: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum OrderDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchRequest<'a> {
    search_type: SearchType,
    search_field: Option<String>,
    query: &'a str,
    order_by: OrderBy,
    order_direction: OrderDirection,
    categories: Option<Vec<u32>>,
    from: u32,
    size: u32,
    hide_unsafe: bool,
    hide_xxx: bool,
}

impl<'a> SearchRequest<'a> {
    pub fn new(query: &'a str, categories: Option<Vec<u32>>) -> Self {
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
