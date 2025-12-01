use std::io::{Read, Write};
use std::net::TcpStream;

use ibkr::contract::Contract;
use ibkr::historical::{BarData, BarSize, Duration, HistoricalDataRequest, WhatToShow};
use ibkr::message::{IncomingMessageId, OutgoingMessageId};
use ibkr::wire::{extract_message, make_field, send_message, FieldIterator};

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
    println!(
        "Connected to TWS v{} at {}",
        server_version,
        fields.get(1).unwrap_or(&"")
    );

    // START_API (msgId=71): version=2, clientId=1, optionalCapabilities=""
    let start_api = format!(
        "{}{}{}{}",
        make_field(OutgoingMessageId::StartApi.as_u32()),
        make_field(2),  // version
        make_field(1),  // clientId
        make_field(""), // optionalCapabilities (required for server v72+)
    );
    send_message(&mut stream, &start_api)?;
    println!("Sent START_API");

    // Give TWS a moment and read initial responses
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Read whatever TWS sends after START_API
    stream.set_read_timeout(Some(std::time::Duration::from_secs(2)))?;
    let mut recv_buf = Vec::new();

    println!("\n--- Initial messages from TWS ---");
    drain_messages(&mut stream, &mut recv_buf, &mut buf);

    // Request account data
    // Note: For TWS (not Gateway), you may need to specify the account code
    // from the ManagedAccounts message (MSG[15])
    println!("\n--- Requesting account data ---");
    let req_acct = format!(
        "{}{}{}{}",
        make_field(OutgoingMessageId::ReqAccountData.as_u32()),
        make_field(2),  // version
        make_field(1),  // subscribe
        make_field(""), // acctCode (empty = first/only account)
    );
    send_message(&mut stream, &req_acct)?;
    println!("Sent REQ_ACCT_DATA\n");

    stream.set_read_timeout(Some(std::time::Duration::from_secs(3)))?;
    drain_messages(&mut stream, &mut recv_buf, &mut buf);

    // Request historical data for AMZN - last week of daily bars
    println!("\n--- Requesting historical data for AMZN (1 week, daily bars) ---");
    let contract = Contract::stock("AMZN", "SMART", "USD");
    let request = HistoricalDataRequest::new(1001, contract)
        .duration(Duration::Weeks(1))
        .bar_size(BarSize::Day1)
        .what_to_show(WhatToShow::Trades)
        .use_rth(true);

    send_message(&mut stream, &request.encode())?;
    println!("Sent REQ_HISTORICAL_DATA (reqId=1001)\n");

    stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
    drain_messages(&mut stream, &mut recv_buf, &mut buf);

    Ok(())
}

/// Read and process all available messages from the stream.
fn drain_messages(stream: &mut TcpStream, recv_buf: &mut Vec<u8>, buf: &mut [u8]) {
    loop {
        match stream.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                recv_buf.extend_from_slice(&buf[..n]);
                while let Some((msg, rest)) = extract_message(recv_buf) {
                    handle_message(&msg);
                    *recv_buf = rest;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => break,
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
    }
}

/// Handle a decoded IBKR message.
fn handle_message(buf: &[u8]) {
    let mut fields = FieldIterator::new(buf);

    let Some(msg_id) = fields.next::<u32>() else {
        return;
    };

    match IncomingMessageId::from_u32(msg_id) {
        Some(IncomingMessageId::Error) => {
            let _version = fields.next_i32();
            let id = fields.next_i32();
            let code = fields.next_i32();
            let message = fields.next_string().unwrap_or("");
            println!("ERROR [{}]: {} - {}", id, code, message);
        }
        Some(IncomingMessageId::AccountValue) => {
            let _version = fields.next_i32();
            let key = fields.next_string().unwrap_or("");
            let value = fields.next_string().unwrap_or("");
            let currency = fields.next_string().unwrap_or("");
            let account = fields.next_string().unwrap_or("");
            println!(
                "ACCT_VALUE: {} = {} {} ({})",
                key, value, currency, account
            );
        }
        Some(IncomingMessageId::PortfolioValue) => {
            let _version = fields.next_i32();
            let con_id = fields.next_i32();
            let symbol = fields.next_string().unwrap_or("");
            // Skip other contract fields for now
            fields.skip(9);
            let position = fields.next_f64();
            let market_price = fields.next_f64();
            let market_value = fields.next_f64();
            println!(
                "PORTFOLIO: {} (conId={}) pos={} @ {:.2} = {:.2}",
                symbol, con_id, position, market_price, market_value
            );
        }
        Some(IncomingMessageId::AccountDownloadEnd) => {
            let _version = fields.next_i32();
            let account = fields.next_string().unwrap_or("");
            println!("ACCOUNT_DOWNLOAD_END: {}", account);
        }
        Some(IncomingMessageId::NextValidId) => {
            let _version = fields.next_i32();
            let order_id = fields.next_i32();
            println!("NEXT_VALID_ID: {}", order_id);
        }
        Some(IncomingMessageId::ManagedAccounts) => {
            let _version = fields.next_i32();
            let accounts = fields.next_string().unwrap_or("");
            println!("MANAGED_ACCOUNTS: {}", accounts);
        }
        Some(IncomingMessageId::HistoricalData) => {
            let req_id = fields.next_i32();
            // For server version < 196, startDateStr and endDateStr come before itemCount
            let start_date = fields.next_string().unwrap_or("");
            let end_date = fields.next_string().unwrap_or("");
            let bar_count = fields.next_i32();
            println!(
                "HISTORICAL_DATA (reqId={}): {} bars ({} to {})",
                req_id, bar_count, start_date, end_date
            );
            for _ in 0..bar_count {
                if let Some(bar) = BarData::parse(&mut fields) {
                    println!(
                        "  {} O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.0}",
                        bar.date, bar.open, bar.high, bar.low, bar.close, bar.volume
                    );
                }
            }
        }
        Some(IncomingMessageId::HistoricalDataEnd) => {
            let req_id = fields.next_i32();
            let start = fields.next_string().unwrap_or("");
            let end = fields.next_string().unwrap_or("");
            println!("HISTORICAL_DATA_END (reqId={}): {} to {}", req_id, start, end);
        }
        Some(IncomingMessageId::HistoricalDataUpdate) => {
            let req_id = fields.next_i32();
            if let Some(bar) = BarData::parse(&mut fields) {
                println!(
                    "HISTORICAL_DATA_UPDATE (reqId={}): {} C:{:.2}",
                    req_id, bar.date, bar.close
                );
            }
        }
        None => {
            println!("MSG[{}]: {:?}", msg_id, fields.remaining());
        }
    }
}
