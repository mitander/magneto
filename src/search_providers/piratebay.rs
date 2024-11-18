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

        let categories_string = categories.join(",");

        let mut query = vec![("q", request.query)];
        if !categories.is_empty() {
            query.push(("cat", &categories_string));
        };

        client
            .get(self.api_url.clone())
            .query(&query)
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    /// Sets up a mock PirateBay provider using a mock server.
    async fn setup_mock_provider() -> PirateBay {
        PirateBay {
            api_url: Server::new_async().await.url(),
        }
    }

    /// Tests building a request with a valid query and a category.
    ///
    /// Ensures that the request includes the `q` parameter for the query
    /// and the `cat` parameter for the category.
    #[tokio::test]
    async fn test_build_request() {
        let provider = setup_mock_provider().await;
        let client = Client::new();

        let search_request = SearchRequest::new("ubuntu").add_category(Category::Software);
        let request = provider.build_request(&client, search_request);

        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.method(), "GET");
        assert!(request.url().as_str().contains("q=ubuntu"));
        assert!(request.url().as_str().contains("cat=300"));
    }

    /// Tests building a request with a valid query but no category.
    ///
    /// Ensures that the request includes the `q` parameter for the query
    /// but does not include a `cat` parameter.
    #[tokio::test]
    async fn test_build_request_no_category() {
        let provider = setup_mock_provider().await;
        let client = Client::new();

        let search_request = SearchRequest::new("ubuntu");
        let request = provider.build_request(&client, search_request);

        assert!(request.is_ok());
        let request = request.unwrap();
        assert!(request.url().as_str().contains("q=ubuntu"));
        assert!(!request.url().as_str().contains("cat="));
    }

    /// Tests parsing a valid API response into a list of torrents.
    ///
    /// Ensures that the response is correctly parsed into a `Torrent` struct
    /// with all expected fields populated.
    #[tokio::test]
    async fn test_parse_response() {
        let provider = setup_mock_provider().await;

        let response_body = r#"
        [
            {
                "id": "1",
                "name": "Ubuntu ISO",
                "info_hash": "abc123",
                "leechers": "10",
                "seeders": "20",
                "num_files": "5",
                "size": "2048",
                "username": "user123",
                "added": "today",
                "status": "active",
                "category": "software",
                "imdb": ""
            }
        ]
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
        assert_eq!(torrent.provider, "piratebay");
    }

    /// Tests handling of invalid JSON responses from the API.
    ///
    /// Ensures that an invalid JSON string results in a `DataParseError`.
    #[tokio::test]
    async fn test_parse_response_invalid_json() {
        let provider = setup_mock_provider().await;

        let invalid_response_body = r#"not a valid json"#;

        let result = provider.parse_response(invalid_response_body);

        assert!(result.is_err());
        if let ClientError::DataParseError(e) = result.unwrap_err() {
            assert!(e.to_string().contains("expected ident at line 1 column 2"));
        } else {
            panic!("Expected ClientError::DataParseError");
        }
    }

    /// Tests handling of responses with invalid entries.
    ///
    /// Ensures that entries with invalid field values, such as unparsable
    /// leechers, are excluded from the results.
    #[tokio::test]
    async fn test_parse_response_invalid_entry() {
        let provider = setup_mock_provider().await;

        let response_body = r#"
        [
            {
                "id": "1",
                "name": "Invalid Torrent",
                "info_hash": "abc123",
                "leechers": "invalid",
                "seeders": "20",
                "num_files": "5",
                "size": "2048",
                "username": "user123",
                "added": "today",
                "status": "active",
                "category": "software",
                "imdb": ""
            }
        ]
        "#;

        let result = provider.parse_response(response_body);

        assert!(result.is_ok());
        let torrents = result.unwrap();
        assert!(
            torrents.is_empty(),
            "Expected empty results due to invalid leechers"
        );
    }

    /// Tests handling of an empty API response.
    ///
    /// Ensures that entries with default or invalid field values are excluded
    /// from the results.
    #[tokio::test]
    async fn test_parse_empty_response() {
        let provider = setup_mock_provider().await;

        let response_body = r#"
        [
            {
                "id": "0",
                "name": "No results returned",
                "info_hash": "0000000000000000000000000000000000000000",
                "leechers": "0",
                "seeders": "0",
                "num_files": "0",
                "size": "0",
                "username": "",
                "added": "",
                "status": "",
                "category": "",
                "imdb": ""
            }
        ]
        "#;

        let result = provider.parse_response(response_body);

        assert!(result.is_ok());
        let torrents = result.unwrap();
        assert!(
            torrents.is_empty(),
            "Expected empty results due to invalid entry"
        );
    }
}
