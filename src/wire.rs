//! Wire protocol utilities for TWS message framing.
//!
//! TWS uses a simple length-prefixed protocol:
//! - Each message is prefixed with a 4-byte big-endian length
//! - Message payload consists of null-terminated string fields

use std::io::{self, Write};

/// Create a null-terminated field from a value.
pub fn make_field<T: std::fmt::Display>(value: T) -> String {
    format!("{}\0", value)
}

/// Build a complete message payload from fields.
///
/// Each field is converted to a null-terminated string.
pub fn make_message(fields: &[&dyn std::fmt::Display]) -> String {
    fields.iter().map(|f| make_field(f)).collect()
}

/// Write a length-prefixed message to a writer.
pub fn send_message<W: Write>(writer: &mut W, payload: &str) -> io::Result<()> {
    let bytes = payload.as_bytes();
    writer.write_all(&(bytes.len() as u32).to_be_bytes())?;
    writer.write_all(bytes)?;
    Ok(())
}

/// Extract a complete message from a buffer.
///
/// Returns `Some((message_bytes, remaining_buffer))` if a complete message
/// is available, or `None` if more data is needed.
pub fn extract_message(buf: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
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

/// Parse a message buffer into null-separated fields.
pub fn parse_fields(buf: &[u8]) -> Vec<&str> {
    std::str::from_utf8(buf)
        .unwrap_or("")
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect()
}

/// Iterator for parsing fields from a message.
///
/// Provides typed field extraction with position tracking.
pub struct FieldIterator<'a> {
    fields: Vec<&'a str>,
    position: usize,
}

impl<'a> FieldIterator<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        let fields = std::str::from_utf8(buf)
            .unwrap_or("")
            .split('\0')
            .filter(|s| !s.is_empty())
            .collect();
        Self { fields, position: 0 }
    }

    /// Get the next field as a string.
    pub fn next_string(&mut self) -> Option<&'a str> {
        if self.position < self.fields.len() {
            let field = self.fields[self.position];
            self.position += 1;
            Some(field)
        } else {
            None
        }
    }

    /// Get the next field parsed as the specified type.
    pub fn next<T: std::str::FromStr>(&mut self) -> Option<T> {
        self.next_string().and_then(|s| s.parse().ok())
    }

    /// Get the next field as i32, defaulting to 0 for empty/invalid.
    pub fn next_i32(&mut self) -> i32 {
        self.next().unwrap_or(0)
    }

    /// Get the next field as f64, defaulting to 0.0 for empty/invalid.
    pub fn next_f64(&mut self) -> f64 {
        self.next().unwrap_or(0.0)
    }

    /// Get the next field as bool (0 = false, anything else = true).
    pub fn next_bool(&mut self) -> bool {
        self.next_i32() != 0
    }

    /// Skip n fields.
    pub fn skip(&mut self, n: usize) {
        self.position = (self.position + n).min(self.fields.len());
    }

    /// Get remaining fields as a slice.
    pub fn remaining(&self) -> &[&'a str] {
        &self.fields[self.position..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_field() {
        assert_eq!(make_field("hello"), "hello\0");
        assert_eq!(make_field(42), "42\0");
        assert_eq!(make_field(3.14), "3.14\0");
    }

    #[test]
    fn test_extract_message() {
        // Build a test message: length prefix + payload
        let payload = b"6\0test\0";
        let mut buf = Vec::new();
        buf.extend(&(payload.len() as u32).to_be_bytes());
        buf.extend(payload);

        let (msg, rest) = extract_message(&buf).unwrap();
        assert_eq!(msg, payload);
        assert!(rest.is_empty());
    }

    #[test]
    fn test_extract_message_incomplete() {
        // Not enough data for length prefix
        assert!(extract_message(&[0, 0]).is_none());

        // Length says 10 bytes, but only 5 available
        let buf = [0, 0, 0, 10, 1, 2, 3, 4, 5];
        assert!(extract_message(&buf).is_none());
    }

    #[test]
    fn test_field_iterator() {
        let buf = b"17\0123\045.5\0hello\0";
        let mut iter = FieldIterator::new(buf);

        assert_eq!(iter.next::<u32>(), Some(17));
        assert_eq!(iter.next_i32(), 123);
        assert_eq!(iter.next_f64(), 45.5);
        assert_eq!(iter.next_string(), Some("hello"));
        assert_eq!(iter.next_string(), None);
    }
}
