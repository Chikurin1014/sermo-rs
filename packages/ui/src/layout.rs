use dioxus::prelude::*;

use crate::components::{ConnectionBar, PortList, SettingsPanel};

#[allow(non_snake_case)]
#[component]
pub fn Layout() -> Element {
    rsx!(
        div { class: "app-layout",
            // Top actions slot: platforms can position a button over this area
            header { class: "top-actions",
                // By default the shared UI is presentational only; platform
                // wrappers (like the web crate) may render an interactive
                // request button adjacent to this region.
                div { class: "top-actions-inner", "" }
            }

            // Left sidebar
            aside { class: "sidebar",
                ConnectionBar {}
                PortList {}
                SettingsPanel {}
            }

            // Main content area: top graph placeholder and bottom console
            main { class: "main-area",
                section { class: "graph-area",
                    h3 { "Graph" }
                    div { class: "graph-placeholder", "(graph placeholder)" }
                }
                section { class: "console-area",
                    div { class: "terminal-view",
                        h4 { "Terminal" }
                        p { "(terminal view skeleton)"}
                    }
                }
            }
        }
    )
}
