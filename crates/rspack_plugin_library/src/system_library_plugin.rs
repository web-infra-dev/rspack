use std::hash::Hash;

use rspack_core::{
  ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements, CompilationParams,
  CompilerCompilation, ExternalModule, ExternalRequest, Filename, LibraryName, LibraryNonUmdObject,
  LibraryOptions, PathData, Plugin, RuntimeGlobals,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
};
use rspack_error::{Result, ToStringResultToRspackResultExt, error_bail};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRender, JsPlugin, RenderSource,
};

use crate::utils::{COMMON_LIBRARY_NAME_MESSAGE, external_module_names, get_options_for_chunk};

const PLUGIN_NAME: &str = "rspack.SystemLibraryPlugin";

#[derive(Debug)]
struct SystemLibraryPluginParsed<'a> {
  name: Option<&'a str>,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
struct SystemLibraryJavascriptModulesPluginPlugin;

#[plugin]
#[derive(Debug, Default)]
pub struct SystemLibraryPlugin;

impl SystemLibraryPlugin {
  fn parse_options<'a>(
    &self,
    library: &'a LibraryOptions,
  ) -> Result<SystemLibraryPluginParsed<'a>> {
    if let Some(name) = &library.name
      && !matches!(
        name,
        LibraryName::NonUmdObject(LibraryNonUmdObject::String(_))
      )
    {
      error_bail!(
        "System.js library name must be a simple string or unset. {COMMON_LIBRARY_NAME_MESSAGE}"
      )
    }
    Ok(SystemLibraryPluginParsed {
      name: library.name.as_ref().map(|n| match n {
        LibraryName::NonUmdObject(LibraryNonUmdObject::String(s)) => s.as_str(),
        _ => unreachable!("System.js library name must be a simple string or unset."),
      }),
    })
  }

  fn get_options_for_chunk<'a>(
    &self,
    compilation: &'a Compilation,
    chunk_ukey: &'a ChunkUkey,
  ) -> Result<Option<SystemLibraryPluginParsed<'a>>> {
    get_options_for_chunk(compilation, chunk_ukey)
      .filter(|library| library.library_type == "system")
      .map(|library| self.parse_options(library))
      .transpose()
  }
}

#[plugin_hook(CompilerCompilation for SystemLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.render.tap(render::new(self));
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRender for SystemLibraryPlugin)]
async fn render(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  // system-named-assets-path is not supported
  let name = if let Some(name) = options.name {
    let chunk = compilation.chunk_by_ukey.get(chunk_ukey);
    let filename = Filename::from(name);
    let path_data = PathData::default()
      .chunk_id_optional(
        chunk.and_then(|c| c.id(&compilation.chunk_ids_artifact).map(|id| id.as_str())),
      )
      .chunk_name_optional(chunk.and_then(|c| c.name()))
      .chunk_hash_optional(chunk.and_then(|c| {
        c.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        )
      }));
    let name = compilation.get_path(&filename, path_data).await?;
    let name_str = serde_json::to_string(&name).to_rspack_result()?;
    format!("{name_str}, ")
  } else {
    "".to_string()
  };

  let module_graph = compilation.get_module_graph();
  let modules = compilation
    .chunk_graph
    .get_chunk_modules_identifier(chunk_ukey)
    .iter()
    .filter_map(|identifier| {
      module_graph
        .module_by_identifier(identifier)
        .and_then(|module| module.as_external_module())
        .and_then(|m| (m.get_external_type() == "system").then_some(m))
    })
    .collect::<Vec<&ExternalModule>>();
  let external_deps_array = modules
    .iter()
    .map(|m| match &m.request {
      ExternalRequest::Single(request) => Some(request.primary()),
      ExternalRequest::Map(map) => map.get("amd").map(|request| request.primary()),
    })
    .collect::<Vec<_>>();
  let external_deps_array = serde_json::to_string(&external_deps_array).to_rspack_result()?;
  let external_arguments = external_module_names(&modules, compilation);

  // The name of the variable provided by System for exporting
  let dynamic_export = "__rspack_dynamic_export";
  let external_var_declarations = external_arguments
    .iter()
    .map(|name| format!("var {name} = {{}};\n"))
    .collect::<Vec<_>>()
    .join("");
  let external_var_initialization = external_arguments
    .iter()
    .map(|name| format!("Object.defineProperty( {name} , \"__esModule\", {{ value: true }});\n"))
    .collect::<Vec<_>>()
    .join("");
  let setters = external_arguments
    .iter()
    .map(|name| {
      format!(
        "function(module) {{\n\tObject.keys(module).forEach(function(key) {{\n {name}[key] = module[key]; }})\n}}"
      )
    })
    .collect::<Vec<_>>()
    .join(",\n");
  let is_has_external_modules = modules.is_empty();
  let mut source = ConcatSource::default();
  source.add(RawStringSource::from(format!("System.register({name}{external_deps_array}, function({dynamic_export}, __system_context__) {{\n")));
  if !is_has_external_modules {
    // 	var __rspack_external_{} = {};
    source.add(RawStringSource::from(external_var_declarations));
    // Object.defineProperty(__rspack_external_{}, "__esModule", { value: true });
    source.add(RawStringSource::from(external_var_initialization));
  }
  source.add(RawStringSource::from_static("return {\n"));
  if !is_has_external_modules {
    // setter : { [function(module){},...] },
    let setters = format!("setters: [{setters}],\n");
    source.add(RawStringSource::from(setters))
  }
  source.add(RawStringSource::from_static("execute: function() {\n"));
  source.add(RawStringSource::from(format!("{dynamic_export}(")));
  source.add(render_source.source.clone());
  source.add(RawStringSource::from_static(")}\n"));
  source.add(RawStringSource::from_static("}\n"));
  source.add(RawStringSource::from_static("\n})"));
  render_source.source = source.boxed();
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for SystemLibraryPlugin)]
async fn js_chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  PLUGIN_NAME.hash(hasher);
  if let Some(name) = options.name {
    name.hash(hasher);
  }
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for SystemLibraryPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let Some(_) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  runtime_requirements.insert(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME);
  Ok(())
}

impl Plugin for SystemLibraryPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    Ok(())
  }
}
