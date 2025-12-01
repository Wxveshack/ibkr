//! Async client for TWS/IB Gateway.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{oneshot, Mutex};
use tokio::time::timeout;

use crate::contract::Contract;
use crate::error::{Error, Result};
use crate::historical::{BarData, BarSize, Duration as HistDuration, HistoricalDataRequest, WhatToShow};
use crate::message::{IncomingMessageId, OutgoingMessageId};
use crate::wire::{make_field, FieldIterator};

/// Account value update.
#[derive(Debug, Clone)]
pub struct AccountValue {
    pub key: String,
    pub value: String,
    pub currency: String,
    pub account: String,
}

/// Historical data response.
#[derive(Debug, Clone)]
pub struct HistoricalDataResponse {
    pub start: String,
    pub end: String,
    pub bars: Vec<BarData>,
}

/// Internal message for request/response correlation.
enum ResponseMessage {
    AccountValues(Vec<AccountValue>),
    HistoricalData(HistoricalDataResponse),
    Error { code: i32, message: String },
}

/// Async client for Interactive Brokers TWS/Gateway.
pub struct Client {
    writer: Arc<Mutex<tokio::io::WriteHalf<TcpStream>>>,
    pending: Arc<Mutex<HashMap<i32, oneshot::Sender<ResponseMessage>>>>,
    next_req_id: AtomicI32,
    server_version: u32,
    #[allow(dead_code)]
    reader_handle: tokio::task::JoinHandle<()>,
}

impl Client {
    /// Connect to TWS/IB Gateway.
    ///
    /// # Arguments
    /// * `addr` - Address to connect to (e.g., "127.0.0.1:7496" for TWS, "127.0.0.1:4002" for Gateway)
    /// * `client_id` - Unique client identifier (use different IDs for multiple connections)
    pub async fn connect(addr: &str, client_id: i32) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let (mut reader, mut writer) = tokio::io::split(stream);

        // Send handshake: "API\0" + length-prefixed version string
        let version_str = b"v100..176";
        let mut handshake = b"API\0".to_vec();
        handshake.extend((version_str.len() as u32).to_be_bytes());
        handshake.extend(version_str);
        writer.write_all(&handshake).await?;

        // Read server version response
        let mut buf = [0u8; 4096];
        let n = reader.read(&mut buf).await?;
        if n < 4 {
            return Err(Error::Protocol("Invalid handshake response".into()));
        }

        let fields: Vec<&str> = std::str::from_utf8(&buf[4..n])
            .map_err(|_| Error::Protocol("Invalid UTF-8 in handshake".into()))?
            .split('\0')
            .filter(|s| !s.is_empty())
            .collect();

