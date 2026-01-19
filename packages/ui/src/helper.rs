use dioxus::prelude::*;

use project_core::data::Message;

/// Simple renderer for message data. Return an Element so callers can embed it
/// in `rsx!` expressions without using component props macros.
pub fn render_data(data: &str) -> Element {
    rsx!( span { class: "message-data", "{data}" } )
}

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
                { messages.iter().map(|m: &Message| {
                    let ts_ms = u64::from(m.timestamp());
                    let ts_str = if let Some(f) = format_timestamp { f(ts_ms) } else { format!("{}", ts_ms) };
                    let label = format!("[{}] {}", ts_str, m.direction());
                    rsx!( li { span { class: "message-label", "{label}" } { crate::helper::render_data(m.text()) } } )
                }) }
            }
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_data_handles_empty_and_text() {
        // Basic compilation-time smoke test: ensure the helper can be called
        let _ = render_data("");
        let _ = render_data("hello world");
    }
}
