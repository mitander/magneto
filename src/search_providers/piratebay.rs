//! # PirateBay (apibay.org) Search Provider
//!
//! The `PirateBay` implementation of the `SearchProvider` trait allows querying
//! PirateBay API for torrent metadata. This provider formats queries,
//! sends them to PirateBay API, and parses the resulting JSON response into
//! a unified `Torrent` structure.

use async_trait::async_trait;
use reqwest::{Client, Request};
use serde::Deserialize;

use crate::{errors::ClientError, Category, SearchProvider, SearchRequest, Torrent};

/// The `PirateBay` provider handles querying and parsing data from the PirateBay API.
pub struct PirateBay {
    /// The base URL for the PirateBay API.
    api_url: String,
}

impl PirateBay {
    /// Creates a new instance of the `PirateBay` provider.
    ///
    /// # Returns
    /// - `PirateBay`: A new provider instance with the default API URL.
    pub fn new() -> Self {
        Self {
            api_url: "https://apibay.org/q.php".to_string(),
        }
    }
}

impl Default for PirateBay {
    /// Provides a default implementation for `PirateBay`, returning an instance with the default API URL.
    fn default() -> Self {
        PirateBay::new()
    }
}

#[async_trait]
impl SearchProvider for PirateBay {
    /// Builds the request to query the PirateBay API.
    ///
    /// # Parameters
    /// - `client`: The HTTP client used to build the request.
    /// - `request`: The `SearchRequest` containing query parameters.
    ///
    /// # Returns
    /// - `Ok(Request)`: The constructed HTTP request.
    /// - `Err(ClientError)`: An error if request building fails.
    fn build_request(
        &self,
        client: &Client,
        request: SearchRequest<'_>,
    ) -> Result<Request, ClientError> {
        let categories: Vec<String> = request
            .categories
            .into_iter()
            .map(|category| match category {
                Category::Movies => "201,202,207,209,211".to_string(),
                Category::TvShows => "205,208,212".to_string(),
                Category::Games => 400.to_string(),
                Category::Software => 300.to_string(),
                Category::Audio => 100.to_string(),
                Category::Anime => 200.to_string(), // not supported: list all video results
                Category::Xxx => 500.to_string(),
            })
            .collect();

        let query = &[("q", request.query), ("cat", &categories.join(","))];

        client
            .get(self.api_url.clone())
            .query(query)
            .build()
            .map_err(|e| ClientError::RequestBuildError {
                source: e.into(),
                url: self.api_url.clone(),
            })
    }

    /// Parses the response from the PirateBay API into a list of torrents.
    ///
    /// # Parameters
    /// - `response`: The raw response body as a string.
    ///
    /// # Returns
    /// - `Ok(Vec<Torrent>)`: A list of parsed torrent metadata.
    /// - `Err(ClientError)`: An error if parsing fails.
    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
        let response: Vec<ResponseEntry> =
            serde_json::from_str(response).map_err(|e| ClientError::DataParseError(e.into()))?;

        let torrents = response
            .iter()
            .filter(|entry| {
                entry.id != "0"
                    && entry.name != "No results returned"
                    && entry.info_hash != "0000000000000000000000000000000000000000"
            })
            .filter_map(|entry| {
                let seeders = entry.seeders.parse().ok()?;
                let peers = entry.leechers.parse().ok()?;
                let size_bytes = entry.size.parse().ok()?;

                Some(Torrent {
                    name: entry.name.clone(),
                    magnet_link: format!("magnet:?xt=urn:btih:{}", entry.info_hash),
                    seeders,
                    peers,
                    size_bytes,
                    provider: "piratebay".to_string(),
                })
            })
            .collect();

        Ok(torrents)
    }

    /// Returns the unique identifier for this provider.
    ///
    /// # Returns
    /// - `String`: The provider's API URL as its unique identifier.
    fn id(&self) -> String {
        self.api_url.clone()
    }
}

/// Represents a single entry in the PirateBay API response.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ResponseEntry {
    /// The unique identifier for the torrent.
    pub id: String,

    /// The name or title of the torrent.
    pub name: String,

    /// The hash of the torrent, used in magnet links.
    pub info_hash: String,

    /// The number of leechers for the torrent as a string.
    pub leechers: String,

    /// The number of seeders for the torrent as a string.
    pub seeders: String,

    /// The number of files included in the torrent.
    pub num_files: String,

    /// The size of the torrent in bytes, represented as a string.
    pub size: String,

    /// The username of the uploader who shared the torrent.
    pub username: String,

    /// The date when the torrent was added.
    pub added: String,

    /// The status of the torrent (e.g., active, inactive).
    pub status: String,

    /// The category of the torrent (e.g., movies, games, software).
    pub category: String,

    /// The IMDb ID associated with the torrent, if available.
    pub imdb: String,
}
