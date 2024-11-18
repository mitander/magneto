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
//! use magneto::{Category, Magneto, SearchRequest, OrderBy};
//!
//! #[tokio::main]
//! async fn main() {
//!     let magneto = Magneto::new();
//!
//!     // You can add categories which your search are filtered by.
//!     let request = SearchRequest::new("Ubuntu")
//!         .add_category(Category::Software)
//!         .add_categories(vec![Category::Audio, Category::Movies]);
//!
//!     // Or initialize the request like this for more customization.
//!     let _request = SearchRequest {
//!         query: "Debian",
//!         query_imdb_id: false,
//!         order_by: OrderBy::Seeders,
//!         categories: vec![Category::Movies],
//!         number_of_results: 10,
//!     };
//!
//!     match magneto.search(request).await {
//!         Ok(results) => {
//!             for torrent in results {
//!                 println!(
//!                     "found: {} from {} with magnet link {} (seeders: {}, peers: {})",
//!                     torrent.name,
//!                     torrent.provider,
//!                     torrent.magnet_link,
//!                     torrent.seeders,
//!                     torrent.peers,
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
//!     let magneto = Magneto::with_providers(providers);
//!
//!     // Or add new providers like this
//!     let magneto = Magneto::default()
//!         .add_provider(Box::new(Knaben::new()))
//!         .add_provider(Box::new(PirateBay::new()));
//! }
//! ```
//!
//! ### Adding a custom provider
//!
//! ```rust
//! use magneto::{ClientError, Magneto, SearchProvider, SearchRequest, Torrent, Client, Request};
//!
//! struct CustomProvider;
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
//!     let custom_provider = CustomProvider{};
//!     let magneto = Magneto::new().add_provider(Box::new(custom_provider));
//! }
//! ```

pub mod errors;
pub mod search_providers;

use core::fmt;

use log::debug;
pub use reqwest::{Client, Request};
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

/// Enum specifying the different categories available for torrents.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub enum Category {
    /// Represents the category for movies.
    Movies,

    /// Represents the category for TV shows.
    TvShows,

    /// Represents the category for games.
    Games,

    /// Represents the category for software.
    Software,

    /// Represents the category for audio.
    Audio,

    /// Represents the category for anime.
    Anime,

    /// Represents the category for adult content.
    Xxx,
}

/// Enum specifying the order by which search results are sorted.
///
/// Implements fmt::Display
#[derive(Serialize, Debug, Clone)]
pub enum OrderBy {
    /// Sort results by the number of seeders.
    Seeders,

    /// Sort results by the number of peers.
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
/// Represents a search request to be sent to torrent providers.
#[derive(Serialize, Debug, Clone)]
pub struct SearchRequest<'a> {
    /// The query string to search for.
    pub query: &'a str,

    /// Whether to query by IMDb ID (not implemented yet).
    pub query_imdb_id: bool,

    /// The order by which results are sorted.
    pub order_by: OrderBy,

    /// Categories to filter results by. Empty means all categories are searched.
    pub categories: Vec<Category>,

    /// The number of results to retrieve.
    pub number_of_results: u32,
}

impl<'a> SearchRequest<'a> {
    /// Creates a new `SearchRequest` with the specified query.
    ///
    /// Remaining fields get the following default values:
    /// - `query_imdb_id`: `false`
    /// - `order_by`: `OrderBy::Seeders`
    /// - `categories`: An empty `Vec<Category>`
    /// - `number_of_results`: `50`
    ///
    /// # Parameters
    /// - `query`: The search term or phrase.
    ///
    /// # Returns
    /// - A new `SearchRequest` instance.
    ///
    /// # Example
    /// ```rust
    /// use magneto::SearchRequest;
    ///
    /// let request = SearchRequest::new("example query");
    /// ```
    pub fn new(query: &'a str) -> Self {
        Self {
            query,
            query_imdb_id: false,
            order_by: OrderBy::Seeders,
            categories: vec![],
            number_of_results: 50,
        }
    }

    /// Adds a single category to the `SearchRequest`.
    ///
    /// This method consumes the current instance and returns a new `SearchRequest`
    /// with the added category.
    ///
    /// # Parameters
    /// - `category`: The `Category` to add.
    ///
    /// # Returns
    /// - `Self`: A new `SearchRequest` instance with the updated category.
    ///
    /// # Example
    /// ```rust
    /// use magneto::{Category, SearchRequest};
    ///
    /// let request = SearchRequest::new("example query")
    ///     .add_category(Category::Movies)
    ///     .add_category(Category::Movies); // duplicates are filtered
    /// assert_eq!(request.categories, vec![Category::Movies]);
    /// ```
    pub fn add_category(mut self, category: Category) -> Self {
        if !self.categories.contains(&category) {
            self.categories.push(category);
        }
        self
    }

