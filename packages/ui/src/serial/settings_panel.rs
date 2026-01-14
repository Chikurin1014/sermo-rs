use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn SettingsPanel() -> Element {
    rsx!(
        div { class: "settings-panel",
            h4 { "Settings" }
            p { "(settings skeleton)" }
        }
    )
}
