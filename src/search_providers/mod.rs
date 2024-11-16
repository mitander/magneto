use async_trait::async_trait;
use reqwest::{Client, Request};

use crate::{errors::ClientError, SearchRequest, Torrent};

pub mod knaben;
pub mod piratebay;

pub use knaben::Knaben;
pub use piratebay::PirateBay;

#[async_trait]
pub trait SearchProvider: Send + Sync {
    async fn send_request(
        &self,
        client: &Client,
        request: SearchRequest<'_>,
    ) -> Result<Vec<Torrent>, ClientError> {
        let request = self.build_request(client, request)?;
        println!(
            "Client sending {} request to {} with {} bytes of data",
            request.method(),
            request.url(),
            request.body().as_slice().len()
        );

        let response = client
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

        self.parse_response(&response_content)
    }

    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError>;

    fn build_request(
        &self,
        client: &Client,
        request: SearchRequest<'_>,
    ) -> Result<Request, ClientError>;

    fn id(&self) -> String;
}
