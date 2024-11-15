use magneto::{Magneto, SearchRequest};

#[tokio::main]
async fn main() {
    let magneto = Magneto::new();
    let req = SearchRequest::new("Interstellar", None);

    match magneto.search(req).await {
        Ok(results) => {
            for res in results {
                println!(
                    "name:{}, magnet:{}, provider:{} seeders:{} peers:{}",
                    res.name, res.magnet_link, res.provider, res.seeders, res.peers
                );
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }
}
