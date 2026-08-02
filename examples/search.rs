//! OAuth end-to-end: mint a live session token, open the brokerage session, search a symbol.
//! Usage: `some-secret-source | cargo run --example search`

use std::io::Read;

use ibkr::iserver::secdef::search;
use ibkr::{Client, Credentials};

const SYMBOL: &str = "AMZN";

fn main() {
    let mut json = String::new();
    std::io::stdin()
        .read_to_string(&mut json)
        .expect("read credentials from stdin");
    let creds = Credentials::from_json(&json).expect("parse credentials JSON");

    // A consumer key registered today fails here until IBKR's nightly maintenance loads it.
    let lst = Client::mint(&creds).expect("mint live session token");
    let client = Client::oauth(&creds, &lst).expect("build oauth client");
    client.init_session().expect("init_session");
    client.tickle().expect("tickle");

    // /iserver data endpoints 503 briefly while the brokerage backend warms up.
    let mut attempt = 0;
    let hits = loop {
        attempt += 1;
        match client.send(search::Request {
            symbol: SYMBOL.into(),
            name: Some(false),
            sec_type: Some(search::SecType::Stk),
        }) {
            Ok(hits) => break hits,
            Err(ibkr::Error::Http(e))
                if e.status() == Some(reqwest::StatusCode::SERVICE_UNAVAILABLE) && attempt < 5 =>
            {
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Err(e) => panic!("secdef search: {e}"),
        }
    };

    // Best match first, then derivatives and unrelated tickers — only the head is interesting.
    println!("{} hits for {SYMBOL}:", hits.len());
    for c in hits.iter().take(5) {
        println!("  {:?}  conid={:?}", c.symbol, c.conid);
    }
}
