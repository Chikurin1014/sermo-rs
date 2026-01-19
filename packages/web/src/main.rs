mod serial;
mod system;

use dioxus::prelude::*;

use ui::App;

use serial::WebSerialPort;
use system::SystemTimeSource;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(WebApp);
}

#[allow(non_snake_case)]
#[component]
fn WebApp() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        App::<WebSerialPort, SystemTimeSource> {}
    }
}
