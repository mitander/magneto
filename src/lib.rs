pub mod errors;
pub mod search_providers;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use errors::ClientError;
use search_providers::{Knaben, PirateBay, SearchProvider};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Torrent {
    pub name: String,
    pub magnet_link: String,
    pub seeders: u32,
    pub peers: u32,
    pub size_bytes: u64,
    pub provider: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum OrderBy {
    Seeders,
    Peers,
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
        Self {
            query,
            query_imdb_id: false,
            order_by: OrderBy::Seeders,
            categories,
            number_of_results: 50,
            hide_xxx: true,
        }
    }
}

#[derive(Default)]
pub struct Magneto {
    active_providers: Vec<Box<dyn SearchProvider>>,
}

impl Magneto {
    pub fn new() -> Self {
        let providers: Vec<Box<dyn SearchProvider>> =
            vec![Box::new(Knaben::new()), Box::new(PirateBay::new())];

        Self {
            active_providers: providers,
        }
    }

    pub fn with_providers(providers: Vec<Box<dyn SearchProvider>>) -> Self {
        Self {
            active_providers: providers,
        }
    }

    pub fn add_provider(&mut self, provider: Box<dyn SearchProvider>) {
        let provider_id = provider.id();

        if self
            .active_providers
            .iter()
            .any(|existing| existing.id() == provider_id)
        {
            println!(
                "Provider '{}' already exists. Skipping addition.",
                provider_id
            );
            return;
        }

        self.active_providers.push(provider);
    }

    pub async fn search(&self, req: SearchRequest<'_>) -> Result<Vec<Torrent>, ClientError> {
        let client = Client::new();
        let mut results = Vec::new();

        for provider in &self.active_providers {
            match provider.send_request(&client, req.clone()).await {
                Ok(mut torrents) => results.append(&mut torrents),
                Err(_) => panic!(),
            }
        }

        Ok(results)
    }
}
