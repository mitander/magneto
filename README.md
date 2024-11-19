<!-- cargo-sync-readme start -->

# Magneto
[<img alt="github" src="https://img.shields.io/badge/github-mitander/magneto-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/mitander/magneto)
[<img alt="crates.io" src="https://img.shields.io/crates/v/magneto.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/magneto)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-magneto-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/magneto)

`Magneto` is a library for searching torrents across multiple providers.
It provides a unified interface for querying torrent metadata and integrating
custom providers.

## Features
- Fully async-powered using `reqwest` and `tokio`.
- Query multiple torrent search providers simultaneously.
- Retrieve torrent results in a unified format.
- Add custom providers with minimal effort.

## Supported providers
- Knaben: A multi search archiver, acting as a cached proxy towards multiple different trackers.
- PirateBay: The galaxy’s most resilient Public BitTorrent site.
- YTS: A public torrent site specialising in HD movies of small size.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
magneto = "0.1"
```

Then:

```rust
use magneto::{Magneto, SearchRequest};

#[tokio::main]
async fn main() {
    let magneto = Magneto::new();

    let request = SearchRequest::new("Ubuntu");
    let results = magneto.search(request).await.unwrap();

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
```

### Specifying search providers

```rust
use magneto::{Magneto, Knaben, PirateBay, Yts};

// By default, all built-in providers are used (Knaben, PirateBay, Yts)
let magneto = Magneto::new();

// You can specify which providers to use like this
let magneto =
    Magneto::with_providers(vec![Box::new(Knaben::new()), Box::new(PirateBay::new())]);

// Or like this
let magneto = Magneto::default()
    .add_provider(Box::new(Knaben::new()))
    .add_provider(Box::new(Yts::new()));
```

### Search request parameters

```rust
use magneto::{Category, SearchRequest, OrderBy};

// You can add categories to filter your search results
let request = SearchRequest::new("Ubuntu")
    .add_category(Category::Software)
    .add_categories(vec![Category::Audio, Category::Movies]);

// Or initialize the request like this for more customization
let request = SearchRequest {
    query: "Debian",
    order_by: OrderBy::Seeders,
    categories: vec![Category::Software],
    number_of_results: 10,
};
```

### Add a custom provider

```rust
use magneto::{ClientError, Magneto, SearchProvider, SearchRequest, Torrent, Client, Request};

struct CustomProvider;

impl SearchProvider for CustomProvider {
    fn build_request(
        &self,
        client: &Client,
        request: SearchRequest<'_>,
    ) -> Result<Request, ClientError> {
        // Convert SearchRequest parameters to a reqwest::Request
        unimplemented!();
    }

    fn parse_response(&self, response: &str) -> Result<Vec<Torrent>, ClientError> {
        // Parse the raw reponse into Vec<Torrent>
        unimplemented!();
    }


    fn id(&self) -> String {
        // Return a unique id, built-in providers use the provider url
        unimplemented!();
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
