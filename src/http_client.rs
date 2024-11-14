use std::error::Error;

use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Empty, Full};
use hyper::client::conn::http1;
use hyper::{Method, Request, Uri};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn send_get_request(
        &self,
        url: Uri,
        query: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let host = url.host().expect("uri has no host");
        let port = url.port_u16().unwrap_or(80);
        let addr = format!("{}:{}", host, port);
        println!("{}", addr);

        let stream = TcpStream::connect(format!("{}/q.php?q={}:{}", host, query, port)).await?;
        let io = TokioIo::new(stream);

        let (mut sender, conn) = http1::handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let req = Request::builder()
            .method(Method::GET)
            .body(Empty::<Bytes>::new())
            .expect("failed to build GET request");

        let res = sender.send_request(req).await?;
        let body = res.collect().await?.to_bytes();
        Ok(serde_json::from_reader(body.reader())?)
    }

    pub async fn send_post_request(
        &self,
        url: Uri,
        query: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let host = url.host().expect("uri has no host");
        let port = url.port_u16().unwrap_or(80);
        let addr = format!("http://{}{}:{}", host, url.path(), port);

        println!("addr: {}", addr);
        let stream = TcpStream::connect(addr).await.unwrap();
        let io = TokioIo::new(stream);

        let (mut sender, conn) = http1::handshake(io).await.unwrap();
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                println!("Connection failed: {:?}", err);
            }
        });

        let req: Request<Full<Bytes>> = Request::builder()
            .method(Method::POST)
            .uri(url.to_string())
            .header("Content-Type", "application/json")
            // TODO: avoid to_string here?
            .body(Full::from(query.to_string()))
            .expect("failed to build POST request");
        println!("req: {:#?}", req);

        let res = sender.send_request(req).await?;
        println!("status: {}", res.status());
        let body = res.collect().await?.to_bytes();
        Ok(String::from_utf8(body.to_vec())?)
        // Ok(serde_json::from_reader(body.reader())?)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
