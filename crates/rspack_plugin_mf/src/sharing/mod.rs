pub mod collect_shared_entry_plugin;
pub mod consume_shared_fallback_dependency;
pub mod consume_shared_module;
pub mod consume_shared_plugin;
pub mod consume_shared_runtime_module;
pub mod provide_for_shared_dependency;
pub mod provide_shared_dependency;
pub mod provide_shared_module;
pub mod provide_shared_module_factory;
pub mod provide_shared_plugin;
pub mod share_runtime_module;
pub mod share_runtime_plugin;
pub mod shared_container_plugin;
pub mod shared_container_runtime_module;
pub mod shared_used_exports_optimizer_plugin;
pub mod shared_used_exports_optimizer_runtime_module;

pub(crate) fn create_lookup_key_for_sharing(request: &str, layer: Option<&str>) -> String {
  if let Some(layer) = layer {
    return format!("({layer}){request}");
  }
  request.to_string()
}

pub(crate) fn strip_lookup_layer_prefix(lookup: &str) -> &str {
  if lookup.starts_with('(')
    && let Some(index) = lookup.find(')')
  {
    return &lookup[index + 1..];
  }
  lookup
}
