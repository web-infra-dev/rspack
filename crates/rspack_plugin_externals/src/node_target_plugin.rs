use rspack_core::{BoxPlugin, ExternalItem, PluginExt};
use rspack_regex::RspackRegex;

use crate::{ExternalsPlugin, node_builtins::NODE_BUILTINS};

pub fn node_target_plugin() -> BoxPlugin {
  let mut externals: Vec<ExternalItem> = NODE_BUILTINS
    .iter()
    .map(|s| ExternalItem::from(s.to_string()))
    .collect();
  externals.push(ExternalItem::from(
    RspackRegex::new("^node:").expect("Invalid regexp"),
  ));
  // Yarn PnP adds pnpapi as "builtin"
  externals.push(ExternalItem::from("pnpapi".to_string()));

  ExternalsPlugin::new("node-commonjs".to_string(), externals, false).boxed()
}
