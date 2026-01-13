use async_trait::async_trait;
use project_core::{Error, PortInfo, Result, SerialPort, SerialPortConfig};
use std::sync::{Arc, Mutex};

/// Desktop implementation of SerialPort using the serialport crate
#[derive(Debug)]
pub struct DesktopSerialPort {
    port: Arc<Mutex<Option<Box<dyn serialport::SerialPort>>>>,
    config: PortInfo,
}

impl DesktopSerialPort {
    pub fn new(config: PortInfo) -> Self {
        Self {
            port: Arc::new(Mutex::new(None)),
            config,
        }
    }
}

impl Default for DesktopSerialPort {
    fn default() -> Self {
        Self::new(PortInfo::default())
    }
}

impl SerialPortConfig for DesktopSerialPort {
    fn with_port(mut self, port: String) -> Self {
        self.config = self.config.with_port(port);
        self
    }

    fn with_baud_rate(mut self, baud_rate: u32) -> Self {
        self.config = self.config.with_baud_rate(baud_rate);
        self
    }

    fn with_data_bits(mut self, data_bits: u8) -> Self {
        self.config = self.config.with_data_bits(data_bits);
        self
    }

    fn with_stop_bits(mut self, stop_bits: u8) -> Self {
        self.config = self.config.with_stop_bits(stop_bits);
        self
    }
}

#[async_trait]
impl SerialPort for DesktopSerialPort {
    async fn open(&mut self) -> Result<()> {
        let port = serialport::new(&self.config.port, self.config.baud_rate)
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

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut port_lock = self.port.lock().unwrap();
        let port = port_lock
            .as_mut()
            .ok_or_else(|| Error::ReadError("Port not open".to_string()))?;

        port.read(buf).map_err(|e| Error::ReadError(e.to_string()))
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut port_lock = self.port.lock().unwrap();
        let port = port_lock
            .as_mut()
            .ok_or_else(|| Error::WriteError("Port not open".to_string()))?;

        port.write(buf)
            .map_err(|e| Error::WriteError(e.to_string()))
    }

    fn is_open(&self) -> bool {
        self.port.lock().unwrap().is_some()
    }

    async fn list_ports() -> Result<Vec<PortInfo>> {
        let ports =
            serialport::available_ports().map_err(|e| Error::DeviceNotFound(e.to_string()))?;

        Ok(ports
            .into_iter()
            .map(|p| PortInfo::new(p.port_name, 9600, 8, 1))
            .collect())
    }
}
