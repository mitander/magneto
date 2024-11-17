use async_trait::async_trait;
use log::debug;
use reqwest::{Client, Request};
use url::Url;

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
    ///
    /// The default implementation:
    /// 1. Calls `build_request` to construct the HTTP request.
    /// 2. Logs the request and response details.
    /// 3. Executes the request using the provided HTTP client.
    /// 4. Calls `parse_response` to process the response body.
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
            .map_err(|e| ClientError::ResponseError { source: e.into() })?;

        let response_status = response.status();
        let response_content = response
            .text()
            .await
            .map_err(|e| ClientError::ResponseError { source: e.into() })?;

        debug!(
            "client received {} response with {} bytes of body data",
            response_status,
            response_content.len()
        );

        if !response_status.is_success() {
            return Err(ClientError::ServerResponseError {
                code: response_status,
                content: Some(response_content.clone()),
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
