use crate::errors::ClientError;

use reqwest::{Method, Request};
use serde::Serialize;
use std::ops::RangeInclusive;
use url::Url;

pub const HTTP_SUCCESS_CODES: RangeInclusive<u16> = 200..=208;

pub struct Client {
    pub http: reqwest::Client,
    pub url: String,
}

impl Client {
    pub fn new(url: &str, http: reqwest::Client) -> Self {
        Client {
            url: url.to_string(),
            http,
        }
    }

    pub fn default(url: &str) -> Self {
        Client {
            url: url.to_string(),
            http: reqwest::Client::default(),
        }
    }
}

impl Client {
    pub async fn send_request(&self, req: Request) -> Result<Vec<u8>, ClientError> {
        println!(
            "Client sending {} request to {} with {} bytes of data",
            req.method(),
            req.url(),
            req.body().as_slice().len(),
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

        let status = response.status().as_u16();
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

        if !HTTP_SUCCESS_CODES.contains(&status) {
            return Err(ClientError::ServerResponseError {
                code: status,
                content: String::from_utf8(body).ok(),
            });
        }
        Ok(body)
    }

    pub fn build_request(
        &self,
        method: Method,
        query: Option<String>,
        data: Option<Vec<u8>>,
    ) -> Result<Request, ClientError> {
        println!("Building endpoint request");
        let uri = build_url(&self.url, query)?;

        let method_err = method.clone();
        let uri_err = uri.to_string();

        let req = match method {
            Method::POST => self
                .http
                .request(method, uri)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(data.unwrap())
                .build(),
            Method::GET => self.http.request(method, uri).build(),
            _ => unimplemented!(),
        };

        req.map_err(|e| ClientError::RequestBuildError {
            source: e.into(),
            method: method_err,
            url: uri_err,
        })
    }
}

pub fn build_body(object: &impl Serialize) -> Result<Vec<u8>, ClientError> {
    let parse_data = serde_json::to_string(object)
        .map_err(|e| ClientError::DataParseError { source: e.into() })?;
    Ok(match parse_data.as_str() {
        "null" | "{}" => "".as_bytes().to_vec(),
        _ => parse_data.as_bytes().to_vec(),
    })
}

pub fn build_query(object: &impl Serialize) -> Result<String, ClientError> {
    serde_urlencoded::to_string(object)
        .map_err(|e| ClientError::UrlQueryParseError { source: e.into() })
}

fn build_url(base: &str, query: Option<String>) -> Result<Url, ClientError> {
    let mut url = Url::parse(base).map_err(|e| ClientError::UrlParseError { source: e })?;
    if let Some(q) = query {
        url.set_query(Some(q.as_str()));
    }
    Ok(url)
}
