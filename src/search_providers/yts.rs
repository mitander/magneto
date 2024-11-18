//! # YTS (yts.mx) Search Provider
//!
//! The `Yts` implementation of the `SearchProvider` trait allows querying
//! YTS API for torrent metadata. This provider formats queries,
//! sends them to YTS API, and parses the resulting JSON response into
//! a unified `Torrent` structure.

use async_trait::async_trait;
use reqwest::{Client, Request};
use serde::Deserialize;

use crate::{errors::ClientError, Category, SearchProvider, SearchRequest, Torrent};

/// The `Yts` provider handles querying and parsing data from the YTS API.
pub struct Yts {
    /// The base URL for the YTS API.
    api_url: String,
}

impl Yts {
    /// Creates a new instance of the `Yts` provider.
    ///
    /// # Returns
    /// - `Yts`: A new provider instance with the default API URL.
    pub fn new() -> Self {
        Self {
            api_url: "https://yts.mx/api/v2/list_movies.json".to_string(),
        }
    }

    /// Creates a new instance of the `Yts` provider with a custom API URL.
    ///
    /// # Parameters
    /// - `url`: The custom API URL to use.
    ///
    /// # Returns
    /// - `Yts`: A new provider instance with the specified API URL.
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            api_url: url.into(),
        }
    }
}

impl Default for Yts {
    /// Provides a default implementation for `Yts`, returning an instance with the default API URL.
    fn default() -> Self {
        Yts::new()
    }
}

#[async_trait]
impl SearchProvider for Yts {
    /// Builds the request to query the YTS API.
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
        let mut query = vec![("query_term", request.query)];
        if let Some(category) = request.categories.first() {
            let genre = match category {
                Category::Movies => "movie",
                Category::Anime => "anime",
                _ => "", // YTS focuses on movies, so unsupported categories are ignored
            };
            if !genre.is_empty() {
                query.push(("genre", genre));
            }
        }

