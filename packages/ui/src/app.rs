use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;

use project_core::{
    serial::{PortConfig, PortInfo, SerialPort},
    TimeSource,
};

use crate::hero::Hero;
use crate::serial_context::SerialContext;

#[allow(non_snake_case)]
#[component]
pub fn App<S: SerialPort + 'static, T: TimeSource + 'static>() -> Element {
    let _ = std::marker::PhantomData::<T>;

    let mut ports = use_signal(|| HashMap::<Uuid, S>::new());
    let port_list = use_memo(move || {
        let ports = ports.read();
        let mut list = HashMap::<Uuid, PortInfo>::new();
        for (id, port) in ports.iter() {
            list.insert(*id, port.info().clone());
        }
        list
    });
    let request_port = use_callback(move |(info, config): (PortInfo, PortConfig)| {
        let mut res = use_signal(|| None);
        use_future(move || {
            let info = info.clone();
            let config = config.clone();
            async move {
                let port = match S::request_port(info, config).await {
                    Ok(p) => p,
                    Err(e) => {
                        res.set(Some(Err(e)));
                        return;
                    }
                };
                let info = port.info();
                ports.write().insert(info.id, port);
                res.set(Some(Ok(())))
            }
        });
        // In order to resolve a lifetime issue, you need to clone result here.
        let res = res.read().clone().transpose().map(|_| ());
        res
    });

    use_context_provider(|| SerialContext {
        request_port,
        port_list: port_list.into(),
    });

    rsx! {
        div {
            class: "app-container",
            h1 { "Sermo - Serial Monitor" }
            p { "Serial port application ready" }
        }

        Hero {}

        crate::layout::Layout {}
    }
}
