use std::io::{Read, Write};
use std::net::TcpStream;
use crate::accounting::{DoubleEntryEngine, StorageBackend};
use crate::server::handler::parse_transfer;

/// Handle a single client connection
pub fn handle_client<S: StorageBackend>(
    mut stream: TcpStream,
    engine: &mut DoubleEntryEngine<S>,
) -> Result<(), String> {
    let mut buffer = [0; 4096];
    
    let n = stream.read(&mut buffer).map_err(|e| format!("Read error: {}", e))?;
    if n == 0 {
        return Ok(());
    }
    
    let request = String::from_utf8_lossy(&buffer[..n]);
    println!("📥 Received: {}", request.trim());
    
    // Parse and process
    match parse_transfer(&request) {
        Ok(transfer) => {
            match engine.process_transfer(&transfer) {
                Ok(result) => {
                    let response = format!(r#"{{"status":"ok","result":"{:?}"}}"#, result);
                    send_response(&mut stream, &response)?;
                    println!("📤 Success: {:?}", result);
                }
                Err(e) => {
                    let response = format!(r#"{{"status":"error","message":"{}"}}"#, e);
                    send_response(&mut stream, &response)?;
                    println!("❌ Error: {}", e);
                }
            }
        }
        Err(e) => {
            let response = format!(r#"{{"status":"error","message":"{}"}}"#, e);
            send_response(&mut stream, &response)?;
            println!("❌ Parse error: {}", e);
        }
    }
    
    Ok(())
}

fn send_response(stream: &mut TcpStream, response: &str) -> Result<(), String> {
    let full = format!("{}\n", response);
    stream.write_all(full.as_bytes()).map_err(|e| format!("Write error: {}", e))
}
