use std::collections::HashMap;
use std::env;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[cfg(debug_assertions)]
macro_rules! debug {
  ($( $args:expr ), *) => { println!( $( $args ), * ); }
}

#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($( $args:expr ),*) => {};
}

#[derive(Clone)]
struct Conn {
    stream: Arc<Mutex<TcpStream>>,
    connections: Connections,
}

impl Conn {
    async fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.lock().await.read(buf).await
    }
    async fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        match self.stream.try_lock() {
            Ok(mut lock) => lock.write(buf).await,
            Err(_e) => Ok(0),
        }
    }
    async fn take_error(&self) -> std::io::Result<Option<std::io::Error>> {
        self.stream.lock().await.take_error()
    }
}

#[derive(Clone)]
struct Connections {
    counter: Arc<Mutex<u32>>,
    connections: Arc<Mutex<HashMap<u32, Conn>>>,
}

impl Connections {
    async fn store(&self, conn: Conn) -> u32 {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        let id = *counter;
        self.connections.lock().await.insert(id, conn);
        id
    }
    async fn remove(&self, id: u32) {
        self.connections.lock().await.remove(&id);
    }

    #[allow(unused_variables)]
    async fn broadcast(&self, buf: Vec<u8>) {
        /* Loop over all connections in map and wite the given buffer */
        for (id, conn) in self.connections.lock().await.iter() {
            match conn.write(&buf).await {
                Ok(size) => {
                    debug!("[{}] Wrote {} to connection...", id, size);
                }
                Err(error) => {
                    debug!("[{}] Error writing to connection {}", id, error);
                }
            }
        }
    }
    pub fn new() -> Connections {
        Connections {
            counter: Arc::new(Mutex::new(0)),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[allow(clippy::read_zero_byte_vec)]
async fn handle_stream(conn: Conn) -> std::io::Result<()> {
    /* Store the connection in the shared map */
    let id = conn.connections.store(conn.clone()).await;
    println!("[{}] Connected...", id);
    /* Start loop to read from socket */
    loop {
        /* Close if there was an error */
        match conn.take_error().await {
            Ok(_) => {}
            Err(_e) => {
                break;
            }
        }

        /* Broadcast message to other sockets */
        let mut buf = vec![];
        match conn.read(&mut buf).await {
            /* If 0 bytes were read the socket has been closed */
            Ok(read) if read == 0 => {
                break;
            }
            Ok(_read) => {
                let video = tokio::fs::read("./examples/video.mp4").await?;
                /* Broadcast message to other sockets */
                conn.connections.broadcast(video).await;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                /* Sleep for blocking error, an implementation of wait_for_fd would be better */
                thread::sleep(Duration::from_millis(10));
            }
            Err(_e) => break,
        };
    }
    /* After loop finishes remove from shared map */
    conn.connections.remove(id).await;
    println!("[{}] Disconnected...", id);

    Ok(())
}

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

// async fn video_formatter(file: Vec<u8>, mut vec: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
//     if vec.len() == 0 {
//         let part_video = file.get(0..4096).unwrap_or(&[]).to_vec();
//         vec.push(part_video);
//         Ok(vec)
//     } else {
//         let part_video = file.get(vec[0].len()..vec[0].len() + 4096).unwrap_or(&[]).to_vec();
//         vec.clear();
//         vec.push(part_video);
//         Ok(vec)
//     }
// }
