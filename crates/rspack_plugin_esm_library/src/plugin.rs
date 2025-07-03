use std::sync::{Arc, LazyLock};

use regex::Regex;
use rspack_collections::{IdentifierIndexMap, IdentifierSet};
use rspack_core::{
  get_target,
  rspack_sources::{ReplaceSource, Source},
  AssetInfo, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationAfterCodeGeneration, CompilationAfterSeal, CompilationConcatenationScope,
  CompilationFinishModules, CompilationParams, CompilationProcessAssets, CompilerCompilation,
  ConcatenatedModuleInfo, ConcatenationScope, ExportInfoGetter, ExportProvided, ExportsInfoGetter,
  ExternalModuleInfo, Logger, ModuleGraph, ModuleIdentifier, ModuleInfo, Plugin, RuntimeCondition,
  RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  dependency::ImportDependencyTemplate, JavascriptModulesRenderChunkContent, JsPlugin, RenderSource,
};
use rspack_util::fx_hash::FxHashMap;
use sugar_path::SugarPath;
use tokio::sync::Mutex;

use crate::dependency::dyn_import::DynamicImportDependencyTemplate;

#[plugin]
#[derive(Debug, Default)]
pub struct EsmLibraryPlugin {
  pub concatenated_modules_map_for_codegen:
    Mutex<FxHashMap<u32, Arc<IdentifierIndexMap<ModuleInfo>>>>,
  pub concatenated_modules_map: Mutex<FxHashMap<u32, Arc<IdentifierIndexMap<ModuleInfo>>>>,
}

#[plugin_hook(CompilerCompilation for EsmLibraryPlugin, stage=100)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  hooks
    .render_chunk_content
    .tap(render_chunk_content::new(self));

  compilation.set_dependency_template(
    ImportDependencyTemplate::template_type(),
    Arc::new(DynamicImportDependencyTemplate::default()),
  );
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderChunkContent for EsmLibraryPlugin)]
async fn render_chunk_content(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  asset_info: &mut AssetInfo,
) -> Result<Option<RenderSource>> {
  self.render_chunk(compilation, chunk_ukey, asset_info).await
}

#[plugin_hook(CompilationAfterSeal for EsmLibraryPlugin)]
async fn after_seal(&self, compilation: &mut Compilation) -> Result<()> {
  self
    .concatenated_modules_map
    .lock()
    .await
    .remove(&compilation.id().0);
  self
    .concatenated_modules_map_for_codegen
    .lock()
    .await
    .remove(&compilation.id().0);
  Ok(())
}

#[plugin_hook(CompilationFinishModules for EsmLibraryPlugin, stage = 100)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let mut modules_map = IdentifierIndexMap::default();
  let modules = module_graph.modules();
  let mut modules = modules.iter().collect::<Vec<_>>();
  modules.sort_by(|(m1, _), (m2, _)| m1.cmp(m2));
  let logger = compilation.get_logger("rspack.EsmLibraryPlugin");

  for (idx, (module_identifier, module)) in modules.into_iter().enumerate() {
    // make sure all exports are provided
    let exports_info = module_graph.get_exports_info(module_identifier);

    let prefetched_exports_info = module_graph.get_prefetched_exports_info(
      module_identifier,
      rspack_core::PrefetchExportsInfoMode::Default,
    );

    let mut should_scope_hoisting = true;

    if module.as_normal_module().is_none() {
      logger.debug(format!(
        "module {module_identifier} is not an normal module"
      ));
      should_scope_hoisting = false;
    } else if let Some(reason) =
      module.get_concatenation_bailout_reason(&module_graph, &compilation.chunk_graph)
    {
      logger.debug(format!(
        "module {module_identifier} has bailout reason: {reason}",
      ));
      should_scope_hoisting = false;
    } else if ModuleGraph::is_async(compilation, module_identifier) {
      logger.debug(format!("module {module_identifier} is an async module"));
      should_scope_hoisting = false;
    } else if !module.build_info().strict {
      logger.debug(format!("module {module_identifier} is not strict module"));
      should_scope_hoisting = false;
    }

    // if we reach here, check exports info
    if should_scope_hoisting {
      let other_export_info = exports_info
        .as_data(&module_graph)
        .other_exports_info()
        .as_data(&module_graph);
      if matches!(other_export_info.provided(), Some(ExportProvided::Unknown)) {
        logger.debug(format!("module {module_identifier} has unknown exports",));
        should_scope_hoisting = false;
      }

      for export_info in prefetched_exports_info.get_relevant_exports(None) {
        if !matches!(export_info.provided(), Some(ExportProvided::Provided)) {
          logger.debug(format!(
            "module {module_identifier} has export {} that is not provided",
            export_info
              .name()
              .map_or(String::new(), |name| name.to_string())
          ));
          should_scope_hoisting = false;
          break;
        }

        if ExportInfoGetter::is_reexport(export_info)
          && get_target(export_info, &module_graph).is_none()
        {
          logger.debug(format!(
            "module {module_identifier} has re-export that is not provided",
          ));
          should_scope_hoisting = false;
          break;
        }
      }
    }

    if should_scope_hoisting {
      modules_map.insert(
        *module_identifier,
        ModuleInfo::Concatenated(Box::new(ConcatenatedModuleInfo {
          index: idx,
          module: *module_identifier,
          ..Default::default()
        })),
      );
    } else {
      let exports_info = ExportsInfoGetter::prefetch_used_info_without_name(
        &exports_info,
        &module_graph,
        None,
        false,
      );
      modules_map.insert(
        *module_identifier,
        ModuleInfo::External(ExternalModuleInfo {
          index: idx,
          module: *module_identifier,
          runtime_condition: if exports_info.is_used() {
            RuntimeCondition::Boolean(true)
          } else {
            RuntimeCondition::Boolean(false)
          },
          interop_namespace_object_used: false,
          interop_namespace_object_name: None,
          interop_namespace_object2_used: false,
          interop_namespace_object2_name: None,
          interop_default_access_used: false,
          interop_default_access_name: None,
          runtime_requirements: RuntimeGlobals::default(),
          name: None,
        }),
      );
    }
  }

  let id = compilation.id();

  // only used for scope
  // we mutably modify data in `self.concatenated_modules_map`
  let mut self_modules_map = self.concatenated_modules_map_for_codegen.lock().await;
  self_modules_map.insert(id.0, Arc::new(modules_map.clone()));

  let mut self_modules_map = self.concatenated_modules_map.lock().await;
  self_modules_map.insert(id.0, Arc::new(modules_map));

  // mark all entry exports as used
  let mut entry_modules = IdentifierSet::default();
  for entry_data in compilation.entries.values() {
    entry_modules.extend(
      entry_data
        .all_dependencies()
        .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
        .copied(),
    );
  }

  let mut module_graph = compilation.get_module_graph_mut();
  for m in entry_modules {
    let exports_info = module_graph.get_exports_info(&m);

    exports_info.set_all_known_exports_used(&mut module_graph, None);
  }

  Ok(())
}

