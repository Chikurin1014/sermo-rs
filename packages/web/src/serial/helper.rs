use js_sys::{Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use project_core::serial::{PortInfo, PortType};

// Try to extract `PortInfo` from a `web_sys::SerialPort` using the
// Web Serial `getInfo()` method when available. This is an async helper
// because `getInfo()` returns a Promise.
pub async fn js_port_to_port_info(port: &web_sys::SerialPort, default_name: String) -> PortInfo {
    let mut info = PortInfo::new(default_name, PortType::Other("web_serial".to_string()));

    let port_js = JsValue::from(port.clone());
    // Prefer explicit `name` or `path` if present on the port object — some
    // browser implementations expose a human-readable label directly.
    if let Ok(name_val) = Reflect::get(&port_js, &JsValue::from_str("name")) {
        if let Some(name_str) = name_val.as_string() {
            info.port = name_str;
            return info;
        }
    }
    if let Ok(path_val) = Reflect::get(&port_js, &JsValue::from_str("path")) {
        if let Some(path_str) = path_val.as_string() {
            info.port = path_str;
            return info;
        }
    }

    if let Ok(get_info_val) = Reflect::get(&port_js, &JsValue::from_str("getInfo")) {
        if get_info_val.is_function() {
            let func: Function = get_info_val.unchecked_into();
            if let Ok(promise_val) = func.call0(&port_js) {
                if let Ok(promise) = promise_val.dyn_into::<js_sys::Promise>() {
                    if let Ok(info_js) = JsFuture::from(promise).await {
                        // Extract USB/metadata fields if present
                        let vendor = Reflect::get(&info_js, &JsValue::from_str("usbVendorId"))
                            .ok()
                            .and_then(|v| v.as_f64().map(|n| n as u16));
                        let product = Reflect::get(&info_js, &JsValue::from_str("usbProductId"))
                            .ok()
                            .and_then(|v| v.as_f64().map(|n| n as u16));

                        let manufacturer =
                            Reflect::get(&info_js, &JsValue::from_str("manufacturer"))
                                .ok()
                                .and_then(|v| v.as_string());
                        let product_name =
                            Reflect::get(&info_js, &JsValue::from_str("productName"))
                                .ok()
                                .and_then(|v| v.as_string());
                        let serial_number =
                            Reflect::get(&info_js, &JsValue::from_str("serialNumber"))
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
                            // If we don't yet have a nicer port name, prefer the
                            // manufacturer/productName combo as the port label.
                            if info.port.is_empty() {
                                if let Some(m) = manufacturer.clone() {
                                    if let Some(p) = product_name.clone() {
                                        info.port = format!("{} {}", m, p);
                                    } else {
                                        info.port = m;
                                    }
                                } else if let Some(p) = product_name.clone() {
                                    info.port = p;
                                }
                            }
                        }

                        if manufacturer.is_some() || product_name.is_some() {
                            let desc = match (manufacturer.clone(), product_name.clone()) {
                                (Some(m), Some(p)) => format!("{} {}", m, p),
                                (Some(m), None) => m,
                                (None, Some(p)) => p,
                                _ => String::new(),
                            };
                            if !desc.is_empty() {
                                // Use description as human-friendly port label and also
                                // prefer it as the `port` identifier if none was found
                                info.description = Some(desc.clone());
                                if info.port.is_empty() {
                                    info.port = desc;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    info
}
