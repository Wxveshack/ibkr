use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    // TCP Connection
    println!("Connecting to TWS...");
    let mut stream = TcpStream::connect("127.0.0.1:7496")?;

    // IBKR Handshake: "API\0" + length-prefixed version string
    let version_str = "v100..176";
    let mut handshake = b"API\0".to_vec();
    handshake.extend((version_str.len() as u32).to_be_bytes());
    handshake.extend(version_str.as_bytes());
    stream.write_all(&handshake)?;

    // Read server version response
    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;
    let fields: Vec<&str> = std::str::from_utf8(&buf[4..n])
        .unwrap()
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect();
    let server_version: u32 = fields[0].parse().unwrap();
    println!("Connected to TWS v{} at {}", server_version, fields.get(1).unwrap_or(&""));

    // START_API (msgId=71): version=2, clientId=1, optionalCapabilities=""
    send_msg(&mut stream, "71\02\01\0\0")?;
    println!("Sent START_API");

    // Give TWS a moment and read initial responses
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Read whatever TWS sends after START_API
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
    let mut recv_buf = Vec::new();

    println!("\n--- Initial messages from TWS ---");
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                recv_buf.extend_from_slice(&buf[..n]);
                while let Some((msg, rest)) = extract_message(&recv_buf) {
                    print_message(&msg);
                    recv_buf = rest;
                }
            }
            Err(_) => break,
        }
    }

    println!("\n--- Requesting account data ---");
    // REQ_ACCT_DATA (msgId=6): version=2, subscribe=1, acctCode=""
    send_msg(&mut stream, "6\02\01\0\0")?;
    println!("Sent REQ_ACCT_DATA\n");

    // Read responses
    stream.set_read_timeout(Some(std::time::Duration::from_secs(3)))?;
    let mut recv_buf = Vec::new();

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                recv_buf.extend_from_slice(&buf[..n]);
                while let Some((msg, rest)) = extract_message(&recv_buf) {
                    print_message(&msg);
                    recv_buf = rest;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => break,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Send a length-prefixed message
fn send_msg(stream: &mut TcpStream, payload: &str) -> std::io::Result<()> {
    let bytes = payload.as_bytes();
    stream.write_all(&(bytes.len() as u32).to_be_bytes())?;
    stream.write_all(bytes)?;
    Ok(())
}

/// Extract a complete message from buffer, return (message, remaining)
fn extract_message(buf: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    if buf.len() < 4 {
        return None;
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if buf.len() >= 4 + len {
        let msg = buf[4..4 + len].to_vec();
        let rest = buf[4 + len..].to_vec();
        Some((msg, rest))
    } else {
        None
    }
}

/// Parse and print an IBKR message
fn print_message(buf: &[u8]) {
    let fields: Vec<&str> = std::str::from_utf8(buf)
        .unwrap_or("")
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect();

    if fields.is_empty() {
        return;
    }

    let msg_id: u32 = fields[0].parse().unwrap_or(0);
    let data = &fields[1..];

    match msg_id {
        4 => println!("ERROR: {:?}", data),
        6 => println!("ACCT_VALUE: {} = {} {} ({})",
            data.get(0).unwrap_or(&"?"),
            data.get(1).unwrap_or(&"?"),
            data.get(2).unwrap_or(&""),
            data.get(3).unwrap_or(&"")),
        _ => println!("MSG[{}]: {:?}", msg_id, data),
    }
}
