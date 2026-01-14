use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn ConnectionBar() -> Element {
    rsx!(
        div { class: "connection-bar",
            h4 { "Connection" }
            p { "(connection controls skeleton)" }
        }
    )
}
