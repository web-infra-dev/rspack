mod chunk_link;
mod dependency;
mod esm_lib_parser_plugin;
mod evaluation;
mod initializer;
mod link;
mod optimize_chunks;
mod plugin;
mod preserve_modules;
mod render;
mod runtime;
mod split_chunks;

pub use plugin::EsmLibraryPlugin;
pub use split_chunks::{GetNameGetter, ModuleFilter, ModuleTypeFilter};

/// Whether a module contributes only a CSS asset and therefore has no
/// JavaScript evaluation value. Extracted CSS modules use a custom source
/// type, so their stable identifier prefix is used as the compatibility path.
pub(crate) fn is_css_only_module(
  module: &dyn rspack_core::Module,
  module_graph: &rspack_core::ModuleGraph,
) -> bool {
  let source_types = module.source_types(module_graph);
  (!source_types.is_empty()
    && source_types.iter().all(|source_type| {
      matches!(
        source_type,
        rspack_core::SourceType::Css | rspack_core::SourceType::CssImport
      )
    }))
    || module.identifier().starts_with("css|")
}
