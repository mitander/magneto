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
    #[error("error parsing endpoint into data")]
    DataParseError(anyhow::Error),

    /// Represents an error returned by a server during a request.
    ///
    /// # Fields
    /// - `code`: The HTTP status code returned by the server.
    /// - `content`: An optional string containing the response content, if available.
    #[error("server returned error")]
    ServerResponseError {
        /// The HTTP status code from the server's response.
        code: reqwest::StatusCode,
        /// Optional content of the server's error response.
        content: Option<String>,
    },

    /// Represents an error that occurs while retrieving an HTTP response.
    #[error("error retrieving http response")]
    ResponseError {
        /// The underlying error that caused the response retrieval to fail.
        source: anyhow::Error,
    },

    /// Represents an error that occurs when building an HTTP request.
    ///
    /// # Fields
    /// - `source`: The underlying error that caused the failure.
    /// - `url`: The URL that was being used when the error occurred.
    #[error("error building http request")]
    RequestBuildError {
        /// The underlying error that caused the request-building failure.
        source: anyhow::Error,
        /// The URL being used when the error occurred.
        url: String,
    },
}
