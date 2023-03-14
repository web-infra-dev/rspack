use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  Chunk, Compilation, ExternalModule, Filename, LibraryOptions, Plugin, PluginContext,
  PluginRenderHookOutput, PluginRenderStartupHookOutput, RenderArgs, RenderStartupArgs, SourceType,
};
use rspack_error::Result;

use super::utils::external_dep_array;

#[derive(Debug)]
pub struct AmdLibraryPlugin {
  pub require_as_wrapper: bool,
}

impl AmdLibraryPlugin {
  pub fn new(require_as_wrapper: bool) -> Self {
    Self { require_as_wrapper }
  }

  pub fn normalize_name(&self, o: &Option<LibraryOptions>) -> Result<Option<&String>> {
    if let Some(library) = o {
      if let Some(name) = &library.name {
        if let Some(root) = &name.root {
          if root.len() > 1 {
            return Err("AMD library name must be a simple string or unset.".into());
          }
          if let Some(name) = root.get(0) {
            return Ok(Some(name));
          }
        }
      }
    }
    Ok(None)
  }
}

impl Plugin for AmdLibraryPlugin {
  fn name(&self) -> &'static str {
    "AmdLibraryPlugin"
  }

  fn render(&self, _ctx: PluginContext, args: &RenderArgs) -> PluginRenderHookOutput {
    let compilation = &args.compilation;
    let chunk = args.chunk();
    let modules = compilation
      .chunk_graph
      .get_chunk_module_identifiers(args.chunk)
      .iter()
      .filter_map(|identifier| {
        compilation
          .module_graph
          .module_by_identifier(identifier)
          .and_then(|module| module.as_external_module())
      })
      .collect::<Vec<&ExternalModule>>();
    let external_deps_array = external_dep_array(&modules);
    let name = self.normalize_name(&compilation.options.output.library)?;
    let mut source = ConcatSource::default();
    if self.require_as_wrapper {
      source.add(RawSource::from(format!(
        "require({external_deps_array}, function(){{\n"
      )));
    } else if let Some(name) = name {
      let normalize_name =
        Filename::from(name).render_with_chunk(chunk, ".js", &SourceType::JavaScript);
      source.add(RawSource::from(format!(
        "define('{normalize_name}', {external_deps_array}, function(){{\n"
      )));
    } else if modules.is_empty() {
      source.add(RawSource::from(format!("define(function(){{\n")));
    } else {
      source.add(RawSource::from(format!(
        "define({external_deps_array}, function(){{\n"
      )));
    }
    source.add(args.source.clone());
    source.add(RawSource::from("\n}"));
    Ok(Some(source.boxed()))
  }
}
