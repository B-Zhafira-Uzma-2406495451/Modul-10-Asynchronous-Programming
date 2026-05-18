use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Sender, channel};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            let formatted_msg = format!("[{}]: {}", addr, text);
                            let _ = bcast_tx.send(formatted_msg);
                        }
                    }
                    _ => break,
                }
            }
            msg = bcast_rx.recv() => {
                match msg {
                    Ok(text) => {
                        ws_stream.send(Message::text(text)).await?;
                    }
                    Err(_e) => break,
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (bcast_tx, _) = channel(16);
    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("listening on port 2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {:?}", addr);
        let bcast_tx = bcast_tx.clone();

        tokio::spawn(async move {
            if let Ok((_req, ws_stream)) = ServerBuilder::new().accept(socket).await {
                if let Err(e) = handle_connection(addr, ws_stream, bcast_tx).await {
                    eprintln!("Error handling connection from {:?}: {}", addr, e);
                }
            }
        });
    }
}