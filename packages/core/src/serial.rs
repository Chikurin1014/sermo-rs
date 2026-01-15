use crate::data::Message;
use crate::error::Result;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Port type information inspired by `serialport::SerialPortType`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PortType {
    Usb {
        vendor_id: Option<u16>,
        product_id: Option<u16>,
        manufacturer: Option<String>,
        product: Option<String>,
        serial_number: Option<String>,
    },
    Bluetooth,
    Pci,
    /// Unknown or other port type. The contained String may be a debug
    /// representation from the platform crate.
    Other(String),
}

/// Static information about a physical port returned by `list_ports()`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortInfo {
    /// Port name (e.g., "COM1", "/dev/ttyUSB0")
    pub port: String,
    /// Port type (USB/Bluetooth/etc)
    pub port_type: PortType,
    /// Optional human-friendly description or manufacturer string
    pub description: Option<String>,
}

impl PortInfo {
    pub fn new(port: String, port_type: PortType) -> Self {
        Self {
            port,
            port_type,
            description: None,
        }
    }
}

impl Default for PortInfo {
    fn default() -> Self {
        Self::new(String::new(), PortType::Other(String::new()))
    }
}

/// Configuration used to open a port (baud rate, data bits, stop bits, ...)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PortConfig {
    pub baud_rate: u32,
    pub data_bits: u8,
    pub stop_bits: u8,
}

impl PortConfig {
    pub fn new(baud_rate: u32, data_bits: u8, stop_bits: u8) -> Self {
        Self {
            baud_rate,
            data_bits,
            stop_bits,
        }
    }
}

impl Default for PortConfig {
    fn default() -> Self {
        Self::new(9600, 8, 1)
    }
}

pub trait SerialPortConfig {
    /// Set the port name (where applicable)
    fn with_port(self, port: String) -> Self;

    /// Replace the entire port configuration
    fn with_config(self, config: PortConfig) -> Self;
}

/// Trait for platform-agnostic serial port communication
#[async_trait(?Send)]
pub trait SerialPort: SerialPortConfig + Sized + PartialEq {
    async fn request_port(info: PortInfo, config: PortConfig) -> Result<Self>;

    /// Open the serial port
    async fn open(&mut self) -> Result<()>;

    /// Close the serial port
    async fn close(&mut self) -> Result<()>;

    /// Read data from the serial port
    /// Returns the number of bytes read and the data buffer
    async fn read(&mut self) -> Result<Message>;

    /// Write data to the serial port
    async fn write(&mut self, message: Message) -> Result<()>;

    /// Flush the serial port buffers
    async fn flush(&mut self) -> Result<()>;

    /// Port configuration
    async fn config(&self) -> &PortConfig;

    /// Port info
    async fn info(&self) -> &PortInfo;

    /// Check if the port is open
    fn is_open(&self) -> bool;

    /// Get available ports on the system
    async fn list_ports() -> Result<Vec<PortInfo>>;
}
