mod search_providers;
use search_providers::{Magneto, Options, Provider, SearchRequest};

#[tokio::main]
async fn main() {
    let magneto = Magneto::new(Provider::Knaben, Options::default());

    let req = SearchRequest::new("Interstellar".to_string(), None);
    let results = magneto.search(req);
    for result in results.await {
        println!("name:{}, magnet:{}", result.name, result.magnet_link);
    }
}
