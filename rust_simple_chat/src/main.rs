use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::broadcast;
use std::sync::Arc;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8081").await?;
    let (tx, _rx) = broadcast::channel::<String>(10);
    let tx = Arc::new(tx);

    loop {
        let (socket, _) = listener.accept().await?;
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            if let Err(e) = process(socket, tx, &mut rx).await {
                eprintln!("Error: {}", e);
            }
        });
    }
}

async fn process(
    mut socket: TcpStream,
    tx: Arc<broadcast::Sender<String>>,
    rx: &mut broadcast::Receiver<String>,
) -> Result<(), Box<dyn Error>> {
    let peer_addr = socket.peer_addr()?;

    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        tokio::select! {
            result = reader.read_line(&mut line) => {
                if result? == 0 {
                    break;
                }
                let msg = format!("{}: {}", peer_addr, line);
                tx.send(msg.clone())?;
                line.clear();
            },
            result = rx.recv() => {
                let msg = result?;
                writer.write_all(msg.as_bytes()).await?;
            },
        }
    }

    Ok(())
}
