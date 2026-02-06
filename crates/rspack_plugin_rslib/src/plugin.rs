use std::{
  sync::Arc,
  time::{Duration, Instant},
};

use rspack_core::{
  AssetEmittedInfo, ChunkUkey, Compilation, CompilationParams, CompilerAssetEmitted,
  CompilerCompilation, CompilerFinishMake, ModuleType, NormalModuleFactoryParser,
  ParserAndGenerator, ParserOptions, Plugin, get_module_directives, get_module_hashbang,
  rspack_sources::{ConcatSource, RawStringSource, Source, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_asset::AssetParserAndGenerator;
use rspack_plugin_javascript::{
  BoxJavascriptParserPlugin, JavascriptModulesRender, JsPlugin, RenderSource,
  parser_and_generator::JavaScriptParserAndGenerator,
};

use crate::{
  asset::RslibAssetParserAndGenerator, hashbang_parser_plugin::HashbangParserPlugin,
  import_dependency::RslibDependencyTemplate,
  import_external::replace_import_dependencies_for_external_modules,
  parser_plugin::RslibParserPlugin, react_directives_parser_plugin::ReactDirectivesParserPlugin,
};

#[derive(Debug)]
pub struct RslibPluginOptions {
  pub intercept_api_plugin: bool,
  pub force_node_shims: bool,
}

#[derive(Debug)]
pub struct ProgressPluginStateInfo {
  pub value: String,
  pub time: Instant,
  pub duration: Option<Duration>,
}

#[plugin]
#[derive(Debug)]
pub struct RslibPlugin {
  options: RslibPluginOptions,
}

impl RslibPlugin {
  pub fn new(options: RslibPluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(NormalModuleFactoryParser for RslibPlugin)]
async fn nmf_parser(
  &self,
  module_type: &ModuleType,
  parser: &mut Box<dyn ParserAndGenerator>,
  _parser_options: Option<&ParserOptions>,
) -> Result<()> {
  if let Some(parser) = parser.downcast_mut::<JavaScriptParserAndGenerator>() {
    if module_type.is_js_like() {
      parser.add_parser_plugin(Box::new(HashbangParserPlugin) as BoxJavascriptParserPlugin);
      parser.add_parser_plugin(Box::new(ReactDirectivesParserPlugin) as BoxJavascriptParserPlugin);
      parser.add_parser_plugin(
        Box::new(RslibParserPlugin::new(self.options.intercept_api_plugin))
          as BoxJavascriptParserPlugin,
      );
    }

    if module_type.is_js_esm() && self.options.force_node_shims {
      // force_node_shims means we want to handle CJS shims (__dirname/__filename) in ESM modules
      // So we use handle_cjs=true to enable __dirname/__filename handling
      parser.add_parser_plugin(Box::new(
        rspack_plugin_javascript::node_stuff_plugin::NodeStuffPlugin::new(true, false),
      ) as BoxJavascriptParserPlugin);
    }
  } else if parser.is::<AssetParserAndGenerator>() {
    // Wrap AssetParserAndGenerator to customize source types
    *parser = Box::new(RslibAssetParserAndGenerator(
      parser
        .downcast_ref::<AssetParserAndGenerator>()
        .expect("is AssetParser")
        .clone(),
    ))
  }

  Ok(())
}

#[plugin_hook(CompilerCompilation for RslibPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_template(
    RslibDependencyTemplate::template_type(),
    Arc::new(RslibDependencyTemplate::default()),
  );

  // Register render hook for hashbang and directives handling during chunk generation
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.render.tap(render::new(self));
  drop(hooks);

  Ok(())
}

#[plugin_hook(JavascriptModulesRender for RslibPlugin)]
async fn render(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  render_source: &mut RenderSource,
) -> Result<()> {
  // NOTE: This function handles hashbang and directives for non new ESM library formats.
  // Similar logic exists in rspack_plugin_esm_library/src/render.rs for ESM format,
  // as that plugin's render path is used instead when ESM library plugin is enabled.
  let entry_modules = compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_entry_modules(chunk_ukey);
  if entry_modules.is_empty() {
    return Ok(());
  }

  let module_graph = compilation.get_module_graph();

  for entry_module_id in &entry_modules {
    let hashbang = get_module_hashbang(module_graph, entry_module_id);
    let directives = get_module_directives(module_graph, entry_module_id);

    if hashbang.is_none() && directives.is_none() {
      continue;
    }

    let original_source_str = render_source.source.source().into_string_lossy();

    let mut new_source = ConcatSource::default();

    if let Some(hashbang) = hashbang {
      new_source.add(RawStringSource::from(format!("{hashbang}\n")));
    }

    if let Some(directives) = directives {
      let use_strict_prefix = "\"use strict\";\n";
      if let Some(rest) = original_source_str.strip_prefix(use_strict_prefix) {
        new_source.add(RawStringSource::from(use_strict_prefix));
        for directive in directives {
          new_source.add(RawStringSource::from(format!("{directive}\n")));
        }
        new_source.add(RawStringSource::from(rest));
      } else {
        for directive in directives {
          new_source.add(RawStringSource::from(format!("{directive}\n")));
        }
        new_source.add(render_source.source.clone());
      }
    } else {
      new_source.add(render_source.source.clone());
    }

    render_source.source = new_source.boxed();
    break;
  }

  Ok(())
}

#[plugin_hook(CompilerFinishMake for RslibPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  // Replace ImportDependency instances with RslibImportDependency for external modules
  replace_import_dependencies_for_external_modules(compilation)?;
  Ok(())
}

#[plugin_hook(CompilerAssetEmitted for RslibPlugin)]
async fn asset_emitted(
  &self,
  compilation: &Compilation,
  _filename: &str,
  info: &AssetEmittedInfo,
) -> Result<()> {
  use rspack_fs::FilePermissions;

  let content = info.source.source().into_string_lossy();
  if content.starts_with("#!") {
    let output_fs = &compilation.output_filesystem;
    let permissions = FilePermissions::from_mode(0o755);
    output_fs
      .set_permissions(&info.target_path, permissions)
      .await?;
  }
  Ok(())
}

impl Plugin for RslibPlugin {
  fn name(&self) -> &'static str {
    "rslib"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .normal_module_factory_hooks
      .parser
      .tap(nmf_parser::new(self));

    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));
    ctx
      .compiler_hooks
      .asset_emitted
      .tap(asset_emitted::new(self));

    Ok(())
  }
}
