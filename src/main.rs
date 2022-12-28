use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::slice;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<()> {
        let connection = tokio::net::TcpListener::bind("localhost:8080").await?;

    match connection.accept().await {
        Ok((_socket, addr)) => println!("new client: {:?}", addr),
        Err(e) => println!("couldn't get client: {:?}", e),
    }

    let mut video = File::open("./examples/minimal.mp4").await?;

    let mut vec = vec![] as Vec<u8>;

    let video_formatter = video_formatter(&mut video, vec).await?;

    loop {
        let (mut tcp_stream, _socket) = connection.accept().await?;
        tcp_stream.write_buf().await?;
    }
}

async fn video_formatter(file: &mut File, mut vec: Vec<u8>) -> Result<&mut [u8]> {
    if vec.len() == 0 {
        let bytes = [4096];
        let mut part_video = file.bytes().take(4096);
        vec.push(part_video.);
    }
    Ok(&mut [1_u8])
}