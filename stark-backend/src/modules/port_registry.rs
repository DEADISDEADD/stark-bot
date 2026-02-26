//! Module port registry — maps module names to actual runtime ports.
//!
//! At startup, `start_module_services()` registers each module's name and the
//! port it was assigned.  `local_rpc` resolves `module="spot_trader"` to
//! `http://127.0.0.1:<port>` transparently.

use std::collections::HashMap;
use std::sync::RwLock;

static PORT_MAP: RwLock<Option<HashMap<String, u16>>> = RwLock::new(None);

/// Register a module name → actual runtime port mapping.
pub fn register(module_name: &str, actual_port: u16) {
    let mut map = PORT_MAP.write().unwrap();
    let m = map.get_or_insert_with(HashMap::new);
    log::info!(
        "[PORT_REGISTRY] {} → port {}",
        module_name,
        actual_port
    );
    m.insert(module_name.to_string(), actual_port);
}

/// Resolve a module name to its runtime port.
pub fn resolve(module_name: &str) -> Option<u16> {
    let map = PORT_MAP.read().unwrap();
    map.as_ref().and_then(|m| m.get(module_name).copied())
}
