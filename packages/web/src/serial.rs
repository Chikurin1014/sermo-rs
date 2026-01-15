mod helper;
mod request_port;

use async_trait::async_trait;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys;

use js_sys::{Date, Function, Reflect, Uint8Array};
use project_core::{PortConfig, PortInfo, Result as CoreResult};

use helper::js_port_to_port_info;

pub use request_port::RequestPort;

/// Simple WebSerial that owns an optional `web_sys::SerialPort` instance and
/// implements the `project_core::SerialPort` trait.
#[derive(Debug, Clone, PartialEq)]
pub struct WebSerialPort {
    port: web_sys::SerialPort,
    info: PortInfo,
    config: PortConfig,
    is_open: bool,
}

impl WebSerialPort {
    async fn request_port(config: PortConfig) -> CoreResult<Self> {
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

        // Try to extract richer `PortInfo` metadata from the selected port.
        let info = js_port_to_port_info(&port, "Selected Web Serial Port".to_string()).await;

        Ok(Self {
            port,
            info,
            config,
            is_open: false,
        })
    }

    async fn read(&mut self) -> CoreResult<Uint8Array> {
        use wasm_bindgen_futures::JsFuture;

        // Use JS reflection to access the readable stream and its reader so
        // we don't depend on specific web-sys bindings which may vary.
        let port_js = JsValue::from(self.port.clone());
        let readable = Reflect::get(&port_js, &JsValue::from_str("readable")).map_err(|_| {
            project_core::Error::ReadError("Port has no readable stream".to_string())
        })?;
        if readable.is_undefined() || readable.is_null() {
            return Err(project_core::Error::ReadError(
                "Readable stream not available".to_string(),
            ));
        }

        let get_reader: Function = Reflect::get(&readable, &JsValue::from_str("getReader"))
            .or(Reflect::get(&readable, &JsValue::from_str("get_reader")))
            .map_err(|_| {
                project_core::Error::ReadError("Readable.getReader not available".to_string())
            })?
            .try_into()
            .map_err(|_| {
                project_core::Error::ReadError("Readable.getReader not available".to_string())
            })?;
        let reader = get_reader
            .call0(&readable)
            .map_err(|_| project_core::Error::ReadError("Failed to get reader".to_string()))?;

        // call reader.read(&reader) -> Promise
        let read: Function = Reflect::get(&reader, &JsValue::from_str("read"))
            .map_err(|_| project_core::Error::ReadError("Reader.read not available".to_string()))?
            .try_into()
            .map_err(|_| project_core::Error::ReadError("Reader.read not available".to_string()))?;
        let promise = read
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
        let array = Uint8Array::new(&value);
        Ok(array)
    }

    async fn write(&mut self, array: &Uint8Array) -> CoreResult<()> {
        use wasm_bindgen_futures::JsFuture;

        let port_js = JsValue::from(self.port.clone());

        let writable = Reflect::get(&port_js, &JsValue::from_str("writable")).map_err(|_| {
            project_core::Error::WriteError("Port has no writable stream".to_string())
        })?;
        if writable.is_undefined() || writable.is_null() {
            return Err(project_core::Error::WriteError(
                "Writable stream not available".to_string(),
            ));
        }

        let get_writer: Function = Reflect::get(&writable, &JsValue::from_str("getWriter"))
            .map_err(|_| {
                project_core::Error::WriteError("Writable.getWriter not available".to_string())
            })?
            .try_into()
            .map_err(|_| {
                project_core::Error::WriteError("Writable.getWriter not available".to_string())
            })?;
        let writer = get_writer.call0(&writable).map_err(|_| {
            project_core::Error::WriteError("Failed to call writable.getWriter".to_string())
        })?;

        // call writer.write(array)
        let write: Function = Reflect::get(&writer, &JsValue::from_str("write"))
            .map_err(|_| project_core::Error::WriteError("Writer.write not available".to_string()))?
            .try_into()
            .map_err(|_| {
                project_core::Error::WriteError("Writer.write not available".to_string())
            })?;

        let promise = write
            .call1(&writer, &JsValue::from(array))
            .map_err(|_| {
                project_core::Error::WriteError("Failed to call writer.write".to_string())
            })?
            .dyn_into::<js_sys::Promise>()
            .map_err(|_| {
                project_core::Error::WriteError("writer.write did not return a Promise".to_string())
            })?;
        JsFuture::from(promise).await.map_err(|_| {
            project_core::Error::WriteError("writer.write promise failed".to_string())
        })?;

        // Optionally release the writer lock if releaseLock exists
        if let Ok(release) = Reflect::get(&writer, &JsValue::from_str("releaseLock")) {
            if release.is_function() {
                let f: Function = release.unchecked_into();
                let _ = f.call0(&writer);
            }
        }

        Ok(())
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
    /// Request a port from the user and return a `WebSerialPort` owning that port.
    /// The first argument `info` will not be used, as the user selects the port.
    async fn request_port(_: PortInfo, config: PortConfig) -> CoreResult<Self> {
        Self::request_port(config).await
    }

    async fn open(&mut self) -> project_core::Result<()> {
        let options = web_sys::SerialOptions::new(self.config.baud_rate);
        options.set_data_bits(self.config.data_bits);
        options.set_stop_bits(self.config.stop_bits);

        let promise = self.port.open(&options);
        JsFuture::from(promise)
            .await
            .map_err(|_| project_core::Error::OpenError("Failed to open port".to_string()))?;
        self.is_open = true;
        Ok(())
    }

    async fn close(&mut self) -> project_core::Result<()> {
        let promise = self.port.close();
        JsFuture::from(promise)
            .await
            .map_err(|_| project_core::Error::OpenError("Failed to close port".to_string()))?;
        self.is_open = false;
        Ok(())
    }
    async fn read(&mut self) -> project_core::Result<project_core::Message> {
        if !self.is_open {
            return Err(project_core::Error::ReadError(
                "Port is not open".to_string(),
            ));
        }
        let array = self.read().await?;
        let text = String::from_utf8_lossy(&array.to_vec()).to_string();
        let timestamp = project_core::Timestamp(Date::now() as u64);
        Ok(project_core::Message::new(
            timestamp,
            project_core::Direction::In,
            text,
        ))
    }

    async fn write(&mut self, message: project_core::Message) -> project_core::Result<()> {
        if !self.is_open {
            return Err(project_core::Error::WriteError(
                "Port is not open".to_string(),
            ));
        }
        // Convert message text to bytes
        let bytes = message.text().as_bytes();
        let array = Uint8Array::from(bytes);

        self.write(&array).await
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    /// Web Serial doesn't expose a flush primitive; noop for now.
    async fn flush(&mut self) -> project_core::Result<()> {
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

        let infos = JsFuture::from(promise)
            .await
            .map_err(|_| project_core::Error::DeviceNotFound("Failed to get ports".to_string()))?
            .dyn_into::<js_sys::Array>()
            .map_err(|_| {
                project_core::Error::DeviceNotFound("Failed to convert ports".to_string())
            })?;
        let mut result = vec![];
        for info in infos {
            if info.is_undefined() || info.is_null() {
                continue;
            }
            if let Some(port) = info.dyn_into::<web_sys::SerialPort>().ok() {
                let info = js_port_to_port_info(&port, "Web Serial Port".to_string()).await;
                result.push(info);
            }
        }
        Ok(result)
    }
}
