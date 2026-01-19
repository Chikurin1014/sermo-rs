use dioxus::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn PortList() -> Element {
    let serial_context = use_context::<crate::serial_context::SerialContext>();
    let port_list = serial_context.port_list;

    rsx!(
        div { class: "port-list",
            h3 { "Ports" }
            ul {
                {
                    // port_list is a HashMap<Uuid, PortInfo>; iterate over (id, info)
                    port_list.read().iter().map(|(id, port_info)| {
                        let id_str = id.to_string();
                        let name = port_info.port.clone();
                        match &port_info.port_type {
                            project_core::serial::PortType::WebSerial {
                                vendor_id,
                                product_id,
                                bluetooth_service_class_id: _
                            } => {
                                let vid_str = match vendor_id {
                                    Some(id) => format!("{:04X}", id),
                                    None => "None".to_string(),
                                };
                                let pid_str = match product_id {
                                    Some(id) => format!("{:04X}", id),
                                    None => "None".to_string(),
                                };
                                let description = port_info.description
                                    .clone()
                                    .map(|d| format!(" - {}", d))
                                    .unwrap_or_default();
                                rsx!(
                                    li {
                                        key: "{id_str}",
                                        "{name} ({pid_str}:{vid_str}){description}"
                                    }
                                )
                            }
                            _ => {
                                let description = port_info.description
                                    .clone()
                                    .map(|d| format!(" - {}", d))
                                    .unwrap_or_default();
                                rsx!(
                                    li {
                                        key: "{id_str}",
                                        "{name}{description}"
                                    }
                                )
                            }
                        }
                    })
                }
            }
        }
    )
}
