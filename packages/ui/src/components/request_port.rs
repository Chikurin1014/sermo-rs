use dioxus::{logger::tracing::error, prelude::*};

use crate::serial_context::SerialContext;
use project_core::serial::{PortConfig, PortInfo};

#[allow(non_snake_case)]
#[component]
pub fn RequestPort() -> Element {
    // Get SerialContext from the nearest provider. If not present, render a
    // disabled button.
    let request_port = use_context::<SerialContext>().request_port;

    let onclick = move |_| {
        // Call the platform-provided request_port callback with default
        // placeholders. The real platform implementation will prompt the
        // user and ignore the `PortInfo` argument (or use it as needed).
        request_port
            .call((PortInfo::default(), PortConfig::default()))
            .unwrap_or_else(|e| {
                error!("{}", e);
            });
    };

    rsx!(
        button { onclick: onclick, "Request Port" }
    )
}