#[plugin_hook(CompilationConcatenationScope for EsmLibraryPlugin)]
async fn concatenation_scope(
  &self,
  compilation: &Compilation,
  module: ModuleIdentifier,
) -> Result<Option<ConcatenationScope>> {
  let modules_map = self.concatenated_modules_map_for_codegen.lock().await;
  let modules_map = modules_map
    .get(&compilation.id().0)
    .expect("should has compilation");

  let Some(current_module) = modules_map.get(&module) else {
    return Ok(None);
  };
  let ModuleInfo::Concatenated(current_module) = current_module else {
    return Ok(None);
  };

  Ok(Some(ConcatenationScope::new(
    modules_map.clone(),
    current_module.as_ref().clone(),
  )))
}

#[plugin_hook(CompilationAfterCodeGeneration for EsmLibraryPlugin)]
async fn after_code_generation(&self, compilation: &mut Compilation) -> Result<()> {
  self.link(compilation).await?;
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for EsmLibraryPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let info_map = self.concatenated_modules_map.lock().await;
  let info_map = info_map
    .get(&compilation.id().0)
    .expect("should have compilation info map");

  for m in compilation
    .chunk_graph
    .get_chunk_modules_identifier(chunk_ukey)
  {
    let info = info_map.get(m).expect("should have this info map");
    runtime_requirements.extend(*info.get_runtime_requirements());

    if info.get_interop_default_access_used() {
      runtime_requirements.insert(RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT);
    }

    if info.get_interop_namespace_object2_used() || info.get_interop_namespace_object_used() {
      runtime_requirements.insert(RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT);
    }
  }

  if chunk.has_runtime(&compilation.chunk_group_by_ukey) && !runtime_requirements.is_empty() {
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
  }

  Ok(())
}

static RSPACK_ESM_CHUNK_PLACEHOLDER_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"__RSPACK_ESM_CHUNK_(\d*)").expect("should have regex"));

#[plugin_hook(CompilationProcessAssets for EsmLibraryPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let mut replaced = vec![];

  for (asset_name, asset) in compilation.assets() {
    if asset.get_info().javascript_module.unwrap_or_default() {
      let Some(source) = asset.get_source() else {
        continue;
      };
      let mut replace_source = ReplaceSource::new(source.clone());
      let output_path = compilation.options.output.path.as_std_path();
      let mut self_path = output_path.join(asset_name);

      // only use the path, pop filename
      self_path.pop();

      for captures in RSPACK_ESM_CHUNK_PLACEHOLDER_RE.find_iter(&source.source()) {
        let whole_str = captures.as_str();
        let chunk_ukey = whole_str
          .strip_prefix("__RSPACK_ESM_CHUNK_")
          .expect("should have correct prefix");
        let chunk_ukey =
          ChunkUkey::from(chunk_ukey.parse::<u32>().expect("should have chunk ukey"));
        let start = captures.range().start as u32;
        let end = captures.range().end as u32;
        let chunk = compilation
          .chunk_by_ukey
          .get(&chunk_ukey)
          .expect("should have chunk");

        let chunk_path = output_path.join(
          chunk
            .files()
            .iter()
            .next()
            .expect("at least one path")
            .as_str(),
        );
        let relative = chunk_path.relative(self_path.as_path());
        let import_str = if relative.starts_with(".") {
          relative.to_string_lossy().to_string()
        } else {
          format!("./{}", relative.display())
        };
        replace_source.replace(start, end, &import_str, None);
      }

      replaced.push((asset_name.clone(), replace_source));
    }
  }
  for (replace_name, replace_source) in replaced {
    compilation
      .assets_mut()
      .get_mut(&replace_name)
      .expect("should have asset")
      .set_source(Some(Arc::new(replace_source)));
  }

  Ok(())
}

impl Plugin for EsmLibraryPlugin {
  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));

    ctx
      .context
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));

    ctx
      .context
      .compilation_hooks
      .after_code_generation
      .tap(after_code_generation::new(self));

    ctx
      .context
      .compilation_hooks
      .concatenation_scope
      .tap(concatenation_scope::new(self));

    ctx
      .context
      .compilation_hooks
      .after_seal
      .tap(after_seal::new(self));

    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));

    ctx
      .context
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));

    Ok(())
  }
}
