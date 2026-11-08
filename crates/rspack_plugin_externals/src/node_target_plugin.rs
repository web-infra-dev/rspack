use rspack_core::{BoxPlugin, ExternalItem, PluginExt};

use crate::{ExternalsPlugin, node_builtins::is_node_builtin};

fn is_node_external(request: &str) -> bool {
  // Yarn PnP adds pnpapi as "builtin"
  is_node_builtin(request) || request == "pnpapi"
}

pub fn node_target_plugin() -> BoxPlugin {
  ExternalsPlugin::new(
    "node-commonjs".to_string(),
    vec![ExternalItem::RequestPredicate(is_node_external)],
    false,
  )
  .boxed()
}
