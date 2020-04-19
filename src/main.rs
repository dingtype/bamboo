use std::io::prelude::*;
use std::time::{Duration, Instant};
use std::net::{TcpListener, TcpStream};
use std::io::BufReader;
use std::thread;
use bytes::BytesMut;
use std::io::Error;
use std::convert::TryInto;

static ADDR: &str = "127.0.0.1:8081";

fn kbits_to_kbytes(kbits: u64) -> u64 {
    kbits * (1000 / 8)
}

// NOTE: download_rate is kbit/s.
fn throttle(stream: TcpStream, download_rate: u64, latency_ms: u64) -> Result<BytesMut, Error> {
    // Frame of reference is 1 second.
    let tf = Duration::new(1, 0);
    let zero = Duration::new(0, 0);
    let latency = Duration::from_millis(latency_ms);

    let n_bytes: usize = kbits_to_kbytes(download_rate).try_into()
	.expect("Fail to convert to usize");
    
    // let mut buf = [0; 128][...];
    let mut reader = BufReader::new(stream);
    let mut buf = BytesMut::with_capacity(1024);

    let mut time_left = tf;
    let start = Instant::now();
    
    while time_left > zero {
	thread::sleep(latency);
	
	reader.read(&mut buf)?;

	let bytes_read = buf.capacity();

	time_left -= start.elapsed();

	let bytes_diff = n_bytes - bytes_read;
	
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

#[cfg(test)]
pub mod tests {
    
    use super::*;

    #[test]
    pub fn test_kbits_to_kbytes() {
	assert_eq!(kbits_to_kbytes(1), 125);
    }
}

