mod magneto;
use crate::magneto::piratebay::PirateBay;
use crate::magneto::SearchProvider;

#[tokio::main]
async fn main() {
    let pb = PirateBay::new();

    match pb.search("Interstellar").await {
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