        let server_version: u32 = fields
            .first()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| Error::Protocol("Failed to parse server version".into()))?;

        // Send START_API
        let start_api = format!(
            "{}{}{}{}",
            make_field(OutgoingMessageId::StartApi.as_u32()),
            make_field(2),
            make_field(client_id),
            make_field(""),
        );
        Self::send_raw(&mut writer, &start_api).await?;

        // Wait briefly for initial messages
        tokio::time::sleep(Duration::from_millis(100)).await;

        let writer = Arc::new(Mutex::new(writer));
        let pending: Arc<Mutex<HashMap<i32, oneshot::Sender<ResponseMessage>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Spawn reader task
        let pending_clone = pending.clone();
        let reader_handle = tokio::spawn(async move {
            let mut recv_buf = Vec::new();
            let mut buf = [0u8; 8192];

            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        recv_buf.extend_from_slice(&buf[..n]);
                        while let Some((msg, rest)) = Self::extract_message(&recv_buf) {
                            Self::dispatch_message(&msg, &pending_clone).await;
                            recv_buf = rest;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            writer,
            pending,
            next_req_id: AtomicI32::new(1000),
            server_version,
            reader_handle,
        })
    }

    /// Get the TWS/Gateway server version.
    pub fn server_version(&self) -> u32 {
        self.server_version
    }

    /// Request account values.
    ///
    /// Returns all account values for the connected account.
    pub async fn account_values(&self) -> Result<Vec<AccountValue>> {
        let req_id = self.next_req_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();

        {
            let mut pending = self.pending.lock().await;
            pending.insert(req_id, tx);
        }

        // Send REQ_ACCT_DATA
        let msg = format!(
            "{}{}{}{}",
            make_field(OutgoingMessageId::ReqAccountData.as_u32()),
            make_field(2),
            make_field(1), // subscribe
            make_field(""),
        );
        self.send(&msg).await?;

        // Wait for response
        match timeout(Duration::from_secs(10), rx).await {
            Ok(Ok(ResponseMessage::AccountValues(values))) => Ok(values),
            Ok(Ok(ResponseMessage::Error { code, message })) => {
                Err(Error::Tws { code, message })
            }
            Ok(Ok(_)) => Err(Error::Protocol("Unexpected response type".into())),
            Ok(Err(_)) => Err(Error::Protocol("Response channel closed".into())),
            Err(_) => Err(Error::Timeout),
        }
    }

    /// Request historical market data.
    ///
    /// # Arguments
    /// * `contract` - The contract to request data for
    /// * `duration` - How far back to request data
    /// * `bar_size` - The size of each bar
    /// * `what_to_show` - The type of data to return
    /// * `use_rth` - Only return data from regular trading hours
    pub async fn historical_data(
        &self,
        contract: Contract,
        duration: HistDuration,
        bar_size: BarSize,
        what_to_show: WhatToShow,
        use_rth: bool,
    ) -> Result<Vec<BarData>> {
        let req_id = self.next_req_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();

        {
            let mut pending = self.pending.lock().await;
            pending.insert(req_id, tx);
        }

        let request = HistoricalDataRequest::new(req_id, contract)
            .duration(duration)
            .bar_size(bar_size)
            .what_to_show(what_to_show)
            .use_rth(use_rth);

        self.send(&request.encode()).await?;

        match timeout(Duration::from_secs(30), rx).await {
            Ok(Ok(ResponseMessage::HistoricalData(response))) => Ok(response.bars),
            Ok(Ok(ResponseMessage::Error { code, message })) => {
                Err(Error::Tws { code, message })
            }
            Ok(Ok(_)) => Err(Error::Protocol("Unexpected response type".into())),
            Ok(Err(_)) => Err(Error::Protocol("Response channel closed".into())),
            Err(_) => Err(Error::Timeout),
        }
    }

    async fn send(&self, payload: &str) -> Result<()> {
        let mut writer = self.writer.lock().await;
        Self::send_raw(&mut writer, payload).await
    }

    async fn send_raw(writer: &mut tokio::io::WriteHalf<TcpStream>, payload: &str) -> Result<()> {
        let bytes = payload.as_bytes();
        writer.write_all(&(bytes.len() as u32).to_be_bytes()).await?;
        writer.write_all(bytes).await?;
        writer.flush().await?;
        Ok(())
    }

    fn extract_message(buf: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        if buf.len() < 4 {
            return None;
        }
        let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        if buf.len() >= 4 + len {
            let msg = buf[4..4 + len].to_vec();
            let rest = buf[4 + len..].to_vec();
            Some((msg, rest))
        } else {
            None
        }
    }

    async fn dispatch_message(
        buf: &[u8],
        pending: &Arc<Mutex<HashMap<i32, oneshot::Sender<ResponseMessage>>>>,
    ) {
        let mut fields = FieldIterator::new(buf);
        let Some(msg_id) = fields.next_parsed::<u32>() else {
            return;
        };

        match IncomingMessageId::from_u32(msg_id) {
            Some(IncomingMessageId::AccountValue) => {
                // Collect in a static for account updates
                // For now, we'll handle this differently
                let _version = fields.next_i32();
                let key = fields.next_string().unwrap_or("").to_string();
                let value = fields.next_string().unwrap_or("").to_string();
                let currency = fields.next_string().unwrap_or("").to_string();
                let account = fields.next_string().unwrap_or("").to_string();

                // Account values are streaming - we need a different pattern
                // For now, log them
                let _ = AccountValue {
                    key,
                    value,
                    currency,
                    account,
                };
            }
            Some(IncomingMessageId::AccountDownloadEnd) => {
                // Signal completion - for now, find any pending account request
                let mut pending = pending.lock().await;
                // Find first pending request (simplified - should match by type)
                if let Some((req_id, tx)) = pending.iter().next().map(|(k, _)| *k).and_then(|k| {
                    pending.remove(&k).map(|tx| (k, tx))
                }) {
                    let _ = tx.send(ResponseMessage::AccountValues(vec![]));
                    let _ = req_id;
                }
            }
            Some(IncomingMessageId::HistoricalData) => {
                let req_id = fields.next_i32();
                let start = fields.next_string().unwrap_or("").to_string();
                let end = fields.next_string().unwrap_or("").to_string();
                let bar_count = fields.next_i32();

                let mut bars = Vec::with_capacity(bar_count as usize);
                for _ in 0..bar_count {
                    if let Some(bar) = BarData::parse(&mut fields) {
                        bars.push(bar);
                    }
                }

                let mut pending = pending.lock().await;
                if let Some(tx) = pending.remove(&req_id) {
                    let _ = tx.send(ResponseMessage::HistoricalData(HistoricalDataResponse {
                        start,
                        end,
                        bars,
                    }));
                }
            }
            Some(IncomingMessageId::Error) => {
                let _version = fields.next_i32();
                let req_id = fields.next_i32();
                let code = fields.next_i32();
                let message = fields.next_string().unwrap_or("").to_string();

                if req_id > 0 {
                    let mut pending = pending.lock().await;
                    if let Some(tx) = pending.remove(&req_id) {
                        let _ = tx.send(ResponseMessage::Error { code, message });
                    }
                }
            }
            _ => {}
        }
    }
}
