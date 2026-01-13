mod serial;

use dioxus::prelude::*;
use ui::App;

pub use serial::DesktopSerialPort;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(DesktopApp);
}

#[allow(non_snake_case)]
#[component]
fn DesktopApp() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        App::<DesktopSerialPort> {}
    }
}
