use simbaledger::config::Config;
use simbaledger::server::Server;
use simbaledger::storage::InMemoryStorage;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::default();
    
    // Create storage
    let storage = InMemoryStorage::new();
    
    // Create and run server
    let mut server = Server::new(config, storage);
    server.run()?;
    
    Ok(())
}
