// One hardcoded request against the local IBKR Client Portal Web API gateway.
// conid 265598 = AAPL. Dumps the raw response. Change the conid to query another stock.
fn main() {
    let url = "https://localhost:5000/v1/api/iserver/marketdata/history\
               ?conid=265598&period=1w&bar=1d";

    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true) // gateway serves a self-signed cert
        .user_agent("ibkr/0.3") // gateway 403s requests with no User-Agent
        .build()
        .expect("build client");

    let resp = client.get(url).send().expect("request failed");
    println!("HTTP {}", resp.status());
    println!("{}", resp.text().expect("read body"));
}
