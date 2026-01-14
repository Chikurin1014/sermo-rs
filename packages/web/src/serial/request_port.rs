use dioxus::prelude::*;
use project_core::SerialPort as _;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys;

use crate::serial::WebSerialPort;

#[allow(non_snake_case)]
#[component]
pub fn RequestPort() -> Element {
    // Simple no-hook component: the button triggers `request_port()` and logs
    // the selected port info to the browser console. This avoids hook API
    // complexity while ensuring `request_port()` is exercised from the UI.
    let on_request = move |_| {
        spawn_local(async move {
            match WebSerialPort::request_port().await {
                Ok(ws) => {
                    // Try to get info from the opened web serial object
                    let msg = format!("{}: {:?}", "Selected port".to_string(), ws.info().await);
                    web_sys::console::log_1(&JsValue::from_str(&msg));
                    // Note: more detailed info can be printed here if needed
                }
                Err(e) => {
                    web_sys::console::error_1(&JsValue::from_str(&format!(
                        "request_port error: {}",
                        e
                    )));
                }
            }
        });
    };

    rsx! {
        div { class: "request-port",
            p { "Open the browser console and click the button to request a port." }
            button { onclick: on_request, "Select Port..." }
        }
    }
}
