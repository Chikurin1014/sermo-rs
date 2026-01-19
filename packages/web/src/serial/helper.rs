use dioxus::logger::tracing::warn;
use js_sys::{Function, Reflect};
use uuid::Uuid;
use wasm_bindgen::{JsCast, JsValue};

use project_core::serial::{PortInfo, PortType};

// Try to extract `PortInfo` from a `web_sys::SerialPort` using the
// Web Serial `getInfo()` method when available. This is an async helper
// because `getInfo()` returns a Promise.
pub async fn js_port_to_port_info(port: &web_sys::SerialPort) -> PortInfo {
    let port_js = JsValue::from(port);
    let info_js = {
        let Ok(get_info_js) = Reflect::get(&port_js, &JsValue::from_str("getInfo")) else {
            warn!("`getInfo` not found on port_js");
            return PortInfo::default();
        };
        if !get_info_js.is_function() {
            warn!("`getInfo` is not a function on port_js");
            return PortInfo::default();
        }
        let get_info: Function = get_info_js.unchecked_into();
        let Ok(info_js) = get_info.call0(&port_js) else {
            warn!("`getInfo` call failed on port_js");
            return PortInfo::default();
        };
        info_js
    };

    // Extract USB/metadata fields if present
    let vendor_id = Reflect::get(&info_js, &JsValue::from_str("usbVendorId"))
        .ok()
        .and_then(|v| v.as_f64().map(|n| n as u16));
    let product_id = Reflect::get(&info_js, &JsValue::from_str("usbProductId"))
        .ok()
        .and_then(|v| v.as_f64().map(|n| n as u16));

    PortInfo::with_id(
        Uuid::new_v4(),
        "WebSerial Device".to_string(),
        PortType::WebSerial {
            vendor_id,
            product_id,
            bluetooth_service_class_id: None,
        },
        None,
    )
}
