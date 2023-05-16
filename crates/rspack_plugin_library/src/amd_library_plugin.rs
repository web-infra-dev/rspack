use std::hash::Hash;

use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  AdditionalChunkRuntimeRequirementsArgs, ExternalModule, Filename, JsChunkHashArgs, LibraryName,
  LibraryOptions, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext,
  PluginJsChunkHashHookOutput, PluginRenderHookOutput, RenderArgs, RuntimeGlobals, SourceType,
};
use rspack_error::Result;

use super::utils::{external_arguments, external_dep_array};

#[derive(Debug)]
pub struct AmdLibraryPlugin {
  pub require_as_wrapper: bool,
}

impl AmdLibraryPlugin {
  pub fn new(require_as_wrapper: bool) -> Self {
    Self { require_as_wrapper }
  }

  pub fn normalize_name(&self, o: &Option<LibraryOptions>) -> Result<Option<String>> {
    if let Some(LibraryOptions {
      name: Some(LibraryName {
        root: Some(root), ..
      }),
      ..
    }) = o
    {
      // TODO error "AMD library name must be a simple string or unset."
      if let Some(name) = root.get(0) {
        return Ok(Some(name.to_string()));
      }
    }
    Ok(None)
  }
}

impl Plugin for AmdLibraryPlugin {
  fn name(&self) -> &'static str {
    "AmdLibraryPlugin"
  }

  fn additional_chunk_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    args
      .runtime_requirements
      .insert(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME);
    Ok(())
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
    let external_arguments = external_arguments(&modules, compilation);
    let mut fn_start = format!("function({external_arguments}){{\n");
    if compilation.options.output.iife || !chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      fn_start.push_str(" return ");
    }
    let name = self.normalize_name(&compilation.options.output.library)?;
    let mut source = ConcatSource::default();
    if self.require_as_wrapper {
      source.add(RawSource::from(format!(
        "require({external_deps_array}, {fn_start}"
      )));
    } else if let Some(name) = name {
      let normalize_name =
        Filename::from(name).render_with_chunk(chunk, ".js", &SourceType::JavaScript, None);
      source.add(RawSource::from(format!(
        "define('{normalize_name}', {external_deps_array}, {fn_start}"
      )));
    } else if modules.is_empty() {
      source.add(RawSource::from(format!("define({fn_start}, ")));
    } else {
      source.add(RawSource::from(format!(
        "define({external_deps_array}, {fn_start}"
      )));
    }
    source.add(args.source.clone());
    source.add(RawSource::from("\n});"));
    Ok(Some(source.boxed()))
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    self.name().hash(&mut args.hasher);
    args
      .compilation
      .options
      .output
      .library
      .hash(&mut args.hasher);
    Ok(())
  }
}
