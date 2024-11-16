use crate::errors::ClientError;

use reqwest::{header::CONTENT_TYPE, Client, Request};

#[derive(Debug, Clone)]
pub enum RequestType<'a> {
    Get(&'a [(&'a str, &'a str)]),
    Post(&'a serde_json::Value),
}

#[derive(Default)]
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        HttpClient {
            client: Client::new(),
        }
    }

    pub fn with_client(client: Client) -> Self {
        HttpClient { client }
    }
}

impl HttpClient {
    pub async fn request(
        &self,
        url: &str,
        request_type: RequestType<'_>,
    ) -> Result<String, ClientError> {
        let request = self.build_request(url, request_type)?;
        self.send_request(request).await
    }

    async fn send_request(&self, request: Request) -> Result<String, ClientError> {
        println!(
            "Client sending {} request to {} with {} bytes of data",
            request.method(),
            request.url(),
            request.body().as_slice().len()
        );

        let response = self
            .client
            .execute(request)
            .await
            .map_err(|e| ClientError::ResponseError { source: e.into() })?;

        let status = response.status();
        let response_content = response
            .text()
            .await
            .map_err(|e| ClientError::ResponseError { source: e.into() })?;

        println!(
            "Client received {} response with {} bytes of body data",
            status,
            response_content.len()
        );

        if !status.is_success() {
            return Err(ClientError::ServerResponseError {
                code: status,
                content: Some(response_content.clone()),
            });
        }

        Ok(response_content)
    }

    fn build_request(&self, url: &str, request_type: RequestType) -> Result<Request, ClientError> {
        println!("Building request for {}", url);

        let request_builder = match request_type {
            RequestType::Get(params) => self.client.get(url).query(params),
            RequestType::Post(body) => self
                .client
                .post(url)
                .header(CONTENT_TYPE, "application/json")
                .body(body.to_string()),
        };

        request_builder
            .build()
            .map_err(|e| ClientError::RequestBuildError {
                source: e.into(),
                url: url.to_string(),
            })
    }
}
