use rspack_core::{BoxPlugin, ExternalItem, ExternalItemFnResult, ExternalItemValue, PluginExt};

use crate::{ExternalsPlugin, node_builtins::is_node_builtin};

pub fn esm_node_target_plugin() -> BoxPlugin {
  ExternalsPlugin::new(
    "node-commonjs".to_string(),
    vec![ExternalItem::Fn(Box::new(|ctx| {
      Box::pin(async move {
        if !is_node_builtin(&ctx.request) {
          return Ok(ExternalItemFnResult {
            external_type: None,
            result: None,
          });
        }
        Ok(ExternalItemFnResult {
          external_type: if ctx.dependency_type == "esm" {
            Some("module-import".to_string())
          } else {
            None
          },
          result: Some(ExternalItemValue::Bool(true)),
        })
      })
    }))],
    false,
  )
  .boxed()
}
