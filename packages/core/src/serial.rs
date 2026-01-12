use crate::Result;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Serial port information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    /// Port name (e.g., "COM1", "/dev/ttyUSB0")
    pub port: String,
    /// Baud rate
    pub baud_rate: u32,
    /// Data bits (typically 5-8)
    pub data_bits: u8,
    /// Number of stop bits
    pub stop_bits: u8,
}

impl PortInfo {
    pub fn new(port: String, baud_rate: u32) -> Self {
        Self {
            port,
            baud_rate,
            data_bits: 8,
            stop_bits: 1,
        }
    }
}

impl Default for PortInfo {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud_rate: 9600,
            data_bits: 8,
            stop_bits: 1,
        }
    }
}

/// Trait for platform-agnostic serial port communication
#[async_trait]
pub trait SerialPort: Send + Sync {
    /// Open the serial port
    async fn open(&mut self) -> Result<()>;

    /// Close the serial port
    async fn close(&mut self) -> Result<()>;

    /// Read data from the serial port
    /// Returns the number of bytes read and the data buffer
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Write data to the serial port
    async fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Check if the port is open
    fn is_open(&self) -> bool;

    /// Get available ports on the system
    async fn list_ports() -> Result<Vec<PortInfo>>;
}
