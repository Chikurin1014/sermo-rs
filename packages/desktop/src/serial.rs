use async_trait::async_trait;
use project_core::{
    Direction, Error, Message, PortConfig, PortInfo, PortType, Result, SerialPort, SerialPortConfig,
};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Desktop implementation of SerialPort using the serialport crate
#[derive(Debug, Clone)]
pub struct DesktopSerialPort {
    port: Arc<Mutex<Option<Box<dyn serialport::SerialPort>>>>,
    info: PortInfo,
    config: PortConfig,
}

impl PartialEq for DesktopSerialPort {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info && self.config == other.config
    }
}

impl DesktopSerialPort {
    pub fn new(info: PortInfo, config: PortConfig) -> Self {
        Self {
            port: Arc::new(Mutex::new(None)),
            info,
            config,
        }
    }
}

impl SerialPortConfig for DesktopSerialPort {
    fn with_port(mut self, port: String) -> Self {
        self.info.port = port;
        self
    }

    fn with_config(mut self, config: PortConfig) -> Self {
        self.config = config;
        self
    }
}

#[async_trait(?Send)]
impl SerialPort for DesktopSerialPort {
    async fn request_port(info: PortInfo, config: PortConfig) -> Result<Self> {
        Ok(DesktopSerialPort::new(info, config))
    }

    async fn open(&mut self) -> Result<()> {
        let port = serialport::new(&self.info.port, self.config.baud_rate)
            .data_bits(match self.config.data_bits {
                5 => serialport::DataBits::Five,
                6 => serialport::DataBits::Six,
                7 => serialport::DataBits::Seven,
                8 => serialport::DataBits::Eight,
                _ => return Err(Error::ConfigError("Invalid data bits".to_string())),
            })
            .stop_bits(match self.config.stop_bits {
                1 => serialport::StopBits::One,
                2 => serialport::StopBits::Two,
                _ => return Err(Error::ConfigError("Invalid stop bits".to_string())),
            })
            .open()
            .map_err(|e| Error::OpenError(e.to_string()))?;

        *self.port.lock().unwrap() = Some(port);
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        *self.port.lock().unwrap() = None;
        Ok(())
    }

    async fn read(&mut self) -> Result<Message> {
        let mut port_lock = self.port.lock().unwrap();
        let port = port_lock
            .as_mut()
            .ok_or_else(|| Error::ReadError("Port not open".to_string()))?;

        // Read available bytes into a buffer (non-blocking simple approach)
        let mut buf = vec![0u8; 1024];
        let read = port
            .read(&mut buf)
            .map_err(|e| Error::ReadError(e.to_string()))?;
        buf.truncate(read);

        let text = String::from_utf8_lossy(&buf).to_string();

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Ok(Message::new(
            project_core::data::Timestamp(ts),
            Direction::In,
            text,
        ))
    }

    async fn write(&mut self, message: Message) -> Result<()> {
        let mut port_lock = self.port.lock().unwrap();
        let port = port_lock
            .as_mut()
            .ok_or_else(|| Error::WriteError("Port not open".to_string()))?;

        let data = message.text().as_bytes();
        port.write(data)
            .map(|_| ())
            .map_err(|e| Error::WriteError(e.to_string()))
    }

    async fn flush(&mut self) -> Result<()> {
        let mut port_lock = self.port.lock().unwrap();
        if let Some(port) = port_lock.as_mut() {
            port.flush().map_err(|e| Error::WriteError(e.to_string()))?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.port.lock().unwrap().is_some()
    }

    async fn config(&self) -> &PortConfig {
        &self.config
    }

    async fn info(&self) -> &PortInfo {
        &self.info
    }

    async fn list_ports() -> Result<Vec<PortInfo>> {
        let ports =
            serialport::available_ports().map_err(|e| Error::DeviceNotFound(e.to_string()))?;

        Ok(ports
            .into_iter()
            .map(|p| {
                // Map platform port type into our PortType enum when possible.
                let port_type = match p.port_type {
                    serialport::SerialPortType::BluetoothPort => PortType::Bluetooth,
                    serialport::SerialPortType::PciPort => PortType::Pci,
                    serialport::SerialPortType::UsbPort(_) => {
                        // We don't attempt to pull out structured USB fields here to
                        // avoid fragile platform-specific field access; store a
                        // textual representation for now.
                        PortType::Other(format!("{:?}", p.port_type))
                    }
                    _ => PortType::Other(format!("{:?}", p.port_type)),
                };

                let mut info = PortInfo::new(p.port_name.clone(), port_type);
                info.description = None;
                info
            })
            .collect())
    }
}
