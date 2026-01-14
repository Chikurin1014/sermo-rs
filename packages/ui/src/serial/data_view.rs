use dioxus::prelude::*;

/// Simple renderer for message data. Return an Element so callers can embed it
/// in `rsx!` expressions without using component props macros.
pub fn render_data(data: &str) -> Element {
    rsx!( span { class: "message-data", "{data}" } )
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
