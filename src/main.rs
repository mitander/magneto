mod search_providers;
use crate::search_providers::Knaben;
use crate::search_providers::PirateBay;
use crate::search_providers::SearchProvider;

#[tokio::main]
async fn main() {
    let _ = PirateBay::new();
    let knaben = Knaben::new();

    match knaben.search("Interstellar").await {
        Ok(t) => {
            for entry in t {
                println!("{} {}", entry.name, entry.magnet_link);
            }
        }
        Err(err) => {
            println!("err: {}", err);
            return;
        }
    }
}
