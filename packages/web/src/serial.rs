use async_trait::async_trait;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys;

use project_core::{PortInfo, Result as CoreResult};

/// Simple WebSerial that owns an optional `web_sys::SerialPort` instance and
/// implements the `project_core::SerialPort` trait.
pub struct WebSerialPort {
    port: Option<web_sys::SerialPort>,
    config: PortInfo,
}

impl WebSerialPort {
    /// Create a new WebSerial with the given config. The `port` will be `None`
    /// until the user selects one via `request_port` or it's set by the caller.
    pub fn new(config: PortInfo) -> Self {
        Self { port: None, config }
    }

    /// Request a port from the user and return a `WebSerialPort` owning that port.
    pub async fn request_port() -> CoreResult<Self> {
        let window = web_sys::window()
            .ok_or_else(|| project_core::Error::DeviceNotFound("No window object".to_string()))?;
        let navigator = window.navigator();
        let serial = navigator.serial();

        let promise = serial.request_port();
        let port_js = JsFuture::from(promise)
            .await
            .map_err(|_| project_core::Error::OpenError("User cancelled or error".to_string()))?;

        let port = port_js.dyn_into::<web_sys::SerialPort>().map_err(|_| {
            project_core::Error::OpenError("Failed to cast to SerialPort".to_string())
        })?;

        Ok(Self {
            port: Some(port),
            config: PortInfo::default(),
        })
    }
}

impl Default for WebSerialPort {
    fn default() -> Self {
        Self::new(PortInfo::default())
    }
}

impl project_core::SerialPortConfig for WebSerialPort {
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

#[async_trait(?Send)]
impl project_core::SerialPort for WebSerialPort {
    async fn open(&mut self) -> project_core::Result<()> {
        if let Some(port) = &self.port {
            let options = web_sys::SerialOptions::new(self.config.baud_rate);
            options.set_data_bits(self.config.data_bits);
            options.set_stop_bits(self.config.stop_bits);

            let promise = port.open(&options);
            JsFuture::from(promise)
                .await
                .map_err(|_| project_core::Error::OpenError("Failed to open port".to_string()))?;

            Ok(())
        } else {
            Err(project_core::Error::OpenError(
                "No port selected".to_string(),
            ))
        }
    }

    async fn close(&mut self) -> project_core::Result<()> {
        if let Some(port) = &self.port {
            let promise = port.close();
            JsFuture::from(promise)
                .await
                .map_err(|_| project_core::Error::OpenError("Failed to close port".to_string()))?;
            self.port = None;
        }
        Ok(())
    }

    async fn read(&mut self, _buf: &mut [u8]) -> project_core::Result<usize> {
        Err(project_core::Error::ReadError(
            "Read not implemented for web".to_string(),
        ))
    }

    async fn write(&mut self, _buf: &[u8]) -> project_core::Result<usize> {
        Err(project_core::Error::WriteError(
            "Write not implemented for web".to_string(),
        ))
    }

    fn is_open(&self) -> bool {
        self.port.is_some()
    }

    /// Return the list of previously permitted ports (does not prompt the user).
    async fn list_ports() -> project_core::Result<Vec<PortInfo>> {
        let window = web_sys::window()
            .ok_or_else(|| project_core::Error::DeviceNotFound("No window object".to_string()))?;
        let navigator = window.navigator();
        let serial = navigator.serial();

        let promise = serial.get_ports();

        let ports = JsFuture::from(promise)
            .await
            .map_err(|_| project_core::Error::DeviceNotFound("Failed to get ports".to_string()))?;

        let ports: js_sys::Array = ports.dyn_into().map_err(|_| {
            project_core::Error::DeviceNotFound("Failed to convert ports".to_string())
        })?;
        let mut result = Vec::new();
        for i in 0..ports.length() {
            if let Some(_port) = ports.get(i).dyn_ref::<web_sys::SerialPort>() {
                result.push(PortInfo::new(format!("Serial Port {}", i), 9600, 8, 1));
            }
        }

        Ok(result)
    }
}