        client
            .get(self.api_url.clone())
            .query(&query)
            .build()
            .map_err(|e| ClientError::RequestBuildError {
                source: e.into(),
                url: self.api_url.clone(),
            })
    }

    /// Parses the response from the YTS API into a list of torrents.
    ///
    /// # Parameters
    /// - `response`: The raw response body as a string.
    ///
    /// # Returns
    /// - `Ok(Vec<Torrent>)`: A list of parsed torrent metadata, or an empty list if no movies are found.
    /// - `Err(ClientError)`: An error if parsing fails.
    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
        let response: YtsResponse =
            serde_json::from_str(response).map_err(|e| ClientError::DataParseError(e.into()))?;

        // Check if the movies field is present; if not, return an empty vector
        let movies = response.data.movies.unwrap_or_default();

        let torrents = movies
            .into_iter()
            .flat_map(|movie| {
                movie.torrents.into_iter().map(move |torrent| Torrent {
                    name: movie.title.clone(),
                    magnet_link: format!("magnet:?xt=urn:btih:{}", torrent.hash),
                    seeders: torrent.seeds,
                    peers: torrent.peers,
                    size_bytes: torrent.size_bytes(),
                    provider: "yts".to_string(),
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

/// Represents the top-level YTS API response.
#[derive(Debug, Deserialize)]
struct YtsResponse {
    /// The `data` field containing the movie list.
    data: YtsData,
}

/// Contains movie data in the YTS API response.
#[derive(Debug, Deserialize)]
struct YtsData {
    /// A list of movies in the response, optional in case no movies are found.
    #[serde(default)]
    movies: Option<Vec<YtsMovie>>,
}

/// Represents a single movie in the YTS API response.
#[derive(Debug, Deserialize)]
struct YtsMovie {
    /// The title of the movie.
    title: String,

    /// A list of available torrents for the movie.
    torrents: Vec<YtsTorrent>,
}

/// Represents a single torrent for a movie in the YTS API response.
#[derive(Debug, Deserialize)]
struct YtsTorrent {
    /// The hash of the torrent, used in magnet links.
    hash: String,

    /// The number of seeders for the torrent.
    seeds: u32,

    /// The number of leechers for the torrent.
    peers: u32,

    /// The size of the torrent as a string, e.g., "700MB".
    size: String,
}

impl YtsTorrent {
    /// Converts the size string into bytes.
    fn size_bytes(&self) -> u64 {
        let size = self.size.to_lowercase();
        if size.ends_with("gb") {
            (size.trim_end_matches("gb").parse::<f64>().unwrap_or(0.0) * 1_000_000_000.0) as u64
        } else if size.ends_with("mb") {
            (size.trim_end_matches("mb").parse::<f64>().unwrap_or(0.0) * 1_000_000.0) as u64
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    /// Sets up a mock YTS provider using a mock server.
    async fn setup_mock_provider() -> Yts {
        Yts {
            api_url: Server::new_async().await.url(),
        }
    }

    /// Tests building a request with a valid query and a category.
    ///
    /// Ensures that the request includes the `query_term` parameter for the query
    /// and the `genre` parameter for the category.
    #[tokio::test]
    async fn test_build_request_with_category() {
        let provider = setup_mock_provider().await;
        let client = Client::new();

        let search_request = SearchRequest::new("Inception").add_category(Category::Movies);
        let request = provider.build_request(&client, search_request);

        assert!(request.is_ok());
        let request = request.unwrap();
        assert_eq!(request.method(), "GET");
        assert!(request.url().as_str().contains("query_term=Inception"));
        assert!(request.url().as_str().contains("genre=movie"));
    }

    /// Tests building a request with a valid query but no category.
    ///
    /// Ensures that the request includes the `query_term` parameter for the query
    /// but does not include a `genre` parameter.
    #[tokio::test]
    async fn test_build_request_no_category() {
        let provider = setup_mock_provider().await;
        let client = Client::new();

        let search_request = SearchRequest::new("Inception");
        let request = provider.build_request(&client, search_request);

        assert!(request.is_ok());
        let request = request.unwrap();
        assert!(request.url().as_str().contains("query_term=Inception"));
        assert!(!request.url().as_str().contains("genre="));
    }

    /// Tests parsing a valid API response into a list of torrents.
    ///
    /// Ensures that the response is correctly parsed into a `Torrent` struct
    /// with all expected fields populated.
    #[tokio::test]
    async fn test_parse_response() {
        let provider = setup_mock_provider().await;

        let response_body = r#"
        {
            "status": "ok",
            "status_message": "Query was successful",
            "data": {
                "movies": [
                    {
                        "title": "Inception",
                        "torrents": [
                            {
                                "hash": "abc123",
                                "seeds": 200,
                                "peers": 50,
                                "size": "1.5GB"
                            }
                        ]
                    }
                ]
            }
        }
        "#;

        let result = provider.parse_response(response_body);

        assert!(result.is_ok());
        let torrents = result.unwrap();
        assert_eq!(torrents.len(), 1);
        let torrent = &torrents[0];
        assert_eq!(torrent.name, "Inception");
        assert_eq!(torrent.magnet_link, "magnet:?xt=urn:btih:abc123");
        assert_eq!(torrent.seeders, 200);
        assert_eq!(torrent.peers, 50);
        assert_eq!(torrent.size_bytes, 1_500_000_000);
        assert_eq!(torrent.provider, "yts");
    }

    /// Tests handling of invalid JSON responses from the API.
    ///
    /// Ensures that an invalid JSON string results in a `DataParseError`.
    #[tokio::test]
    async fn test_parse_response_invalid_json() {
        let provider = setup_mock_provider().await;

        let invalid_response_body = r#"not valid json"#;

        let result = provider.parse_response(invalid_response_body);

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), ClientError::DataParseError(_)),
            "Expected ClientError::DataParseError"
        );
    }

    /// Tests handling of empty movie data in the API response.
    ///
    /// Ensures that an empty movie list results in no torrents being parsed.
    #[tokio::test]
    async fn test_parse_response_empty_data() {
        let provider = setup_mock_provider().await;

        let response_body = r#"
    {
        "status": "ok",
        "status_message": "Query was successful",
        "data": {
            "movie_count": 0,
            "limit": 20,
            "page_number": 1
        }
    }
    "#;

        let result = provider.parse_response(response_body);

        assert!(result.is_ok());
        let torrents = result.unwrap();
        assert!(
            torrents.is_empty(),
            "Expected empty results due to no movies"
        );
    }
}
