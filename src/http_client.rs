use crate::errors::ClientError;

use reqwest::{Method, Request};
use serde::Serialize;
use url::Url;

#[derive(Debug, Clone)]
pub enum RequestMethod {
    GET(String),
    POST(Vec<u8>),
}

#[derive(Default)]
pub struct Client {
    pub http: reqwest::Client,
}

impl Client {
    pub fn new(http: reqwest::Client) -> Self {
        Client { http }
    }
}

impl Client {
    pub async fn send_request(&self, req: Request) -> Result<Vec<u8>, ClientError> {
        println!(
            "Client sending {} request to {} with {} bytes of data",
            req.method(),
            req.url(),
            req.body().as_slice().len()
        );

        let url_err = req.url().to_string();
        let method_err = req.method().to_string();
        let response = self
            .http
            .execute(req)
            .await
            .map_err(|e| ClientError::RequestError {
                source: e.into(),
                url: url_err,
                method: method_err,
            })?;

        let status = response.status();
        let body = response
            .bytes()
            .await
            .map_err(|e| ClientError::ResponseError { source: e.into() })?
            .to_vec();

        println!(
            "Client received {} response with {} bytes of body data",
            status,
            body.len()
        );

        if !status.is_success() {
            return Err(ClientError::ServerResponseError {
                code: status,
                content: String::from_utf8(body).ok(),
            });
        }
        Ok(body)
    }

    pub fn build_request(&self, url: &str, method: RequestMethod) -> Result<Request, ClientError> {
        println!("Building endpoint request");
        let mut url = Url::parse(url).map_err(|e| ClientError::UrlParseError { source: e })?;
        let method_err = method.clone();
        let url_error = url.to_string();

        let req = match method {
            RequestMethod::POST(data) => self
                .http
                .request(Method::POST, url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(data)
                .build(),
            RequestMethod::GET(query) => {
                url.set_query(Some(query.as_str()));
                self.http.request(Method::GET, url).build()
            }
        };

        req.map_err(|e| ClientError::RequestBuildError {
            source: e.into(),
            method: method_err,
            url: url_error,
        })
    }
}

pub fn build_body(data: &impl Serialize) -> Result<Vec<u8>, ClientError> {
    let data = serde_json::to_string(data)
        .map_err(|e| ClientError::DataParseError { source: e.into() })?;
    Ok(match data.as_str() {
        "null" | "{}" => Vec::new(),
        _ => data.as_bytes().to_vec(),
    })
}

pub fn build_query(data: &impl Serialize) -> Result<String, ClientError> {
    serde_urlencoded::to_string(data)
        .map_err(|e| ClientError::UrlQueryParseError { source: e.into() })
}
