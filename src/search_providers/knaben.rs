//! # Knaben Search Provider
//!
//! The `Knaben` implementation of the `SearchProvider` trait allows querying
//! the Knaben API for torrent metadata. This provider constructs requests with
//! JSON bodies and parses the response into the `Torrent` structure.

use async_trait::async_trait;
use reqwest::{header::CONTENT_TYPE, Client, Request};
use serde::{Deserialize, Serialize};

use crate::{Category, ClientError, SearchProvider, SearchRequest, Torrent};

/// The `Knaben` provider handles querying and parsing data from the Knaben API.
pub struct Knaben {
    /// The base URL for the Knaben API.
    api_url: String,
}

impl Knaben {
    /// Creates a new instance of the `Knaben` provider.
    ///
    /// # Returns
    /// - `Knaben`: A new provider instance with the default API URL.
    pub fn new() -> Self {
        Self {
            api_url: "https://api.knaben.eu/v1".to_string(),
        }
    }
}

impl Default for Knaben {
    /// Provides a default implementation for `Knaben`, returning an instance with the default API URL.
    fn default() -> Self {
        Knaben::new()
    }
}

#[async_trait]
impl SearchProvider for Knaben {
    /// Builds the request to query the Knaben API.
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
        let knaben_request = KnabenRequest::from_search_request(request);
        let json = serde_json::to_value(&knaben_request)
            .map_err(|e| ClientError::DataParseError(e.into()))?;

        client
            .post(self.api_url.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(json.to_string())
            .build()
            .map_err(|e| ClientError::RequestBuildError {
                source: e.into(),
                url: self.api_url.clone(),
            })
    }

    /// Parses the response from the Knaben API into a list of torrents.
    ///
    /// # Parameters
    /// - `response`: The raw response body as a string.
    ///
    /// # Returns
    /// - `Ok(Vec<Torrent>)`: A list of parsed torrent metadata.
    /// - `Err(ClientError)`: An error if parsing fails.
    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
        let response: Response =
            serde_json::from_str(response).map_err(|e| ClientError::DataParseError(e.into()))?;

        let torrents = response
            .entries
            .iter()
            .filter(|entry| entry.hash.is_some() && entry.peers != 0)
            .map(|entry| Torrent {
                name: entry.title.to_owned(),
                magnet_link: format!("magnet:?xt=urn:btih:{}", entry.hash.to_owned().unwrap()),
                seeders: entry.seeders,
                peers: entry.peers,
                size_bytes: entry.bytes,
                provider: format!("{} (via knaben)", entry.tracker),
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

/// Represents the structure of a Knaben API request.
#[derive(Serialize, Deserialize, Debug)]
struct KnabenRequest {
    /// The type of search to perform.
    search_type: String,

    /// The field to search within.
    search_field: String,

    /// The query string to search for.
    query: String,

    /// The field by which results are ordered.
    order_by: String,

    /// The direction of ordering (asc/desc).
    order_direction: String,

    /// Optional categories to filter results by.
    categories: Option<Vec<u32>>,

    /// The number of results to retrieve.
    size: u32,

    /// Whether to hide very old results and results that has a high potential virus score.
    hide_unsafe: bool,

    /// Whether to hide adult content.
    hide_xxx: bool,

    /// Time (in seconds) since the last seen torrent to filter results.
    seconds_since_last_seen: u32,
}
impl KnabenRequest {
    /// Converts a `SearchRequest` into a `KnabenRequest`.
    ///
    /// # Parameters
    /// - `req`: The `SearchRequest` to convert.
    ///
    /// # Returns
    /// - `KnabenRequest`: A request formatted for the Knaben API.
    pub fn from_search_request(request: SearchRequest<'_>) -> Self {
        let mut hide_xxx = true;
        let categories: Option<Vec<u32>> = if request.categories.is_empty() {
            None
        } else {
            Some(
                request
                    .categories
                    .iter()
                    .map(|category| match category {
                        Category::Movies => 3000000,
                        Category::TvShows => 2000000,
                        Category::Games => 4001000,
                        Category::Software => 4002000,
                        Category::Audio => 1000000,
                        Category::Anime => 6000000,
                        Category::Xxx => {
                            hide_xxx = false;
                            5000000
                        }
                    })
                    .collect(),
            )
        };

        Self {
            search_type: "score".to_string(),
            search_field: "title".to_string(),
            query: request.query.to_string(),
            order_by: request.order_by.to_string(),
            order_direction: "desc".to_string(),
            categories,
            size: 50,
            hide_unsafe: true,
            hide_xxx,
            seconds_since_last_seen: 86400, // 24hr
        }
    }
}

/// Represents the structure of the response returned by the Knaben API.
#[derive(Debug, Deserialize)]
struct Response {
    /// A list of search results returned by the API.
    #[serde(rename = "hits")]
    entries: Vec<ResponseEntry>,
}

/// Represents a single entry in the Knaben API response.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseEntry {
    /// Unique identifier for the torrent.
    id: String,

    /// Title of the torrent.
    title: String,

    /// Hash of the torrent, used in the magnet link.
    hash: Option<String>,

    /// Number of peers sharing the torrent.
    peers: u32,

    /// Number of seeders sharing the torrent.
    seeders: u32,

    /// Size of the torrent in bytes.
    bytes: u64,

    /// Date when the torrent was added.
    date: String,

    /// The tracker where the torrent is hosted.
    tracker: String,

    /// Categories associated with the torrent.
    category_id: Vec<u32>,
}
