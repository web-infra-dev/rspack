mod eval_dev_tool_module_plugin;
mod eval_source_map_dev_tool_plugin;
mod generate_debug_id;
mod module_filename_helpers;
mod source_map_dev_tool_module_options_plugin;
mod source_map_dev_tool_plugin;

use std::sync::{Arc, LazyLock};

pub use eval_dev_tool_module_plugin::*;
pub use eval_source_map_dev_tool_plugin::*;
use futures::future::BoxFuture;
use rspack_core::{Compilation, ModuleIdentifier, runtime_mode::RuntimeMode};
use rspack_error::Result;
use rspack_paths::Utf8Path;
use rspack_util::{identifier::make_paths_absolute, node_path::NodePath};
pub use source_map_dev_tool_module_options_plugin::*;
pub use source_map_dev_tool_plugin::*;

static WEBPACK_SOURCE_MAP_MODULE_FILENAME_TEMPLATE: LazyLock<ModuleFilenameTemplate> =
  LazyLock::new(|| {
    ModuleFilenameTemplate::String("webpack://[namespace]/[resourcePath]".to_string())
  });
static RSPACK_SOURCE_MAP_MODULE_FILENAME_TEMPLATE: LazyLock<ModuleFilenameTemplate> =
  LazyLock::new(|| {
    ModuleFilenameTemplate::String("rspack://[namespace]/[resourcePath]".to_string())
  });
static WEBPACK_SOURCE_MAP_MODULE_FILENAME_TEMPLATE_WITH_HASH: LazyLock<ModuleFilenameTemplate> =
  LazyLock::new(|| {
    ModuleFilenameTemplate::String("webpack://[namespace]/[resourcePath]?[hash]".to_string())
  });
static RSPACK_SOURCE_MAP_MODULE_FILENAME_TEMPLATE_WITH_HASH: LazyLock<ModuleFilenameTemplate> =
  LazyLock::new(|| {
    ModuleFilenameTemplate::String("rspack://[namespace]/[resourcePath]?[hash]".to_string())
  });

pub(crate) fn default_source_map_module_filename_template(
  runtime_mode: RuntimeMode,
) -> &'static ModuleFilenameTemplate {
  match runtime_mode {
    RuntimeMode::Webpack => &WEBPACK_SOURCE_MAP_MODULE_FILENAME_TEMPLATE,
    RuntimeMode::Rspack => &RSPACK_SOURCE_MAP_MODULE_FILENAME_TEMPLATE,
  }
}

pub(crate) fn default_source_map_fallback_module_filename_template(
  runtime_mode: RuntimeMode,
) -> &'static ModuleFilenameTemplate {
  match runtime_mode {
    RuntimeMode::Webpack => &WEBPACK_SOURCE_MAP_MODULE_FILENAME_TEMPLATE_WITH_HASH,
    RuntimeMode::Rspack => &RSPACK_SOURCE_MAP_MODULE_FILENAME_TEMPLATE_WITH_HASH,
  }
}

pub(crate) fn default_eval_module_filename_template(
  runtime_mode: RuntimeMode,
) -> &'static ModuleFilenameTemplate {
  default_source_map_fallback_module_filename_template(runtime_mode)
}

pub type ModuleFilenameTemplateFn =
  Arc<dyn Fn(ModuleFilenameTemplateFnCtx) -> BoxFuture<'static, Result<String>> + Sync + Send>;

pub struct ModuleFilenameTemplateFnCtx {
  pub identifier: String,
  pub short_identifier: String,
  pub resource: String,
  pub resource_path: String,
  pub absolute_resource_path: String,
  pub relative_resource_path: Option<String>,
  pub loaders: String,
  pub all_loaders: String,
  pub query: String,
  pub module_id: String,
  pub hash: String,
  pub namespace: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SourceReference {
  Source(Arc<str>),
  Module(ModuleIdentifier),
}

fn resolve_source_reference(compilation: &Compilation, source: &str) -> SourceReference {
  let Some(request) = source
    .strip_prefix("webpack://")
    .or_else(|| source.strip_prefix("rspack://"))
  else {
    return SourceReference::Source(Arc::from(source));
  };

  let context = &compilation.options.context;
  let identifier = make_paths_absolute(context.as_str(), request);
  if let Some(module) = compilation
    .get_module_graph()
    .module_by_identifier(&ModuleIdentifier::from(identifier.as_str()))
  {
    return SourceReference::Module(module.identifier());
  }

  // An unmatched source is a path from a map. Only module lookup needs to
  // interpret loader and identifier separators.
  if request.starts_with("./") || request.starts_with("../") {
    let (path, suffix) = module_filename_helpers::split_source_request(request);
    let mut absolute = Utf8Path::new(context.as_str())
      .node_join(path)
      .node_normalize()
      .into_string();
    absolute.push_str(suffix);
    SourceReference::Source(Arc::from(absolute))
  } else {
    SourceReference::Source(Arc::from(request))
  }
}
