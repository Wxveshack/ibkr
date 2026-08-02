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

    let t = client.tickle().expect("tickle");
    eprintln!("[ok] tickle session id: {:?}", t.session);

    // /iserver data endpoints can 503 briefly while the brokerage backend warms up.
    let mut attempt = 0;
    let hits = loop {
        attempt += 1;
        match client.send(search::Request {
            symbol: "AAPL".into(),
            name: Some(false),
            sec_type: Some(search::SecType::Stk),
        }) {
            Ok(hits) => break hits,
            Err(ibkr::Error::Http(e))
                if e.status() == Some(reqwest::StatusCode::SERVICE_UNAVAILABLE) && attempt < 6 =>
            {
                eprintln!("[warn] 503 (backend warming up), retry {attempt}/5 in 2s…");
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
            Err(e) => panic!("secdef search: {e}"),
        }
    };

    println!("{} contracts:", hits.len());
    for c in hits.iter().take(5) {
        println!("  {:?}  conid={:?}  {:?}", c.symbol, c.conid, c.description);
    }
}
