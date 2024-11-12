use magneto::{Magneto, Provider, SearchRequest};

#[tokio::main]
async fn main() {
    let magneto = Magneto::new(Provider::Knaben);
    let req = SearchRequest::new("Interstellar", None);

    match magneto.search(req).await {
        Ok(results) => {
            for res in results {
                println!("name:{}, magnet:{}", res.name, res.magnet_link);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
