use async_trait::async_trait;
use log::debug;
use reqwest::{Client, Request};

use crate::{errors::ClientError, SearchRequest, Torrent};

pub mod knaben;
pub mod piratebay;

pub use knaben::Knaben;
pub use piratebay::PirateBay;

/// The `SearchProvider` trait defines the interface for implementing
/// search providers to query and parse torrent metadata.
///
/// Implementors of this trait are responsible for:
/// - Constructing provider-specific HTTP requests.
/// - Parsing responses into a common format (`Torrent`).
/// - Providing a unique identifier for the provider.
///
/// This trait includes a default implementation for sending requests (`send_request`),
/// which handles the common request-sending logic.
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Sends a search request to the provider's API, processes the response,
    /// and parses it into a list of torrents.
    ///
    /// # Parameters
    /// - `client`: The `reqwest::Client` used for making HTTP requests.
    /// - `request`: A `SearchRequest` containing the search parameters.
    ///
    /// # Returns
    /// - `Ok(Vec<Torrent>)`: A list of parsed torrents on success.
    /// - `Err(ClientError)`: An error if the request or parsing fails.
    async fn send_request(
        &self,
        client: &Client,
        request: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, ClientError> {
        let request = self.build_request(client, request)?;
        debug!(
            "client sending {} request to {} with {} bytes of data",
            request.method(),
            request.url(),
            request.body().as_slice().len()
        );

        let response = client
            .execute(request)
            .await
            .map_err(|e| ClientError::ResponseError(e.into()))?;

        let response_status = response.status();
        let response_content = response
            .text()
            .await
            .map_err(|e| ClientError::ResponseError(e.into()))?;

        debug!(
            "client received {} response with {} bytes of body data",
            response_status,
            response_content.len()
        );

        if !response_status.is_success() {
            return Err(ClientError::ServerResponseError {
                code: response_status,
                content: response_content.clone(),
            });
        }

        self.parse_response(&response_content)
    }

    /// Parses the response body from the provider's API into a list of torrents.
    ///
    /// # Parameters
    /// - `response`: The raw HTTP response body as a string.
    ///
    /// # Returns
    /// - `Ok(Vec<Torrent>)`: A list of parsed torrents on success.
    /// - `Err(ClientError)`: An error if the parsing fails.
    ///
    /// Implementors should define custom parsing logic for the specific
    /// response format of the provider's API.
    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError>;

    /// Builds an HTTP request for the provider's API.
    ///
    /// # Parameters
    /// - `client`: The `reqwest::Client` used to build the request.
    /// - `request`: A `SearchRequest` containing the search parameters.
    ///
    /// # Returns
    /// - `Ok(Request)`: The constructed HTTP request.
    /// - `Err(ClientError)`: An error if request building fails.
    ///
    /// Implementors should include provider-specific details, such as
    /// headers, query parameters, or request body content.
    fn build_request(
        &self,
        client: &Client,
        request: SearchRequest<'_>,
    ) -> Result<Request, ClientError>;

    /// Returns a unique identifier for the provider.
    ///
    /// # Returns
    /// - `String`: A unique string identifying the provider.
    ///
    /// This identifier can be used for distinguishing between different
    /// providers in a multi-provider setup.
    fn id(&self) -> String;
}

#[cfg(test)]
mod tests {
    use core::panic;

    use super::*;
    use async_trait::async_trait;
    use mockito::Server;
    use reqwest::Client;
    use serde_json::json;

    /// A mock implementation of the `SearchProvider` trait for testing purposes.
    struct MockProvider {
        url: String,
    }

    impl MockProvider {
        fn new(url: &str) -> Self {
            Self {
                url: url.to_string(),
            }
        }
    }

    #[async_trait]
    impl SearchProvider for MockProvider {
        fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
            let parsed: Vec<Torrent> = serde_json::from_str(response)
                .map_err(|e| ClientError::DataParseError(e.into()))?;
            Ok(parsed)
        }

        fn build_request(
            &self,
            client: &Client,
            request: SearchRequest<'_>,
        ) -> Result<Request, ClientError> {
            let request = client
                .get(format!("{}/search", self.url.clone()))
                .query(&[("q", request.query)])
                .build()
                .unwrap();
            Ok(request)
        }

        fn id(&self) -> String {
            self.url.clone()
        }
    }

    /// Tests that the `send_request` method successfully parses a valid response.
    ///
    /// This test uses the `mockito` library to mock an HTTP response with valid torrent
    /// data. It verifies that the parsed torrents match the expected values.
    #[tokio::test]
    async fn test_send_request_success() {
        let mut server = Server::new_async().await;
        let provider = MockProvider::new(&server.url());
        let client = Client::new();

        let _mock = server
            .mock("GET", "/search?q=ubuntu")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!([{
                    "name": "Ubuntu ISO",
                    "magnet_link": "magnet:?xt=urn:btih:abc123",
                    "seeders": 20,
                    "peers": 10,
                    "size_bytes": 2048,
                    "provider": server.url()
                }])
                .to_string(),
            )
            .create();

        let search_request = SearchRequest::new("ubuntu");
        let result = provider.send_request(&client, search_request).await;

        assert!(result.is_ok());
        let torrents = result.unwrap();
        assert_eq!(torrents.len(), 1);

        let torrent = &torrents[0];
        assert_eq!(torrent.name, "Ubuntu ISO");
        assert_eq!(torrent.magnet_link, "magnet:?xt=urn:btih:abc123");
        assert_eq!(torrent.seeders, 20);
        assert_eq!(torrent.peers, 10);
        assert_eq!(torrent.size_bytes, 2048);
        assert_eq!(torrent.provider, server.url());
    }

    /// Tests that the `send_request` method handles an HTTP error response.
    ///
    /// This test mocks an HTTP response with a `500 Internal Server Error` status.
    /// It verifies that the `ServerResponseError` is returned with the correct details.
    #[tokio::test]
    async fn test_send_request_error_response() {
        let mut server = Server::new_async().await;
        let provider = MockProvider::new(&server.url());
        let client = Client::new();

        let _mock = server
            .mock("GET", "/search?q=ubuntu")
            .with_status(500)
            .with_body("Internal Server Error")
            .create();

        let search_request = SearchRequest::new("ubuntu");
        let result = provider.send_request(&client, search_request).await;

        assert!(result.is_err());
        if let ClientError::ServerResponseError { code, content } = result.unwrap_err() {
            assert_eq!(code.as_u16(), 500);
            assert_eq!(content, "Internal Server Error");
        } else {
            panic!("Expected ServerResponseError");
        }
    }
}
