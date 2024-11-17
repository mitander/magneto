//! # Magneto
//!
//! `Magneto` is a library for searching torrents across multiple providers.
//! It provides a unified interface for querying torrent metadata and integrating
//! custom providers.
//!
//! ## Features
//! - Query multiple torrent search providers simultaneously.
//! - Add custom providers with minimal effort.
//! - Retrieve results in a unified format with metadata like magnet link, seeders, peers, and size.
//!
//! ## Supported providers
//! - PirateBay (apibay.org)
//! - Knaben (knaben.eu)
//!
//! ## Examples
//!
//! ### Creating a `Magneto` instance and searching
//!
//! ```rust
//! use magneto::{Magneto, SearchRequest};
//!
//! #[tokio::main]
//! async fn main() {
//!     let magneto = Magneto::new();
//!
//!     let request = SearchRequest::new("Ubuntu", None);
//!     match magneto.search(request).await {
//!         Ok(results) => {
//!             for torrent in results {
//!                 println!(
//!                     "found: {} (seeders: {}, peers: {})",
//!                     torrent.name, torrent.seeders, torrent.peers
//!                 );
//!             }
//!         }
//!         Err(e) => eprintln!("error during search: {:?}", e),
//!     }
//! }
//! ```
//!
//! ### Creating a `Magneto` instance from list of providers
//!
//! ```rust
//! use magneto::{
//!     search_providers::{Knaben, PirateBay, SearchProvider},
//!     Magneto,
//! };
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create instance from list of providers
//!     let providers: Vec<Box<dyn SearchProvider>> =
//!         vec![Box::new(Knaben::new()), Box::new(PirateBay::new())];
//!     let _magneto = Magneto::with_providers(providers);
//!
//!     // Or use add_provider() to add to list of active providers
//!     let mut magneto = Magneto::default(); // no providers
//!     magneto.add_provider(Box::new(Knaben::new()));
//!     magneto.add_provider(Box::new(PirateBay::new()));
//! }
//! ```
//!
//! ### Adding a custom provider
//!
//! ```rust
//! use magneto::{errors::ClientError, Magneto, SearchProvider, SearchRequest, Torrent};
//! use reqwest::{Client, Request};
//!
//! struct CustomProvider;
//!
//! impl CustomProvider {
//!     pub fn new() -> Self {
//!         Self {}
//!     }
//! }
//!
//! impl SearchProvider for CustomProvider {
//!     fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
//!         todo!("parse response data into Vec<Torrent>");
//!     }
//!
//!     fn build_request(
//!         &self,
//!         client: &Client,
//!         request: SearchRequest<'_>,
//!     ) -> Result<Request, ClientError> {
//!         todo!("convert SearchRequest to reqwest::Request");
//!     }
//!
//!     fn id(&self) -> String {
//!         "custom_provider".to_string()
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let custom_provider = CustomProvider::new();
//!     let mut magneto = Magneto::new();
//!     magneto.add_provider(Box::new(custom_provider));
//! }
//! ```

pub mod errors;
pub mod search_providers;

use log::debug;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub use errors::ClientError;
pub use search_providers::{Knaben, PirateBay, SearchProvider};

/// Represents metadata for a torrent returned by a search provider.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Torrent {
    /// The name of the torrent.
    pub name: String,

    /// The magnet link for downloading the torrent.
    pub magnet_link: String,

    /// The number of seeders available.
    pub seeders: u32,

    /// The number of peers available.
    pub peers: u32,

    /// The size of the torrent in bytes.
    pub size_bytes: u64,

    /// The identifier of the provider that returned this torrent.
    pub provider: String,
}

/// Enum specifying the order by which search results are sorted.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum OrderBy {
    /// Sort results by the number of seeders.
    Seeders,

    /// Sort results by the number of peers.
    Peers,
}

/// Represents a search request to be sent to torrent providers.
#[derive(Serialize, Debug, Clone)]
pub struct SearchRequest<'a> {
    /// The query string to search for.
    query: &'a str,

    /// Whether to query by IMDb ID (not implemented yet).
    query_imdb_id: bool,

    /// The order by which results are sorted (default: `Seeders`).
    order_by: OrderBy,

    /// Optional categories to filter results by.
    categories: Option<Vec<String>>,

    /// The number of results to retrieve (default: 50).
    number_of_results: u32,

    /// Whether to hide adult content (default: true).
    hide_xxx: bool,
}

impl<'a> SearchRequest<'a> {
    /// Creates a new `SearchRequest` with the specified query and optional categories.
    ///
    /// # Parameters
    /// - `query`: The search term or phrase.
    /// - `categories`: An optional list of categories to filter results.
    ///
    /// # Returns
    /// - A new `SearchRequest` instance.
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

/// The main interface for managing and querying torrent providers.
///
/// `Magneto` manages a collection of torrent search providers and allows
/// querying them simultaneously. It supports adding custom providers, querying
/// specific providers, and retrieving results in a unified format.
#[derive(Default)]
pub struct Magneto {
    active_providers: Vec<Box<dyn SearchProvider>>,
}

impl Magneto {
    /// Creates a new `Magneto` instance with default providers.
    ///
    /// The default providers include:
    /// - `Knaben`
    /// - `PirateBay`
    ///
    /// # Returns
    /// - A new `Magneto` instance with default providers.
    pub fn new() -> Self {
        let providers: Vec<Box<dyn SearchProvider>> =
            vec![Box::new(Knaben::new()), Box::new(PirateBay::new())];

        Self {
            active_providers: providers,
        }
    }

    /// Creates a new `Magneto` instance with the specified providers.
    ///
    /// # Parameters
    /// - `providers`: A vector of custom providers implementing the `SearchProvider` trait.
    ///
    /// # Returns
    /// - A new `Magneto` instance with the specified providers.
    pub fn with_providers(providers: Vec<Box<dyn SearchProvider>>) -> Self {
        Self {
            active_providers: providers,
        }
    }

    /// Adds a provider to the list of active providers.
    ///
    /// # Parameters
    /// - `provider`: A provider implementing the `SearchProvider` trait.
    ///
    /// # Notes
    /// If a provider with the same ID already exists, it will not be added again.
    pub fn add_provider(&mut self, provider: Box<dyn SearchProvider>) {
        let provider_id = provider.id();

        if self
            .active_providers
            .iter()
            .any(|existing| existing.id() == provider_id)
        {
            debug!(
                "provider '{}' already exists, skipping addition",
                provider_id
            );
            return;
        }

        self.active_providers.push(provider);
    }

    /// Executes a search query across all active providers.
    ///
    /// # Parameters
    /// - `req`: The `SearchRequest` specifying the search parameters.
    ///
    /// # Returns
    /// - `Ok(Vec<Torrent>)`: A list of torrents returned by all active providers.
    /// - `Err(ClientError)`: An error if the query fails for any provider.
    ///
    /// # Notes
    /// This method queries each provider in sequence and aggregates the results.
    pub async fn search(&self, req: SearchRequest<'_>) -> Result<Vec<Torrent>, ClientError> {
        let client = Client::new();
        let mut results = Vec::new();

        for provider in &self.active_providers {
            match provider.send_request(&client, req.clone()).await {
                Ok(mut torrents) => results.append(&mut torrents),
                Err(e) => return Err(e),
            }
        }

        Ok(results)
    }
}
