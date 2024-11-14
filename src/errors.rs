#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    UrlQueryParseError {
        source: anyhow::Error,
    },
    UrlParseError {
        source: url::ParseError,
    },
    DataParseError {
        source: anyhow::Error,
    },
    ServerResponseError {
        code: u16,
        content: Option<String>,
    },
    RequestBuildError {
        source: anyhow::Error,
        method: reqwest::Method,
        url: String,
    },
    ReqwestBuildError {
        source: reqwest::Error,
    },
    ResponseError {
        source: anyhow::Error,
    },
    RequestError {
        source: anyhow::Error,
        url: String,
        method: String,
    },
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "hello")
    }
}