    /// Adds multiple categories to the `SearchRequest`.
    ///
    /// This method consumes the current instance and returns a new `SearchRequest`
    /// with the added categories.
    ///
    /// # Parameters
    /// - `categories`: A vector of `Category` values to add.
    ///
    /// # Returns
    /// - `Self`: A new `SearchRequest` instance with the updated categories.
    ///
    /// # Example
    /// ```rust
    /// use magneto::{Category, SearchRequest};
    ///
    /// let request = SearchRequest::new("example query").add_categories(vec![
    ///     Category::Movies,
    ///     Category::Anime,
    ///     Category::Anime, // duplicates are filtered
    /// ]);
    /// assert_eq!(request.categories, vec![Category::Movies, Category::Anime]);
    /// ```
    pub fn add_categories(mut self, categories: Vec<Category>) -> Self {
        for category in categories {
            if !self.categories.contains(&category) {
                self.categories.push(category);
            }
        }
        self
    }
}

/// The main interface for managing and querying torrent providers.
///
/// `Magneto` manages a collection of torrent search providers and allows
/// querying them simultaneously. It supports adding custom providers, querying
/// specific providers, and retrieving results in a unified format.
#[derive(Default)]
pub struct Magneto {
    pub active_providers: Vec<Box<dyn SearchProvider>>,
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
    /// - A new `Magneto` instance with unique providers.
    ///
    /// # Notes
    /// Duplicate providers are filtered based on their `id()` method to avoid duplicate searches.
    ///
    /// # Examples
    /// ```
    /// use magneto::{search_providers::{Knaben, SearchProvider}, Magneto};
    ///
    /// let providers: Vec<Box<dyn SearchProvider>> =
    ///     vec![Box::new(Knaben::new()), Box::new(Knaben::new())];
    /// let magneto = Magneto::with_providers(providers);
    ///
    /// // Duplicates are removed
    /// assert_eq!(magneto.active_providers.len(), 1);
    /// ```
    pub fn with_providers(providers: Vec<Box<dyn SearchProvider>>) -> Self {
        providers
            .into_iter()
            .fold(Self::default(), |acc, provider| acc.add_provider(provider))
    }

    /// Adds a provider to the list of active providers.
    ///
    /// This method consumes the current `Magneto` instance and returns a new instance
    /// with the added provider. If a provider with the same ID already exists, it will
    /// not be added again.
    ///
    /// # Parameters
    /// - `provider`: A provider implementing the `SearchProvider` trait.
    ///
    /// # Returns
    /// - A new `Magneto` instance with the updated list of providers.
    ///
    ///
    /// # Examples
    /// ```
    /// use magneto::{search_providers::Knaben, Magneto};
    ///
    /// let magneto = Magneto::default()
    ///     .add_provider(Box::new(Knaben::new()))
    ///     .add_provider(Box::new(Knaben::new()));
    ///
    /// // Duplicates are removed
    /// assert_eq!(magneto.active_providers.len(), 1);
    /// ```
    pub fn add_provider(mut self, provider: Box<dyn SearchProvider>) -> Self {
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
            return self;
        }

        self.active_providers.push(provider);
        self
    }

    /// Executes a search query across all active providers in sequence and aggregates the results.
    ///
    /// # Parameters
    /// - `req`: The `SearchRequest` specifying the search parameters.
    ///
    /// # Returns
    /// - `Ok(Vec<Torrent>)`: A list of torrents returned by all active providers.
    /// - `Err(ClientError)`: An error if the query fails for any provider.
    ///
    /// # Examples
    /// ```no_run
    /// use magneto::{Magneto, SearchRequest};
    ///
    /// let magneto = Magneto::new();
    /// let request = SearchRequest::new("Ubuntu");
    ///
    /// // Search default providers for "Ubuntu" and returns a vector of torrent metadata
    /// let torrents = magneto.search(request);
    /// ```
    pub async fn search(&self, req: SearchRequest<'_>) -> Result<Vec<Torrent>, ClientError> {
        let client = Client::new();
        let mut results = Vec::new();

        for provider in &self.active_providers {
            match provider.send_request(&client, req.clone()).await {
                Ok(mut torrents) => results.append(&mut torrents),
                Err(e) => return Err(e),
            }
        }

        results.sort_by(|a, b| match req.order_by {
            OrderBy::Seeders => b.seeders.cmp(&a.seeders),
            OrderBy::Peers => b.peers.cmp(&a.peers),
        });

        Ok(results)
    }
}
