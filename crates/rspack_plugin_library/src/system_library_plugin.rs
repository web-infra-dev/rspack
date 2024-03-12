use std::hash::Hash;

use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  AdditionalChunkRuntimeRequirementsArgs, ChunkUkey, Compilation, ExternalModule, ExternalRequest,
  JsChunkHashArgs, LibraryName, LibraryNonUmdObject, LibraryOptions, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  PluginRenderHookOutput, RenderArgs, RuntimeGlobals,
};
use rspack_error::{error, error_bail, Result};

use crate::utils::{external_module_names, get_options_for_chunk, COMMON_LIBRARY_NAME_MESSAGE};

#[derive(Debug)]
struct SystemLibraryPluginParsed<'a> {
  name: Option<&'a str>,
}

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

#[async_trait::async_trait]
impl Plugin for SystemLibraryPlugin {
  fn name(&self) -> &'static str {
    "rspack.SystemLibraryPlugin"
  }

  async fn additional_chunk_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let Some(_) = self.get_options_for_chunk(args.compilation, args.chunk)? else {
      return Ok(());
    };
    args
      .runtime_requirements
      .insert(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME);
    Ok(())
  }

  fn render(&self, _ctx: PluginContext, args: &RenderArgs) -> PluginRenderHookOutput {
    let compilation = &args.compilation;
    let Some(options) = self.get_options_for_chunk(compilation, args.chunk)? else {
      return Ok(None);
    };
    // system-named-assets-path is not supported
    let name = options
      .name
      .map(serde_json::to_string)
      .transpose()
      .map_err(|e| error!(e.to_string()))?
      .map(|s| format!("{s}, "))
      .unwrap_or_else(|| "".to_string());

    let modules = compilation
      .chunk_graph
      .get_chunk_module_identifiers(args.chunk)
      .iter()
      .filter_map(|identifier| {
        compilation
          .get_module_graph()
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
    let external_deps_array =
      serde_json::to_string(&external_deps_array).map_err(|e| error!(e.to_string()))?;
    let external_arguments = external_module_names(&modules, compilation);

    // The name of the variable provided by System for exporting
    let dynamic_export = "__WEBPACK_DYNAMIC_EXPORT__";
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
    source.add(RawSource::from(format!("System.register({name}{external_deps_array}, function({dynamic_export}, __system_context__) {{\n")));
    if !is_has_external_modules {
      // 	var __WEBPACK_EXTERNAL_MODULE_{}__ = {};
      source.add(RawSource::from(external_var_declarations));
      // Object.defineProperty(__WEBPACK_EXTERNAL_MODULE_{}__, "__esModule", { value: true });
      source.add(RawSource::from(external_var_initialization));
    }
    source.add(RawSource::from("return {\n"));
    if !is_has_external_modules {
      // setter : { [function(module){},...] },
      let setters = format!("setters: [{}],\n", setters);
      source.add(RawSource::from(setters))
    }
    source.add(RawSource::from("execute: function() {\n"));
    source.add(RawSource::from(format!("{dynamic_export}(")));
    source.add(args.source.clone());
    source.add(RawSource::from(")}\n"));
    source.add(RawSource::from("}\n"));
    source.add(RawSource::from("\n})"));
    Ok(Some(source.boxed()))
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    let Some(options) = self.get_options_for_chunk(args.compilation, args.chunk_ukey)? else {
      return Ok(());
    };
    self.name().hash(&mut args.hasher);
    if let Some(name) = options.name {
      name.hash(&mut args.hasher);
    }
    Ok(())
  }
}
