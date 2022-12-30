use crate::connections::Conn;
use std::time::Duration;
use std::{io, thread};

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! debug {
  ($( $args:expr ), *) => { println!( $( $args ), * ); }
}

#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! debug {
    ($( $args:expr ),*) => {};
}

#[allow(clippy::read_zero_byte_vec)]
pub async fn handle_stream(conn: Conn) -> std::io::Result<()> {
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
