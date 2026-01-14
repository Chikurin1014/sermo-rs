use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn Notifications() -> Element {
    rsx!(
        div { class: "notifications",
            p { "(notifications skeleton)" }
        }
    )
}
