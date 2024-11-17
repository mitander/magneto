<!-- cargo-sync-readme start -->

# Magneto

`Magneto` is a library for searching torrents across multiple providers.
It provides a unified interface for querying torrent metadata and integrating
custom providers.

## Features
- Query multiple torrent search providers simultaneously.
- Add custom providers with minimal effort.
- Retrieve results in a unified format with metadata like magnet link, seeders, peers, and size.

## Supported providers
- PirateBay (apibay.org)
- Knaben (knaben.eu)

## Examples

### Creating a `Magneto` instance and searching

```rust
use magneto::{Category, Magneto, SearchRequest};

#[tokio::main]
async fn main() {
    let magneto = Magneto::new();

    let request = SearchRequest::new("Ubuntu", Some(vec![Category::Software]));
    match magneto.search(request).await {
        Ok(results) => {
            for torrent in results {
                println!(
                    "found: {} from {} with magnet link {} (seeders: {}, peers: {})",
                    torrent.name,
                    torrent.provider,
                    torrent.magnet_link,
                    torrent.seeders,
                    torrent.peers,
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
    Magneto,
};

#[tokio::main]
async fn main() {
    // Create instance from list of providers
    let providers: Vec<Box<dyn SearchProvider>> =
        vec![Box::new(Knaben::new()), Box::new(PirateBay::new())];
    let _magneto = Magneto::with_providers(providers);

    // Or use add_provider() to add to list of active providers
    let mut magneto = Magneto::default(); // no providers
    magneto.add_provider(Box::new(Knaben::new()));
    magneto.add_provider(Box::new(PirateBay::new()));
}
```

### Adding a custom provider

```rust
use magneto::{errors::ClientError, Magneto, SearchProvider, SearchRequest, Torrent};
use reqwest::{Client, Request};

struct CustomProvider;

impl CustomProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl SearchProvider for CustomProvider {
    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
        todo!("parse response data into Vec<Torrent>");
    }

    fn build_request(
        &self,
        client: &Client,
        request: SearchRequest<'_>,
    ) -> Result<Request, ClientError> {
        todo!("convert SearchRequest to reqwest::Request");
    }

    fn id(&self) -> String {
        "custom_provider".to_string()
    }
}

#[tokio::main]
async fn main() {
    let custom_provider = CustomProvider::new();
    let mut magneto = Magneto::new();
    magneto.add_provider(Box::new(custom_provider));
}
```

<!-- cargo-sync-readme end -->

## License
[MIT](/LICENSE)
