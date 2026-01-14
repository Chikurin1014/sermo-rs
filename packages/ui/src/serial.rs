//! Shared serial-monitor UI components (skeletons)
pub mod connection_bar;
pub mod data_view;
pub mod notifications;
pub mod port_list;
pub mod settings_panel;
pub mod terminal_view;

pub use connection_bar::ConnectionBar;
pub use port_list::PortList;
pub use terminal_view::TerminalView;
// `render_data` is provided in `data_view` as a helper function; callers can
// import it explicitly if needed.
