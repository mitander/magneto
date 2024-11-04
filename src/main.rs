mod search_providers;
use search_providers::{Magneto, Options};

#[tokio::main]
async fn main() {
    let opts = Options::default();
    let magneto = Magneto::new(opts);

    let results = magneto.search("Interstellar");
    for result in results.await {
        println!("name:{}, magnet:{}", result.name, result.magnet_link);
    }
}
