use ibkr::{BarSize, Client, Contract, Duration, WhatToShow};

#[tokio::main]
async fn main() -> ibkr::Result<()> {
    println!("Connecting to TWS...");
    let client = Client::connect("127.0.0.1:7496", 1).await?;
    println!("Connected to TWS v{}", client.server_version());

    // Request historical data for AMZN
    println!("\nRequesting AMZN daily bars for the past week...");
    let contract = Contract::stock("AMZN", "SMART", "USD");
    let bars = client
        .historical_data(
            contract,
            Duration::Weeks(1),
            BarSize::Day1,
            WhatToShow::Trades,
            true,
        )
        .await?;

    println!("Received {} bars:", bars.len());
    for bar in &bars {
        println!(
            "  {} O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.0}",
            bar.date, bar.open, bar.high, bar.low, bar.close, bar.volume
        );
    }

    Ok(())
}
