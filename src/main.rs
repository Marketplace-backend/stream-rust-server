use connections::{Conn, Connections};
use handle_stream::handle_stream;
use std::env;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

mod connections;
mod handle_stream;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let connections = Connections::new(); /* Initialize struct containing all active connections */

    /* Parse arguments */
    let args: Vec<String> = env::args().collect();
    let port = if args.len() > 1 {
        args[1].parse::<u16>().expect("Port must be a number")
    } else {
        1300
    };
    let addr = if args.len() > 2 {
        IpAddr::from_str(&args[2]).expect("Address must be valid")
    } else {
        IpAddr::from(Ipv4Addr::new(127, 0, 0, 1))
    };

    /* Create TCPListener */
    let socket_addr = SocketAddr::from((addr, port));
    let socket = TcpListener::bind(socket_addr).await?;
    println!("Listening on {}", socket_addr);

    /* Accept connections in infinite loop */
    loop {
        match socket.accept().await {
            Ok((stream, _)) => {
                /* Set stream to non-blocking as read/write called from multiple threads */
                /* Store stream in Mutex for locking, and create struct to hold references */
                let conn = Conn {
                    stream: Arc::new(Mutex::new(stream)),
                    connections: connections.clone(),
                };
                /* Spawn the handler thread */
                tokio::spawn(handle_stream(conn)).await??;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                /* Sleep for a bit when blocking error, an implementation of wait_for_fd would be better */
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => panic!("Encountered IO error: {}", e),
        }
    }
}
