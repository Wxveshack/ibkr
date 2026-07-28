//! Mint a live session token from a JSON credential payload, then open a session.
//! Usage: `mint_lst creds.json`  or  `some-secret-source | mint_lst` (reads stdin).

use std::io::Read;

use ibkr::{Client, Credentials};

fn main() {
    let json = match std::env::args().nth(1) {
        Some(path) => std::fs::read_to_string(&path).expect("read credentials file"),
        None => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .expect("read credentials from stdin");
            buf
        }
    };

    let creds = Credentials::from_json(&json).expect("parse credentials JSON");

    let lst = Client::mint(&creds).expect("mint live session token");
    println!("live session token: {}", lst.token);
    println!("expires (epoch ms): {}", lst.expiration);

    // New keys can take ~24h to propagate; a failure here right after setup is usually that.
    let client = Client::oauth(&creds, &lst).expect("build oauth client");
    match client.init_session() {
        Ok(s) => println!("session: authenticated={:?} connected={:?}", s.authenticated, s.connected),
        Err(e) => eprintln!("init_session failed (often key propagation, ~24h): {e}"),
    }
    match client.tickle() {
        Ok(t) => println!("tickle session id: {:?}", t.session),
        Err(e) => eprintln!("tickle failed: {e}"),
    }
}
