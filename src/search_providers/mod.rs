use async_trait::async_trait;
use std::error::Error;

pub mod knaben;
pub mod piratebay;

pub use knaben::Knaben;
pub use piratebay::PirateBay;

#[allow(dead_code)]
#[derive(Default)]
pub struct Options {
    disable_knaben: bool,
    number_of_results: u16,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Magneto {
    providers: Vec<Box<dyn SearchProvider>>,
    options: Options,
}

impl Magneto {
    pub fn new(opts: Options) -> Self {
        Magneto {
            options: opts,
            providers: vec![Box::new(Knaben::new()), Box::new(PirateBay::new())],
        }
    }

    pub async fn search(&self, query: &str) -> Vec<Torrent> {
        let results = Vec::new();
        let provider = if self.options.disable_knaben {
            &self.providers[1]
        } else {
            &self.providers[0]
        };

        match provider.search(query).await {
            Ok(t) => t,
            Err(err) => {
                println!("err: {}", err);
                results
            }
        }
    }
}

#[allow(dead_code)]
pub struct Torrent {
    pub name: String,
    pub magnet_link: String,
    pub seeders: Option<u32>,
    pub leechers: Option<u32>,
    pub size_bytes: Option<u64>,
}

#[async_trait]
pub trait SearchProvider {
    async fn search(&self, query: &str) -> Result<Vec<Torrent>, Box<dyn Error + Send + Sync>>;
}
