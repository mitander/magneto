# Magneto

`Magneto` is a library for searching torrents across multiple providers.
It provides a unified interface for querying torrent metadata and integrating
custom providers with ease.

## Features
- Query multiple torrent search providers simultaneously.
- Add custom providers with minimal effort.
- Retrieve results in a unified format with metadata like seeders, peers, and size.

## Examples

### Creating a `Magneto` instance and searching

```rust
use magneto::{Magneto, SearchRequest};

#[tokio::main]
async fn main() {
    let magneto = Magneto::new();

    let request = SearchRequest::new("Ubuntu", None);
    match magneto.search(request).await {
        Ok(results) => {
            for torrent in results {
                println!(
                    "found: {} (seeders: {}, peers: {})",
                    torrent.name, torrent.seeders, torrent.peers
                );
            }
        }
        Err(e) => eprintln!("error during search: {:?}", e),
    }
}
```

### Creating a `Magneto` instance from list of providers

```rust
use magneto::{
    search_providers::{Knaben, PirateBay, SearchProvider},
    Magneto, SearchRequest,
};

#[tokio::main]
async fn main() {
    let providers: Vec<Box<dyn SearchProvider>> =
        vec![Box::new(Knaben::new()), Box::new(PirateBay::new())];
    let _magneto = Magneto::with_providers(providers);
}

```

### Adding a custom provider

```rust
use magneto::{errors::ClientError, Magneto, SearchProvider, SearchRequest, Torrent};

use reqwest::{Client, Request};

struct CustomProvider;

impl CustomProvider {
    pub fn new() -> Self
        Self {}
    }

impl SearchProvider for CustomProvider {
    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
        todo!(); // parse response data into Vec<Torrent>
    }

    fn build_request(
        &self,
        client: &Client,
        request: SearchRequest<'_>,
    ) -> Result<Request, ClientError> {
        todo!(); // convert SearchRequest to reqwest::Request
    }

   fn id(&self) -> String {
       "custom_provider".to_string()
   }
}

#[tokio::main]
async fn main() {
    let mut magneto = Magneto::new();

    let custom_provider = CustomProvider::new();
    magneto.add_provider(Box::new(custom_provider));

    let request = SearchRequest::new("Ubuntu", None);
    match magneto.search(request).await {
        Ok(results) => {
            for torrent in results {
                println!(
                    "found: {} (provider: {})",
                    torrent.name, torrent.provider
                );
            }
        }
        Err(e) => eprintln!("error during search: {:?}", e),
    }
}
```

## License
[MIT](/LICENSE)
