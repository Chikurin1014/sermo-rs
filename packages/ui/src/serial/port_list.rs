use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn PortList() -> Element {
    rsx!(
        div { class: "port-list",
            h3 { "Ports" }
            p { "(port list skeleton)" }
        }
    )
}
