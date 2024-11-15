pub mod errors;
pub mod http_client;
pub mod search_providers;

use core::panic;
use errors::ClientError;
use search_providers::{Knaben, PirateBay, SearchProvider};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt};

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Provider {
    Knaben,
    PirateBay,
}

impl Provider {
    pub fn initialize(&self) -> Box<dyn SearchProvider> {
        match self {
            Provider::PirateBay => Box::new(PirateBay::new()),
            Provider::Knaben => Box::new(Knaben::new()),
        }
    }
}

#[derive(Default)]
pub struct Magneto {
    active_providers: HashSet<Provider>,
}

impl Magneto {
    pub fn new() -> Self {
        let all_providers: HashSet<Provider> = vec![Provider::PirateBay, Provider::Knaben]
            .into_iter()
            .collect();

        Self {
            active_providers: all_providers,
        }
    }

    pub fn with_providers(providers: Vec<Provider>) -> Self {
        Self {
            active_providers: providers.into_iter().collect(),
        }
    }
    pub fn add_provider(&mut self, provider: Provider) {
        self.active_providers.insert(provider);
    }

    pub fn remove_provider(&mut self, provider: &Provider) -> bool {
        self.active_providers.remove(provider)
    }

    pub fn active_providers(&self) -> Vec<Provider> {
        self.active_providers.iter().cloned().collect()
    }

    pub async fn search(&self, req: SearchRequest<'_>) -> Result<Vec<Torrent>, ClientError> {
        let mut results = Vec::new();

        for provider in &self.active_providers {
            match provider.initialize().execute_request(req.clone()).await {
                Ok(mut torrents) => results.append(&mut torrents),
                Err(_) => panic!(),
            }
        }

        Ok(results)
    }
}

pub struct Torrent {
    pub name: String,
    pub magnet_link: String,
    pub seeders: u32,
    pub peers: u32,
    pub size_bytes: u64,
    pub provider: String,
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
