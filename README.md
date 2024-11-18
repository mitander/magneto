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
use magneto::{Category, Magneto, SearchRequest, OrderBy};

#[tokio::main]
async fn main() {
    let magneto = Magneto::new();

    // You can add categories which your search are filtered by.
    let request = SearchRequest::new("Ubuntu")
        .add_category(Category::Software)
        .add_categories(vec![Category::Audio, Category::Movies]);

    // Or initialize the request like this for more customization.
    let _request = SearchRequest {
        query: "Debian",
        query_imdb_id: false,
        order_by: OrderBy::Seeders,
        categories: vec![Category::Movies],
        number_of_results: 10,
    };

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
    let magneto = Magneto::with_providers(providers);

    // Or add new providers like this
    let magneto = Magneto::default()
        .add_provider(Box::new(Knaben::new()))
        .add_provider(Box::new(PirateBay::new()));
}
```

### Adding a custom provider

```rust
use magneto::{ClientError, Magneto, SearchProvider, SearchRequest, Torrent, Client, Request};

struct CustomProvider;

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
    let custom_provider = CustomProvider{};
    let magneto = Magneto::new().add_provider(Box::new(custom_provider));
}
```

<!-- cargo-sync-readme end -->

## License
[MIT](/LICENSE)
