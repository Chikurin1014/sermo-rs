use async_trait::async_trait;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys;

use js_sys::{Function, Reflect};
use project_core::{PortConfig, PortInfo, PortType, Result as CoreResult};

/// Simple WebSerial that owns an optional `web_sys::SerialPort` instance and
/// implements the `project_core::SerialPort` trait.
pub struct WebSerialPort {
    port: Option<web_sys::SerialPort>,
    info: PortInfo,
    config: PortConfig,
}

impl WebSerialPort {
    /// Create a new WebSerial with the given info and config. The `port` will
    /// be `None` until the user selects one via `request_port` or it's set by
    /// the caller.
    pub fn new(info: PortInfo, config: PortConfig) -> Self {
        Self {
            port: None,
            info,
            config,
        }
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
            info: PortInfo::default(),
            config: PortConfig::default(),
        })
    }
}

impl Default for WebSerialPort {
    fn default() -> Self {
        Self::new(PortInfo::default(), PortConfig::default())
    }
}

impl project_core::SerialPortConfig for WebSerialPort {
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
    async fn read(&mut self) -> project_core::Result<project_core::Message> {
        // Reading from web serial requires handling ReadableStream; leave as
        // not implemented for now and return an error.
        Err(project_core::Error::ReadError(
            "Read not implemented for web".to_string(),
        ))
    }

    async fn write(&mut self, _message: project_core::Message) -> project_core::Result<()> {
        // Writing requires handling WritableStream; not implemented here.
        Err(project_core::Error::WriteError(
            "Write not implemented for web".to_string(),
        ))
    }

    fn is_open(&self) -> bool {
        self.port.is_some()
    }

    async fn flush(&mut self) -> project_core::Result<()> {
        // Web Serial doesn't expose a flush primitive; noop for now.
        Ok(())
    }

    async fn config(&self) -> &project_core::PortConfig {
        &self.config
    }

    async fn info(&self) -> &project_core::PortInfo {
        &self.info
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
            let js_port = ports.get(i);
            if js_port.is_undefined() || js_port.is_null() {
                continue;
            }

            if js_port.dyn_ref::<web_sys::SerialPort>().is_some() {
                // Default basic info
                let mut info = PortInfo::new(
                    format!("Serial Port {}", i),
                    PortType::Other("web_serial".to_string()),
                );

                // Try to call `getInfo()` on the SerialPort (if the browser exposes it).
                // We call the JS function via Reflect to avoid depending on a specific
                // web-sys binding being present.
                if let Ok(get_info_val) = Reflect::get(&js_port, &JsValue::from_str("getInfo")) {
                    if get_info_val.is_function() {
                        let func: Function = get_info_val.unchecked_into();
                        match func.call0(&js_port) {
                            Ok(promise_val) => {
                                if let Ok(promise) = promise_val.dyn_into::<js_sys::Promise>() {
                                    match JsFuture::from(promise).await {
                                        Ok(info_js) => {
                                            // Try to extract USB vendor/product ids and other fields
                                            let vendor = Reflect::get(
                                                &info_js,
                                                &JsValue::from_str("usbVendorId"),
                                            )
                                            .ok()
                                            .and_then(|v| v.as_f64().map(|n| n as u16));
                                            let product = Reflect::get(
                                                &info_js,
                                                &JsValue::from_str("usbProductId"),
                                            )
                                            .ok()
                                            .and_then(|v| v.as_f64().map(|n| n as u16));

                                            let manufacturer = Reflect::get(
                                                &info_js,
                                                &JsValue::from_str("manufacturer"),
                                            )
                                            .ok()
                                            .and_then(|v| v.as_string());
                                            let product_name = Reflect::get(
                                                &info_js,
                                                &JsValue::from_str("productName"),
                                            )
                                            .ok()
                                            .and_then(|v| v.as_string());
                                            let serial_number = Reflect::get(
                                                &info_js,
                                                &JsValue::from_str("serialNumber"),
                                            )
                                            .ok()
                                            .and_then(|v| v.as_string());

                                            if vendor.is_some() || product.is_some() {
                                                info.port_type = PortType::Usb {
                                                    vendor_id: vendor,
                                                    product_id: product,
                                                    manufacturer: manufacturer.clone(),
                                                    product: product_name.clone(),
                                                    serial_number: serial_number.clone(),
                                                };
                                            }

                                            info.description = None;
                                        }
                                        Err(_) => {
                                            // ignore failures to obtain info, fallback to Other
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // calling getInfo threw; ignore and fall back
                            }
                        }
                    }
                }

                result.push(info);
            }
        }

        Ok(result)
    }
}
