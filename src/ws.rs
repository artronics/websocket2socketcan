use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use log::*;
use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;

use super::can::{Can, CanFrame};

type Tx = UnboundedSender<Message>;
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;
type CanFrameHandler = fn(&Can, CanFrame);

pub async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr, can: &Can) {
    info!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    info!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        info!("Received a message from {}: {}", addr, msg.to_text().unwrap());
        let peers = peer_map.lock().unwrap();
        let f = CanFrame::new(0x123, [1; 8], 8, false, false);
        can.write(f);

        // We want to broadcast the message to everyone except ourselves.
        // let broadcast_recipients =
        //     peers.iter().filter(|(peer_addr, _)| peer_addr != &&addr).map(|(_, ws_sink)| ws_sink);
        let broadcast_recipients =
            peers.iter().map(|(_, ws_sink)| ws_sink);


        for recp in broadcast_recipients {
            info!("sending message");
            recp.unbounded_send(msg.clone()).unwrap();
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    info!("{} disconnected", &addr);
    peer_map.lock().unwrap().remove(&addr);
}

#[cfg(test)]
mod tests {
    use log::*;
    use tokio::net::TcpListener;

    use crate::ws::accept_connection;

    #[test]
    fn it_works() {
        env_logger::init();

        let addr = "127.0.0.1:9002";
        let listener = TcpListener::bind(&addr).await.expect("Can't listen");
        info!("Listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream.peer_addr().expect("connected streams should have a peer address");
            info!("Peer address: {}", peer);

            tokio::spawn(accept_connection(peer, stream));
        }
    }
}
