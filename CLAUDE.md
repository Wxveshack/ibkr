# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build          # Build the library
cargo test           # Run all tests
cargo run            # Run example binary (requires TWS running)
cargo clippy         # Lint
cargo test wire::    # Run tests in specific module
```

## What This Is

A Rust client for Interactive Brokers' TWS API. IBKR has the most comprehensive brokerage API (stocks, options, futures, forex, bonds, 150+ markets) but their wire protocol is pre-JSON/protobuf era. This crate provides a clean async interface.

**Currently implemented:** Account data, Historical market data

## Architecture

```
src/
├── client.rs      # Async Client - the main public interface
├── wire.rs        # Protocol framing: length-prefixed messages, null-terminated fields
├── message.rs     # Message ID enums (Incoming/Outgoing)
├── contract.rs    # Contract struct (what instrument to trade/query)
├── historical.rs  # Historical data types (BarSize, Duration, BarData)
├── error.rs       # Error types
└── lib.rs         # Public exports
```

**Key pattern:** Request/response correlation via `req_id`. Client sends request with ID, stores a oneshot channel, reader task dispatches response to correct channel.

## The Wire Protocol

IBKR uses a custom TCP protocol:
- Messages are length-prefixed (4-byte big-endian) with null-terminated string fields
- Version negotiation: client sends "v100..176", server responds with its version
- Field inclusion is conditional on negotiated server version (see `server_versions.py`)

```
[4-byte length][msgId\0field1\0field2\0...]
```

## Reference Implementation

The official Python client is in `twsapi_macunix/IBJts/source/pythonclient/ibapi/`:
- `client.py` - Request methods and field construction
- `decoder.py` - Response parsing (check field order here)
- `message.py` - Message ID constants
- `server_versions.py` - Version-dependent field inclusion

**When adding new endpoints:** Always cross-reference the Python `client.py` for request encoding and `decoder.py` for response parsing. Field order and version checks matter.

## Development Requirements

Requires TWS or IB Gateway running with API enabled:
- TWS: Configure > API > Enable ActiveX and Socket Clients
- Port 7496 (TWS) or 4002 (Gateway)
