use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::accounting::{DoubleEntryEngine, StorageBackend};
use crate::config::Config;
use crate::demo::setup_demo_accounts;
use crate::server::client::handle_client;

pub mod client;
pub mod handler;

/// The main server that accepts connections
pub struct Server<S: StorageBackend> {
    config: Config,
    engine: Arc<Mutex<DoubleEntryEngine<S>>>,
}

impl<S: StorageBackend + Send + 'static> Server<S> {
    pub fn new(config: Config, storage: S) -> Self {
        let engine = DoubleEntryEngine::new(storage);
        
        Self {
            config,
            engine: Arc::new(Mutex::new(engine)),
        }
    }
    
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🦁 SimbaLedger — The Lion of Ledgers");
        println!("   Starting up...");
        
        // Setup demo accounts if enabled
        if self.config.demo_accounts {
            let mut engine = self.engine.lock().unwrap();
            setup_demo_accounts(&mut engine, self.config.demo_balance)?;
        }
        
        // Start server
        let addr = self.config.addr();
        let listener = TcpListener::bind(&addr)?;
        println!("   Listening on {}", addr);
        println!("\n   Send JSON transfers to http://{}", addr);
        println!("   Example: echo '{{\"id\":1,\"debit_account_id\":1,\"credit_account_id\":2,\"amount\":100}}' | nc localhost 3000");
        println!("\n   Press Ctrl+C to stop\n");
        
        // Accept connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let engine = Arc::clone(&self.engine);
                    thread::spawn(move || {
                        let mut engine = engine.lock().unwrap();
                        if let Err(e) = handle_client(stream, &mut engine) {
                            eprintln!("Client error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
        
        Ok(())
    }
}
