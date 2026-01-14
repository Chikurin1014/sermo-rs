use async_trait::async_trait;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys;

use js_sys::{Date, Function, Reflect, Uint8Array};
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
        use wasm_bindgen_futures::JsFuture;

        let port = match &self.port {
            Some(p) => p,
            None => return Err(project_core::Error::ReadError("No port open".to_string())),
        };

        // Use JS reflection to access the readable stream and its reader so
        // we don't depend on specific web-sys bindings which may vary.
        let port_js = JsValue::from(port.clone());

        let readable = Reflect::get(&port_js, &JsValue::from_str("readable")).map_err(|_| {
            project_core::Error::ReadError("Port has no readable stream".to_string())
        })?;
        if readable.is_undefined() || readable.is_null() {
            return Err(project_core::Error::ReadError(
                "Readable stream not available".to_string(),
            ));
        }

        // getReader()
        // let get_reader = match Reflect::get(&readable, &JsValue::from_str("getReader")) {
        //     Ok(v) => v,
        //     Err(_) => match Reflect::get(&readable, &JsValue::from_str("get_reader")) {
        //         Ok(v2) => v2,
        //         Err(_) => {
        //             return Err(project_core::Error::ReadError(
        //                 "Readable.getReader not available".to_string(),
        //             ))
        //         }
        //     },
        // };
        let get_reader = Reflect::get(&readable, &JsValue::from_str("getReader"))
            .or(Reflect::get(&readable, &JsValue::from_str("get_reader")))
            .map_err(|_| {
                project_core::Error::ReadError("Readable.getReader not available".to_string())
            })?;
        if !get_reader.is_function() {
            return Err(project_core::Error::ReadError(
                "Readable.getReader not available".to_string(),
            ));
        }
        let get_reader_fn: Function = get_reader.unchecked_into();

        let reader = get_reader_fn
            .call0(&readable)
            .map_err(|_| project_core::Error::ReadError("Failed to get reader".to_string()))?;

        // call reader.read() -> Promise
        let read_fn = Reflect::get(&reader, &JsValue::from_str("read"))
            .map_err(|_| project_core::Error::ReadError("Reader.read not available".to_string()))?;
        if !read_fn.is_function() {
            return Err(project_core::Error::ReadError(
                "Reader.read not a function".to_string(),
            ));
        }
        let read_fn: Function = read_fn.unchecked_into();

        let promise = read_fn
            .call0(&reader)
            .map_err(|_| project_core::Error::ReadError("Failed to call reader.read".to_string()))?
            .dyn_into::<js_sys::Promise>()
            .map_err(|_| {
                project_core::Error::ReadError("reader.read did not return a Promise".to_string())
            })?;
        let result = JsFuture::from(promise).await.map_err(|_| {
            project_core::Error::ReadError("reader.read promise failed".to_string())
        })?;

        // result.value is the Uint8Array (or undefined)
        let value = Reflect::get(&result, &JsValue::from_str("value")).unwrap_or(JsValue::NULL);
        if value.is_null() || value.is_undefined() {
            return Err(project_core::Error::ReadError(
                "No data available".to_string(),
            ));
        }

        let u8 = Uint8Array::new(&value);
        let mut vec = vec![0u8; u8.length() as usize];
        u8.copy_to(&mut vec[..]);

        let text = String::from_utf8_lossy(&vec).to_string();
        let timestamp = project_core::Timestamp(Date::now() as u64);
        Ok(project_core::Message::new(
            timestamp,
            project_core::Direction::In,
            text,
        ))
    }

    async fn write(&mut self, _message: project_core::Message) -> project_core::Result<()> {
        use wasm_bindgen_futures::JsFuture;

        let port = match &self.port {
            Some(p) => p,
            None => return Err(project_core::Error::WriteError("No port open".to_string())),
        };

        let port_js = JsValue::from(port.clone());

        let writable = Reflect::get(&port_js, &JsValue::from_str("writable")).map_err(|_| {
            project_core::Error::WriteError("Port has no writable stream".to_string())
        })?;
        if writable.is_undefined() || writable.is_null() {
            return Err(project_core::Error::WriteError(
                "Writable stream not available".to_string(),
            ));
        }

        let get_writer =
            Reflect::get(&writable, &JsValue::from_str("getWriter")).map_err(|_| {
                project_core::Error::WriteError("Writable.getWriter not available".to_string())
            })?;
        if !get_writer.is_function() {
            return Err(project_core::Error::WriteError(
                "Writable.getWriter not available".to_string(),
            ));
        }
        let get_writer_fn: Function = get_writer.unchecked_into();
        let writer = get_writer_fn.call0(&writable).map_err(|_| {
            project_core::Error::WriteError("Failed to call writable.getWriter".to_string())
        })?;

        // Convert message text to bytes
        let bytes = _message.text().as_bytes();
        let array = Uint8Array::from(bytes);

        // call writer.write(array)
        let write_fn = Reflect::get(&writer, &JsValue::from_str("write")).map_err(|_| {
            project_core::Error::WriteError("Writer.write not available".to_string())
        })?;
        if !write_fn.is_function() {
            return Err(project_core::Error::WriteError(
                "Writer.write not a function".to_string(),
            ));
        }
        let write_fn: Function = write_fn.unchecked_into();
        let promise = write_fn
            .call1(&writer, &JsValue::from(array))
            .map_err(|_| {
                project_core::Error::WriteError("Failed to call writer.write".to_string())
            })?;

        let promise = promise.dyn_into::<js_sys::Promise>().map_err(|_| {
            project_core::Error::WriteError("writer.write did not return a Promise".to_string())
        })?;

        JsFuture::from(promise).await.map_err(|_| {
            project_core::Error::WriteError("writer.write promise failed".to_string())
        })?;

        // Optionally release the writer lock if releaseLock exists
        if let Ok(release_fn) = Reflect::get(&writer, &JsValue::from_str("releaseLock")) {
            if release_fn.is_function() {
                let rel: Function = release_fn.unchecked_into();
                let _ = rel.call0(&writer);
            }
        }

        Ok(())
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
