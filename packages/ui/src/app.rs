use dioxus::prelude::*;
use project_core::SerialPort;
use std::marker::PhantomData;

use crate::hero::Hero;

#[allow(non_snake_case)]
#[component]
pub fn App<S: SerialPort + 'static>() -> Element {
    let _phantom = PhantomData::<S>;

    rsx! {
        div {
            class: "app-container",
            h1 { "Sermo - Serial Monitor" }
            p { "Serial port application ready" }
        }

        Hero {}
    }
}
