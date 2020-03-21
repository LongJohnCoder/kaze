use crate::graph;

use std::collections::HashMap;

pub struct InstanceDecls {
    pub input_names: HashMap<String, String>,
    pub output_names: HashMap<String, String>,
}

pub struct MemDecls {
    pub write_address_name: String,
    pub write_value_name: String,
    pub write_enable_name: String,
}

pub struct RegisterDecls<'a> {
    pub(super) data: &'a graph::RegisterData<'a>,
    pub value_name: String,
    pub next_name: String,
}

pub struct ModuleDecls<'graph> {
    pub instances: HashMap<&'graph graph::Instance<'graph>, InstanceDecls>,
    pub mems: HashMap<&'graph graph::Mem<'graph>, MemDecls>,
    pub regs: HashMap<&'graph graph::Signal<'graph>, RegisterDecls<'graph>>,
}
