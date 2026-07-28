//! Minimal OAuth smoke test: mint LST, open the session, run a secdef search.
//! Usage: `some-secret-source | cargo run --bin search`

use std::io::Read;

use ibkr::iserver::secdef::search;
use ibkr::{Client, Credentials};

fn main() {
    let mut json = String::new();
    std::io::stdin().read_to_string(&mut json).expect("read credentials from stdin");
    let creds = Credentials::from_json(&json).expect("parse credentials JSON");

    let lst = Client::mint(&creds).expect("mint live session token");
    eprintln!("[ok] live session token minted (expires {})", lst.expiration);

    let client = Client::oauth(&creds, &lst).expect("build oauth client");
    let session = client.init_session().expect("init_session");
    eprintln!(
        "[ok] session: authenticated={:?} connected={:?} competing={:?}",
        session.authenticated, session.connected, session.competing
    );

    let hits = client
        .send(search::Request {
            symbol: "AAPL".into(),
            name: Some(false),
            sec_type: Some(search::SecType::Stk),
        })
        .expect("secdef search");

    println!("{} contracts:", hits.len());
    for c in hits.iter().take(5) {
        println!("  {:?}  conid={:?}  {:?}", c.symbol, c.conid, c.description);
    }
}
