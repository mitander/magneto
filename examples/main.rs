use log::info;
use magneto::{Magneto, SearchRequest};

#[tokio::main]
async fn main() {
    env_logger::init();

    let magneto = Magneto::new();
    let req = SearchRequest::new("Ubuntu", None);

    match magneto.search(req).await {
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
