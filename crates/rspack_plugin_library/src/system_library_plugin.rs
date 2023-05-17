use std::hash::Hash;

use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  AdditionalChunkRuntimeRequirementsArgs, ExternalModule, JsChunkHashArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  PluginRenderHookOutput, RenderArgs, RuntimeGlobals,
};

use super::utils::external_system_dep_array;
use crate::utils::{external_module_names, normalize_name};

#[derive(Debug, Default)]
pub struct SystemLibraryPlugin;

impl SystemLibraryPlugin {}

impl Plugin for SystemLibraryPlugin {
  fn name(&self) -> &'static str {
    "SystemLibraryPlugin"
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
    // system-named-assets-path is not supported
    let name = normalize_name(&compilation.options.output.library)?
      .map(|name| format!("\"{name}\","))
      .unwrap_or("".to_string());

    let modules = compilation
      .chunk_graph
      .get_chunk_module_identifiers(args.chunk)
      .iter()
      .filter_map(|identifier| {
        compilation
          .module_graph
          .module_by_identifier(identifier)
          .and_then(|module| module.as_external_module())
          .and_then(|m| {
            if m.get_external_type() == "system" {
              Some(m)
            } else {
              None
            }
          })
      })
      .collect::<Vec<&ExternalModule>>();
    let external_deps_array = external_system_dep_array(&modules);
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
