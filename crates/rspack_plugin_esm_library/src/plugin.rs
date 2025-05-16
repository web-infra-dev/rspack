use std::{
  collections::hash_map::{Entry, OccupiedEntry},
  sync::Arc,
};

use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rspack_collections::{
  IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, IdentifierSet, UkeyIndexMap, UkeyMap,
};
use rspack_core::{
  BoxModule, ChunkLink, ChunkUkey, Compilation, CompilationAfterCodeGeneration,
  CompilationAfterSeal, CompilationConcatenationScope, CompilationFinishModules, CompilationParams,
  CompilerCompilation, ConcatenatedModuleIdent, ConcatenatedModuleInfo, ConcatenationScope,
  DependencyId, ExportInfo, ExportInfoProvided, ExternalModuleInfo, IdentCollector, Module,
  ModuleGraph, ModuleGraphConnection, ModuleIdentifier, ModuleInfo, PathInfo, Plugin,
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
use swc_core::{
  common::{FileName, SyntaxContext},
  ecma::{
    ast::{EsVersion, Program},
    parser::{Syntax, parse_file_as_module},
  },
};
use tokio::sync::Mutex;

#[plugin]
#[derive(Debug, Default)]
pub struct EsmLibraryPlugin {
  pub concatenated_modules_map: Mutex<FxHashMap<u32, Arc<IdentifierIndexMap<ModuleInfo>>>>,
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
) -> Result<Option<RenderSource>> {
  self.render_chunk(compilation, chunk_ukey).await
}

#[plugin_hook(CompilationAfterSeal for EsmLibraryPlugin)]
async fn after_seal(&self, compilation: &mut Compilation) -> Result<()> {
  self
    .concatenated_modules_map
    .lock()
    .await
    .remove(&compilation.id().0);
  Ok(())
}

#[plugin_hook(CompilationFinishModules for EsmLibraryPlugin)]
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
    if module.as_normal_module().is_none() {
      should_scope_hoisting = false;
    };
    if should_scope_hoisting {
      for export_info in exports_info.exports(&module_graph) {
        if !matches!(
          export_info.provided(&module_graph),
          Some(ExportInfoProvided::True)
        ) {
          should_scope_hoisting = false;
          break;
        };

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
  let modules_map = self.concatenated_modules_map.lock().await;
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
  self.calculate_chunk_relation(compilation).await
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

    Ok(())
  }
}
