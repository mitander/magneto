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

    /// Creates a new instance of the `Knaben` provider with a custom API URL.
    /// This can be useful if the provider changes url or you want to use a proxy server.
    ///
    /// # Parameters
    /// - `url`: The custom API URL to use.
    ///
    /// # Returns
    /// - `Knaben`: A new provider instance with the specified API URL.
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            api_url: url.into(),
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
            .filter_map(|entry| {
                entry.hash.as_ref().map(|hash| Torrent {
                    name: entry.title.to_owned(),
                    magnet_link: format!("magnet:?xt=urn:btih:{}", hash),
                    seeders: entry.seeders,
                    peers: entry.peers,
                    size_bytes: entry.bytes,
                    provider: format!("{} (via Knaben)", entry.tracker),
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

    /// Whether to hide unsafe or potentially malicious results.
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
    /// - `request`: The `SearchRequest` to convert.
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
            seconds_since_last_seen: 86400, // 24 hours
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

#[cfg(test)]
mod tests {

    use super::*;
    use mockito::Server;
    use serde_json::{json, Value};

    /// Sets up a mock `Knaben` provider using a mock server.
    async fn setup_mock_provider() -> Knaben {
        Knaben {
            api_url: Server::new_async().await.url(),
        }
    }

    /// Tests building a request with a valid query and categories.
    ///
    /// Ensures that the request contains the appropriate JSON body
    /// with the query and categories serialized correctly.
    #[tokio::test]
    async fn test_build_request_with_categories() {
        let provider = setup_mock_provider().await;
        let client = Client::new();

        let search_request = SearchRequest::new("ubuntu").add_category(Category::Movies);
        let request = provider.build_request(&client, search_request);

        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.method(), "POST");
        assert_eq!(
            request.headers().get(CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let body: serde_json::Value =
            serde_json::from_slice(request.body().unwrap().as_bytes().unwrap())
                .expect("Body should be valid JSON");
        assert_eq!(body["query"], "ubuntu");
        assert_eq!(body["categories"], json![[3000000]]);
    }

    /// Tests building a request with a valid query but no categories.
    ///
    /// Ensures that the request contains the query but omits the `categories` field in the body.
    #[tokio::test]
    async fn test_build_request_without_categories() {
        let provider = setup_mock_provider().await;
        let client = Client::new();

        let search_request = SearchRequest::new("ubuntu");
        let request = provider.build_request(&client, search_request);

        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.method(), "POST");

        let body: serde_json::Value =
            serde_json::from_slice(request.body().unwrap().as_bytes().unwrap())
                .expect("Body should be valid JSON");
        assert_eq!(body["query"], "ubuntu");
        assert_eq!(body.get("categories").unwrap(), &Value::Null);
    }

    /// Tests parsing a valid API response into a list of torrents.
    ///
    /// Ensures that the response is correctly parsed into a list of `Torrent` structs
    /// with all expected fields populated.
    #[tokio::test]
    async fn test_parse_response_valid() {
        let provider = setup_mock_provider().await;

        let response_body = r#"
        {
            "hits": [
                {
                    "id": "1",
                    "title": "Ubuntu ISO",
                    "hash": "abc123",
                    "peers": 10,
                    "seeders": 20,
                    "bytes": 2048,
                    "date": "2024-01-01",
                    "tracker": "knaben",
                    "categoryId": [3000000]
                }
            ]
        }
        "#;

        let result = provider.parse_response(response_body);

        assert!(result.is_ok());
        let torrents = result.unwrap();
        assert_eq!(torrents.len(), 1);

        let torrent = &torrents[0];
        assert_eq!(torrent.name, "Ubuntu ISO");
        assert_eq!(torrent.magnet_link, "magnet:?xt=urn:btih:abc123");
        assert_eq!(torrent.seeders, 20);
        assert_eq!(torrent.peers, 10);
        assert_eq!(torrent.size_bytes, 2048);
        assert_eq!(torrent.provider, "knaben (via Knaben)");
    }

    /// Tests parsing an API response with invalid JSON.
    ///
    /// Ensures that invalid JSON results in a `DataParseError`.
    #[tokio::test]
    async fn test_parse_response_invalid_json() {
        let provider = setup_mock_provider().await;

        let invalid_response_body = r#"not a valid json"#;

        let result = provider.parse_response(invalid_response_body);

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ClientError::DataParseError(_)),
            "Expected ClientError::DataParseError"
        );
    }

    /// Tests parsing an API response with invalid entries.
    ///
    /// Ensures that entries with missing or invalid fields (e.g., missing hash)
    /// are excluded from the results.
    #[tokio::test]
    async fn test_parse_response_invalid_entries() {
        let provider = setup_mock_provider().await;

        let response_body = r#"
        {
            "hits": [
                {
                    "id": "1",
                    "title": "Ubuntu ISO",
                    "hash": null,
                    "peers": 10,
                    "seeders": 20,
                    "bytes": 2048,
                    "date": "2024-01-01",
                    "tracker": "knaben",
                    "categoryId": [3000000]
                }
            ]
        }
        "#;

        let result = provider.parse_response(response_body);

        assert!(result.is_ok());
        let torrents = result.unwrap();
        assert!(
            torrents.is_empty(),
            "Expected empty results due to invalid entries"
        );
    }

    /// Tests parsing an API response with no entries.
    ///
    /// Ensures that an empty `hits` field results in an empty list of torrents.
    #[tokio::test]
    async fn test_parse_response_empty() {
        let provider = setup_mock_provider().await;

        let response_body = r#"{ "hits": [] }"#;

        let result = provider.parse_response(response_body);

        assert!(result.is_ok());
        let torrents = result.unwrap();
        assert!(torrents.is_empty(), "Expected empty results");
    }
}
