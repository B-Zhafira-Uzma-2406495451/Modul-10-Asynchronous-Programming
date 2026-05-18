use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut ws_stream, _) = ClientBuilder::from_uri(Uri::from_static("ws://127.0.0.1:8080"))
        .connect()
        .await?;

    let stdin = tokio::io::stdin();
    let mut stdin_lines = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            line = stdin_lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        if !text.trim().is_empty() {
                            ws_stream.send(Message::text(text)).await?;
                        }
                    }
                    _ => break,
                }
            }
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("{}", text);
                        }
                    }
                    _ => {
                        println!("Koneksi ke chat server terputus.");
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}