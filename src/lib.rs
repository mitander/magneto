pub mod errors;
pub mod http_client;
pub mod search_providers;

use errors::ClientError;
use search_providers::{Knaben, PirateBay, SearchProvider};
use serde::{Deserialize, Serialize};
use std::fmt;

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

    pub async fn search(&self, req: SearchRequest<'_>) -> Result<Vec<Torrent>, ClientError> {
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

pub struct Torrent {
    pub name: String,
    pub magnet_link: String,
    pub seeders: Option<u32>,
    pub leechers: Option<u32>,
    pub size_bytes: Option<u64>,
}

#[derive(Serialize, Debug, Clone)]
pub struct SearchRequest<'a> {
    query: &'a str,
    query_imdb_id: bool,
    order_by: OrderBy,
    categories: Option<Vec<String>>,
    number_of_results: u32,
    hide_xxx: bool,
}

impl<'a> SearchRequest<'a> {
    pub fn new(query: &'a str, categories: Option<Vec<String>>) -> Self {
        SearchRequest {
            query,
            query_imdb_id: false,
            order_by: OrderBy::Seeders,
            categories,
            number_of_results: 50,
            hide_xxx: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum OrderBy {
    #[serde(rename = "seeders")]
    Seeders,
    #[serde(rename = "peers")]
    Peers,
}

impl fmt::Display for OrderBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderBy::Seeders => write!(f, "seeders"),
            OrderBy::Peers => write!(f, "peers"),
        }
    }
}
