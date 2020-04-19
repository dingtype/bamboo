use std::io::prelude::*;
use std::time::{Duration, Instant};
use std::net::{TcpListener, TcpStream};
use std::io::BufReader;
use std::thread;
use bytes::{BytesMut, BufMut};
use std::io::Error;

static ADDR: &str = "127.0.0.1:8081";

fn throttle(stream: TcpStream, download_rate: usize) -> Result<BytesMut, Error> {
    // Frame of reference is 1 second.
    let tf = Duration::new(1, 0);
    let zero = Duration::new(0, 0);
    
    // let mut buf = [0; 128][...];
    let mut reader = BufReader::new(stream);
    let mut buf = BytesMut::with_capacity(1024);

    let mut time_left = tf;
    let start = Instant::now();
    
    while time_left > zero {
	reader.read(&mut buf)?;

	let bytes_read = buf.capacity();

	time_left -= start.elapsed();

	let bytes_diff = download_rate - bytes_read;
	
	// If there is time left before 1s and
	// there are still bytes left to read, keep reading.
	if bytes_diff > 0 {
	    continue
	} else {
	    // Should sleep and wait until next second.
	    thread::sleep(time_left);
	}
    }

    Ok(buf)
}

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 10];
    let len = stream.peek(&mut buf).expect("peek failed");
    println!("Length: {}", len);
    
    // stream.write(&[1])?;
    stream.set_read_timeout(None).expect("set_readtimeout failed");
    
    stream.read(&mut [0; 128]).expect("fail to read");
}
    
fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(ADDR)?;

    for stream in listener.incoming() {
	handle_client(stream?);
    }

    Ok(())
}
