use log::info;
use magneto::{Category, Magneto, SearchRequest};

#[tokio::main]
async fn main() {
    env_logger::init();

    let magneto = Magneto::new();
    let request = SearchRequest::new("Ubuntu").add_category(Category::Software);

    match magneto.search(request).await {
        Ok(results) => {
            for res in results {
                info!(
                    "name:{}, magnet:{}, provider:{} seeders:{} peers:{}",
                    res.name, res.magnet_link, res.provider, res.seeders, res.peers
                );
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
