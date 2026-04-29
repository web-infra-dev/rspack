use std::{
  path::{Component, PathBuf},
  sync::Arc,
  time::{Duration, Instant},
};

use cow_utils::CowUtils;
use pathdiff::diff_paths;
use rspack_core::{
  AssetEmittedInfo, AssetInfo, BuildInfo, BuildModuleGraphArtifact, ChunkUkey, Compilation,
  CompilationAsset, CompilationOptimizeDependencies, CompilationParams, CompilationProcessAssets,
  CompilerAssetEmitted, CompilerCompilation, DependencyType, ExportsInfoArtifact, ModuleType,
  NormalModuleFactoryParser, ParserAndGenerator, ParserOptions, Plugin, RuntimeCodeTemplate,
  SideEffectsOptimizeArtifact, get_module_directives, get_module_hashbang,
  rspack_sources::{ConcatSource, RawStringSource, Source, SourceExt},
};
use rspack_error::{Diagnostic, Result, error};
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_plugin_asset::AssetParserAndGenerator;
use rspack_plugin_externals::EsmNodeTargetPlugin;
use rspack_plugin_javascript::{
  BoxJavascriptParserPlugin, JavascriptModulesRender, JsPlugin, RenderSource,
  parser_and_generator::JavaScriptParserAndGenerator,
};

use crate::{
  asset::RslibAssetParserAndGenerator,
  dyn_import_external::{
    ExportImportedDependencyTemplate, ImportDependencyTemplate, cutout_dyn_import_externals,
    cutout_star_re_export_externals,
  },
  hashbang_parser_plugin::HashbangParserPlugin,
  parser_plugin::RslibParserPlugin,
  react_directives_parser_plugin::ReactDirectivesParserPlugin,
};

#[derive(Debug, Clone)]
pub struct RslibPluginOptions {
  pub intercept_api_plugin: bool,
  pub force_node_shims: bool,
  pub auto_cjs_node_builtin: bool,
  pub emit_dts: Option<SwcEmitDtsPluginOptions>,
}

#[derive(Debug, Clone)]
pub struct SwcEmitDtsPluginOptions {
  pub root_dir: String,
  pub declaration_dir: String,
}

#[derive(Debug, Clone)]
struct SwcEmitDtsBuildInfo {
  resource_path: String,
  code: String,
}

fn get_build_info(build_info: &BuildInfo) -> Option<SwcEmitDtsBuildInfo> {
  let value = build_info.extras.get("rspack-swc-isolated-dts-emit")?;
  let value = value.as_object()?;
  Some(SwcEmitDtsBuildInfo {
    resource_path: value.get("resource_path")?.as_str()?.to_string(),
    code: value.get("code")?.as_str()?.to_string(),
  })
}

fn emit_isolated_dts_asset(
  compilation: &mut Compilation,
  emit_dts_options: &SwcEmitDtsPluginOptions,
  dts: SwcEmitDtsBuildInfo,
) -> Result<()> {
  let SwcEmitDtsBuildInfo {
    resource_path,
    code,
  } = dts;
  let resource_path = Utf8PathBuf::from(resource_path);
  let compiler_root = compilation.options.context.as_path();
  let resolved_root_dir = resolve_emit_dts_path(compiler_root, &emit_dts_options.root_dir);
  let resolved_declaration_dir =
    resolve_emit_dts_path(compiler_root, &emit_dts_options.declaration_dir);
  let output_path = compilation.options.output.path.clone();
  let output_relative_path = resource_path
    .strip_prefix(&resolved_root_dir)
    .map_err(|_| {
      error!(
        "Failed to emit declaration files for {} because it is outside rootDir {}",
        resource_path, resolved_root_dir
      )
    })?;
  let declaration_file_path = resolved_declaration_dir
    .join(output_relative_path)
    .with_extension("d.ts");
  let filename = if let Ok(relative_path) = declaration_file_path.strip_prefix(&output_path) {
    relative_path.to_string()
  } else {
    diff_paths(&declaration_file_path, &output_path)
      .ok_or_else(|| {
        error!(
          "Failed to emit declaration files for {} because declarationDir {} can not be relativized against output.path {}",
          resource_path, resolved_declaration_dir, output_path
        )
      })?
      .to_string_lossy()
      .cow_replace('\\', "/")
      .into_owned()
  };

  compilation.emit_asset(
    filename,
    CompilationAsset::new(
      Some(RawStringSource::from(code).boxed()),
      AssetInfo {
        source_filename: Some(resource_path.as_str().to_string()),
        ..Default::default()
      },
    ),
  );

  Ok(())
}

fn resolve_emit_dts_path(base: &Utf8Path, value: &str) -> Utf8PathBuf {
  let path = Utf8Path::new(value);
  if path.is_absolute() {
    path.to_path_buf()
  } else {
    normalize_joined_path(base.as_std_path().join(path.as_std_path()))
      .to_string_lossy()
      .to_string()
      .into()
  }
}

fn normalize_joined_path(path: PathBuf) -> PathBuf {
  let mut normalized = PathBuf::new();

  for component in path.components() {
    match component {
      Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
      Component::RootDir => normalized.push(component.as_os_str()),
      Component::CurDir => {}
      Component::ParentDir => {
        normalized.pop();
      }
      Component::Normal(part) => normalized.push(part),
    }
  }

  normalized
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

#[plugin_hook(CompilerCompilation for RslibPlugin, stage=10)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let import_template = compilation.get_dependency_template(
    rspack_core::DependencyTemplateType::Dependency(DependencyType::DynamicImport),
  );
  compilation.set_dependency_template(
    rspack_core::DependencyTemplateType::Dependency(DependencyType::DynamicImport),
    Arc::new(ImportDependencyTemplate {
      template: import_template,
    }),
  );

  let export_template = compilation.get_dependency_template(
    rspack_core::DependencyTemplateType::Dependency(DependencyType::EsmExportImportedSpecifier),
  );
  compilation.set_dependency_template(
    rspack_core::DependencyTemplateType::Dependency(DependencyType::EsmExportImportedSpecifier),
    Arc::new(ExportImportedDependencyTemplate {
      template: export_template,
    }),
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
  _runtime_template: &RuntimeCodeTemplate<'_>,
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

#[plugin_hook(CompilationOptimizeDependencies for RslibPlugin)]
async fn optimize_dependencies(
  &self,
  compilation: &Compilation,
  _side_effects_optimize_artifact: &mut SideEffectsOptimizeArtifact,
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  cutout_dyn_import_externals(build_module_graph_artifact);
  cutout_star_re_export_externals(
    compilation,
    build_module_graph_artifact,
    exports_info_artifact,
  );

  Ok(None)
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

#[plugin_hook(CompilationProcessAssets for RslibPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let Some(options) = &self.options.emit_dts else {
    return Ok(());
  };
  let dts_outputs = compilation
    .get_module_graph()
    .modules()
    .filter_map(|(_, module)| get_build_info(module.build_info()))
    .collect::<Vec<_>>();

  for dts in dts_outputs {
    emit_isolated_dts_asset(compilation, options, dts)?;
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

    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));

    ctx
      .compiler_hooks
      .asset_emitted
      .tap(asset_emitted::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));

    if self.options.auto_cjs_node_builtin {
      EsmNodeTargetPlugin::new().apply(ctx)?;
    }

    Ok(())
  }
}
