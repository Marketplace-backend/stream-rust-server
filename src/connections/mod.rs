use crate::debug;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Conn {
    pub stream: Arc<Mutex<TcpStream>>,
    pub connections: Connections,
}

impl Conn {
    pub(crate) async fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.lock().await.read(buf).await
    }
    async fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        match self.stream.try_lock() {
            Ok(mut lock) => lock.write(buf).await,
            Err(_e) => Ok(0),
        }
    }
    pub(crate) async fn take_error(&self) -> std::io::Result<Option<std::io::Error>> {
        self.stream.lock().await.take_error()
    }
}

#[derive(Clone)]
pub struct Connections {
    pub counter: Arc<Mutex<u32>>,
    pub connections: Arc<Mutex<HashMap<u32, Conn>>>,
}

impl Connections {
    pub(crate) async fn store(&self, conn: Conn) -> u32 {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        let id = *counter;
        self.connections.lock().await.insert(id, conn);
        id
    }
    pub(crate) async fn remove(&self, id: u32) {
        self.connections.lock().await.remove(&id);
    }

    #[allow(unused_variables)]
    pub(crate) async fn broadcast(&self, buf: Vec<u8>) {
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
