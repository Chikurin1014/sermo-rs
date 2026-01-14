use dioxus::prelude::*;
use project_core::Message;

/// Render a terminal view for the given messages. This is a plain function
/// (not a component) which returns an Element so callers can embed it in
/// their components without requiring Dioxus Props or Scope types in this file.
#[allow(dead_code)]
pub fn render_terminal(
    messages: &[Message],
    format_timestamp: Option<fn(u64) -> String>,
) -> Element {
    rsx!(
        div { class: "terminal-view",
            h4 { "Terminal" }
            ul {
                {messages.iter().map(|m| {
                    let ts_str = if let Some(f) = format_timestamp { f(m.timestamp) } else { format!("{}", m.timestamp) };
                    let label = format!("[{}] {:?}", ts_str, m.direction);
                    rsx!( li { span { class: "message-label", "{label}" } { super::data_view::render_data(&m.data) } } )
                })}
            }
        }
    )
}

#[allow(non_snake_case)]
#[component]
pub fn TerminalView() -> Element {
    // Placeholder skeleton component; real usage should call `render_terminal` from a wrapper.
    rsx!( div { class: "terminal-view", h4 { "Terminal" } p { "(terminal skeleton)" } } )
}
