use bytes::{BufMut, BytesMut};
use std::convert::TryInto;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Error;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

static ADDR: &str = "127.0.0.1:8081";

// NOTE: download_rate is kb/s.
fn throttle(mut stream: TcpStream, download_rate: u64, latency_ms: u64) -> Result<BytesMut, Error> {
    let second = Duration::new(1, 0);
    let zero = Duration::new(0, 0);

    let latency = Duration::from_millis(latency_ms);

    // Convert kbits to bytes
    let n_bytes = download_rate * 125;

    let ns_per_byte: u128 = (1000_000_000 / n_bytes).into();

    let mut reader = BufReader::new(&stream);
    let mut out = BytesMut::with_capacity(1024);
    let mut buf = [0; 1];

    let start = Instant::now();
    let mut bytes_read = 0;
    let mut time_passed = zero;

    loop {
        if time_passed >= second {
            break;
        }

        let init = Instant::now();

        let len: u64 = reader.read(&mut buf)?.try_into().unwrap();

        out.put(&buf[..]);

        bytes_read += len;

        time_passed += start.elapsed();

        let a: u128 = init.saturating_duration_since(start).as_nanos();
        let b: u128 = Instant::now().saturating_duration_since(start).as_nanos();

        if (a + ns_per_byte) > b {
            let d = Duration::from_nanos(((a + ns_per_byte) - b).try_into().unwrap());
            println!("Stalling for {} seconds...", d.as_secs_f64());
            thread::sleep(d);
        }
    }

    let end = time_passed;
    println!("out {:#?}", out);
    println!("taken {:#?}", end);
    Ok(out)
}

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 10];
    let len = stream.peek(&mut buf).expect("peek failed");
    println!("Length: {}", len);
    stream
        .set_read_timeout(None)
        .expect("set_readtimeout failed");
    stream.read(&mut [0; 128]).expect("fail to read");
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(ADDR)?;

    for stream in listener.incoming() {
        let result = throttle(stream?, 25000, 0)?;
        println!("{:#?}", result);
    }

    Ok(())
}

/*
#[cfg(test)]
pub mod tests {

    use super::*;

    #[test]
    pub fn test_kbits_to_kbytes() {
    assert_eq!(kbits_to_kbytes(1), 125);
    }
}
*/
