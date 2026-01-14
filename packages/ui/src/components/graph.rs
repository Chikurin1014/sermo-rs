use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn Graph() -> Element {
    rsx!( div { class: "graph-placeholder", h4 { "Graph Placeholder" } p { "(replace with real graph later)" } } )
}
