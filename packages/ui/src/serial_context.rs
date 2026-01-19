use dioxus::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use project_core::{
    serial::{PortConfig, PortInfo},
    Result as CoreResult,
};

#[derive(Clone, PartialEq)]
pub struct SerialContext {
    pub request_port: Callback<(PortInfo, PortConfig), CoreResult<()>>,
    pub port_list: ReadSignal<HashMap<Uuid, PortInfo>>,
}
