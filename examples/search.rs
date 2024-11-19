use magneto::{Category, Knaben, Magneto, OrderBy, SearchRequest};

#[tokio::main]
async fn main() {
    // Only use Knaben provider
    let magneto = Magneto::default().add_provider(Box::new(Knaben::new()));

    // You can add categories which your search are filtered by.
    let request = SearchRequest::new("Ubuntu")
        .add_category(Category::Software)
        .add_categories(vec![Category::Audio, Category::Movies]);

    // Or initialize the request like this for more customization.
    let _request = SearchRequest {
        query: "Debian",
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
