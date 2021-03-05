use std::collections::HashMap;
use std::env;
use std::io::Error as IoError;
use std::sync::Mutex;

use log::*;
use socketcan::{CANFrame, CANSocket};
use tokio::net::TcpListener;

use ws::handle_connection;
use ws::PeerMap;

mod ws;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    env_logger::init();

    let d: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let f = CANFrame::new(0x123, &d, false, false).unwrap();
    // let s = CANSocket::open("can0").unwrap();

    let addr = env::args().nth(1).unwrap_or_else(|| "192.168.0.28:8080".to_string());

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }

    Ok(())
}
