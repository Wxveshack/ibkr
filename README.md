# ibkr

A Rust client for Interactive Brokers' TWS API.

## Usage

```rust
use ibkr::{Client, Contract, BarSize, Duration, WhatToShow};

#[tokio::main]
async fn main() -> ibkr::Result<()> {
    let client = Client::connect("127.0.0.1:7496", 1).await?;

    let contract = Contract::stock("AAPL", "SMART", "USD");
    let bars = client.historical_data(
        contract,
        Duration::Days(5),
        BarSize::Day1,
        WhatToShow::Trades,
        true,
    ).await?;

    for bar in bars {
        println!("{}: {}", bar.date, bar.close);
    }
    Ok(())
}
```

## Architecture

See [CLAUDE.md](CLAUDE.md).

## IBKR Documentation

https://www.interactivebrokers.com/campus/ibkr-api-page/twsapi-doc/#api-introduction
