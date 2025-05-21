use std::{
  borrow::Cow,
  collections::hash_map::{Entry, OccupiedEntry},
  path::{Path, PathBuf},
  sync::{Arc, LazyLock},
};

use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use rspack_collections::{
  IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, IdentifierSet, UkeyIndexMap, UkeyMap,
};
use rspack_core::{
  AssetInfo, BoxModule, ChunkLink, ChunkUkey, Compilation,
  CompilationAdditionalChunkRuntimeRequirements, CompilationAfterCodeGeneration,
  CompilationAfterSeal, CompilationConcatenationScope, CompilationFinishModules, CompilationParams,
  CompilationProcessAssets, CompilerCompilation, ConcatenatedModuleIdent, ConcatenatedModuleInfo,
  ConcatenationScope, DependencyId, ExportInfo, ExportInfoProvided, ExternalModuleInfo,
  IdentCollector, Module, ModuleGraph, ModuleGraphConnection, ModuleIdentifier, ModuleInfo, Plugin,
  RuntimeCondition, RuntimeGlobals, SourceType,
  reserved_names::RESERVED_NAMES,
  rspack_sources::{ConcatSource, RawSource, ReplaceSource, Source},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_javascript_compiler::ast::Ast;
use rspack_plugin_javascript::{
  JavascriptModulesRenderChunkContent, JsPlugin, RenderSource, visitors::swc_visitor::resolver,
};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet, FxIndexSet},
};
use sugar_path::SugarPath;
use tokio::sync::Mutex;

#[plugin]
#[derive(Debug, Default)]
pub struct EsmLibraryPlugin {
  pub concatenated_modules_map: Mutex<FxHashMap<u32, Arc<IdentifierIndexMap<ModuleInfo>>>>,
  pub concatenated_modules_map_ref: Mutex<FxHashMap<u32, Arc<IdentifierIndexMap<ModuleInfo>>>>,
}

#[plugin_hook(CompilerCompilation for EsmLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  hooks
    .render_chunk_content
    .tap(render_chunk_content::new(self));
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
    .concatenated_modules_map_ref
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

  for (idx, (module_identifier, module)) in modules.into_iter().enumerate() {
    // make sure all exports are provided
    let exports_info = module_graph.get_exports_info(&module_identifier);

    let mut should_scope_hoisting = true;
    if module.as_normal_module().is_none()
      || module
        .get_concatenation_bailout_reason(&module_graph, &compilation.chunk_graph)
        .is_some()
      || ModuleGraph::is_async(compilation, module_identifier)
      || !module.build_info().strict
    {
      should_scope_hoisting = false;
    }

    if should_scope_hoisting {
      for export_info in exports_info.get_relevant_exports(&module_graph, None) {
        if !matches!(
          export_info.provided(&module_graph),
          Some(ExportInfoProvided::True)
        ) {
          dbg!(module.identifier());
          dbg!(export_info.provided(&module_graph));
          should_scope_hoisting = false;
          break;
        }

        if export_info.is_reexport(&module_graph) && export_info.get_target(&module_graph).is_none()
        {
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
      modules_map.insert(
        *module_identifier,
        ModuleInfo::External(ExternalModuleInfo {
          index: idx,
          module: *module_identifier,
          runtime_condition: if exports_info.is_used(&module_graph, None) {
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
          name: None,
        }),
      );
    }
  }

  let id = compilation.id();

  // only used for scope
  // we mutably modify data in `self.concatenated_modules_map`
  let mut concatenated_modules_map_ref = self.concatenated_modules_map_ref.lock().await;
  concatenated_modules_map_ref.insert(id.0, Arc::new(modules_map.clone()));

  let mut self_modules_map = self.concatenated_modules_map.lock().await;
  self_modules_map.insert(id.0, Arc::new(modules_map));

  Ok(())
}

impl EsmLibraryPlugin {
  fn get_imports(
    m: &Box<dyn Module>,
    module_graph: &ModuleGraph,
  ) -> impl Iterator<Item = (ModuleIdentifier, Vec<DependencyId>)> {
    let mut modules = IdentifierIndexMap::default();
    for dep in m.get_dependencies() {
      let Some(conn) = module_graph.connection_by_dependency_id(dep) else {
        continue;
      };

      if !conn.is_target_active(module_graph, None) {
        continue;
      }

      let Some(dep_module) = module_graph.module_identifier_by_dependency_id(dep) else {
        continue;
      };

      let connections: &mut Vec<DependencyId> = modules.entry(*dep_module).or_default();
      connections.push(conn.dependency_id);
    }

    modules.into_iter()
  }
}

#[plugin_hook(CompilationConcatenationScope for EsmLibraryPlugin)]
async fn concatenation_scope(
  &self,
  compilation: &Compilation,
  module: ModuleIdentifier,
) -> Result<Option<ConcatenationScope>> {
  let modules_map = self.concatenated_modules_map_ref.lock().await;
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
  self.calculate_chunk_relation(compilation).await?;
  self.link(compilation).await
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for EsmLibraryPlugin)]
async fn additional_chunk_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk_link = compilation
    .chunk_graph
    .link
    .as_ref()
    .unwrap()
    .get(chunk_ukey)
    .unwrap();

  let chunk_modules_len = compilation
    .chunk_graph
    .get_chunk_modules_identifier(chunk_ukey)
    .len();

  if chunk_modules_len > chunk_link.hoisted_modules.len() {
    runtime_requirements.insert(RuntimeGlobals::DEFINE_PROPERTY_GETTERS);
    runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES);
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
        let chunk_ukey = whole_str.strip_prefix("__RSPACK_ESM_CHUNK_").unwrap();
        let chunk_ukey = ChunkUkey::from(chunk_ukey.parse::<u32>().unwrap());
        let start = captures.range().start as u32;
        let end = captures.range().end as u32;
        let chunk = compilation.chunk_by_ukey.get(&chunk_ukey).unwrap();

        let chunk_path = output_path.join(chunk.files().iter().next().unwrap().as_str());
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
      .unwrap()
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
