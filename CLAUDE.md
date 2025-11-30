# IBKR Rust Client

A Rust crate providing a clean, idiomatic interface to Interactive Brokers' TWS API.

## Mission

IBKR has the most comprehensive brokerage API (stocks, options, futures, forex, bonds, 150+ markets, lowest commissions) but their developer experience is stuck in 2001. We're building the bridge.

## The Protocol

IBKR uses a custom TCP protocol designed pre-JSON/protobuf era:

### Connection Flow
```
1. TCP connect to 127.0.0.1:7496 (TWS) or :4002 (IB Gateway)
2. Send: "API\0" + [4-byte len] + "v100..176"
3. Recv: [4-byte len] + "<server_version>\0<timestamp>\0"
4. Send: START_API (msgId=71) with clientId
5. Now ready for requests
```

### Message Format
```
[4-byte big-endian length][payload]

Payload = null-terminated string fields:
"<msgId>\0<field1>\0<field2>\0..."

Example REQ_ACCT_DATA: "6\02\01\0\0"
  - 6 = message ID (REQ_ACCT_DATA)
  - 2 = version
  - 1 = subscribe (true)
  - "" = account code (empty for single account)
```

### Key Message IDs
- OUT: REQ_MKT_DATA=1, PLACE_ORDER=3, CANCEL_ORDER=4, REQ_ACCT_DATA=6, START_API=71
- IN: TICK_PRICE=1, TICK_SIZE=2, ORDER_STATUS=3, ERR_MSG=4, ACCT_VALUE=6

### Version Negotiation
Client sends supported range (v100..176). Server responds with its version.
Message fields are conditionally included based on negotiated version.
Reference: twsapi_macunix/IBJts/source/pythonclient/ibapi/server_versions.py

## Architecture Principles

- KISS: Start minimal, extract patterns as they emerge
- No premature abstraction
- Test against real TWS before building more
- Async-first (tokio) for the final interface

## Reference Implementation

The Python reference code is in `twsapi_macunix/IBJts/source/pythonclient/ibapi/`:
- `client.py` (4891 lines) - All request methods, field construction
- `decoder.py` - Response parsing and dispatch
- `comm.py` - Message framing (read_msg, make_msg, make_field)
- `message.py` - Message ID constants (IN/OUT classes)
- `server_versions.py` - Version constants and compatibility

## Current State

`src/main.rs` - Proof of concept that connects, handshakes, and requests account data.

## Target API

```rust
let client = IbkrClient::connect("127.0.0.1:7496", client_id).await?;
let account = client.account_summary().await?;
println!("Net liquidation: {}", account.net_liquidation);
```

## Development

Requires TWS or IB Gateway running with API enabled:
- TWS: Configure > API > Enable ActiveX and Socket Clients
- Default port: 7496 (TWS) or 4002 (Gateway)
