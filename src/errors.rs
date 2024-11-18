//! # Errors
//!
//! Contains the common error enum used across this crate. The `ClientError`
//! enum is designed to represent errors that can occur while querying and
//! processing data from torrent providers.

use thiserror::Error;

/// Represents errors that can occur during client operations.
#[derive(Error, Debug)]
pub enum ClientError {
    /// Represents an error that occurs when parsing data from an endpoint fails.
    #[error("error parsing endpoint into data: {0}")]
    DataParseError(#[source] anyhow::Error),

    /// Represents an error returned by a server during a request.
    ///
    /// # Fields
    /// - `code`: The HTTP status code returned by the server.
    /// - `content`: An optional string containing the response content, if available.
    #[error("server returned error with status {code}: {content}")]
    ServerResponseError {
        /// The HTTP status code from the server's response.
        code: reqwest::StatusCode,
        /// Content of the server's error response.
        content: String,
    },
    /// Represents an error that occurs while retrieving an HTTP response.
    #[error("error retrieving http response: {0}")]
    ResponseError(#[source] anyhow::Error),

    /// Represents an error that occurs when building an HTTP request.
    ///
    /// # Fields
    /// - `source`: The underlying error that caused the failure.
    /// - `url`: The URL that was being used when the error occurred.
    #[error("error building http request to {url}: {source}")]
    RequestBuildError {
        /// The underlying error that caused the request-building failure.
        #[source]
        source: anyhow::Error,
        /// The URL being used when the error occurred.
        url: String,
    },
}
