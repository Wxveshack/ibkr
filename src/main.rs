//! Demo: fetch one week of daily bars for AAPL (conid 265598) over the local gateway.

use ibkr::iserver::marketdata::history;
use ibkr::Client;

fn main() {
    let client = Client::new();

    let hist = client
        .send(history::Request {
            conid: 265598,
            bar: history::BarSize::Day1,
            period: Some("1w".to_string()),
            exchange: None,
            start_time: None,
            outside_rth: None,
            source: None,
        })
        .expect("history request failed");

    println!("{:?}: {} bars", hist.symbol, hist.data.len());
    for bar in &hist.data {
        println!("  t={} o={} h={} l={} c={} v={}", bar.t, bar.o, bar.h, bar.l, bar.c, bar.v);
    }
}
