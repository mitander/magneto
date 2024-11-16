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
        code: reqwest::StatusCode,
        content: Option<String>,
    },
    RequestBuildError {
        source: anyhow::Error,
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
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: fix this
        todo!()
    }
}
