use std::{hash::BuildHasherDefault, rc::Rc};

use rayon::prelude::*;
use rspack_collections::{IdentifierHasher, IdentifierMap};
use rspack_core::{
  ApplyContext, AsyncModulesArtifact, Compilation, CompilationFinishModules, Dependency,
  DependencyId, GetTargetResult, InnerGraphMapValue, InnerGraphState, InnerGraphUsageOperation,
  Plugin, PrefetchExportsInfoMode, get_target,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::SpanExt;
use rustc_hash::FxHashMap;
use swc_core::common::BytePos;

use crate::{
  InnerGraphParserPlugin,
  dependency::{ESMImportSpecifierDependency, PureExpressionDependency, URLDependency},
  parser_and_generator::JavaScriptParserAndGenerator,
};

#[plugin]
#[derive(Debug, Default)]
pub struct InnerGraphPlugin;

#[plugin_hook(CompilationFinishModules for InnerGraphPlugin)]
async fn finish_modules(
  &self,
  compilation: &mut Compilation,
  _async_modules_artifact: &mut AsyncModulesArtifact,
) -> Result<()> {
  let module_graph = compilation.get_module_graph_mut();
  let modules = module_graph.modules().keys().copied().collect::<Vec<_>>();
  let mut inner_graph_states: IdentifierMap<InnerGraphState> =
    IdentifierMap::with_capacity_and_hasher(
      modules.len(),
      BuildHasherDefault::<IdentifierHasher>::default(),
    );

  for module_id in modules {
    let module = module_graph
      .module_by_identifier_mut(&module_id)
      .expect("should have module");
    let Some(normal_module) = module.as_normal_module_mut() else {
      continue;
    };

    let Some(parser) = normal_module
      .parser_and_generator_mut()
      .downcast_mut::<JavaScriptParserAndGenerator>()
    else {
      continue;
    };

    if !parser.inner_graph.is_enabled() {
      continue;
    }

    let state = std::mem::take(&mut parser.inner_graph);
    inner_graph_states.insert(module_id, state);
  }

  let module_graph = compilation.get_module_graph();

  let finalized_vec = inner_graph_states
    .par_iter_mut()
    .map(|(module_identifier, state)| {
      let module = module_graph
        .module_by_identifier(module_identifier)
        .expect("should have module");
      // Build map of (start, end) -> DependencyId for ESMImportSpecifierDependency
      let dep_by_span: FxHashMap<(BytePos, BytePos), DependencyId> = module
        .get_dependencies()
        .iter()
        .filter_map(|dep_id| {
          let dep = module_graph.dependency_by_id(dep_id);
          let specifier_dep = dep.downcast_ref::<ESMImportSpecifierDependency>()?;
          let range = specifier_dep.range()?;
          Some(((BytePos(range.start), BytePos(range.end)), *dep_id))
        })
        .collect();

      // Cross-module pure function analysis
      let mut used_symbol = vec![];
      for (symbol, symbol_data) in state.symbol_map.iter() {
        for (name, span) in &symbol_data.depend_on_pure {
          // Find dependency by span
          if let Some(dep_id) = dep_by_span.get(&(BytePos(span.real_lo()), BytePos(span.real_hi())))
          // Resolve dependency to module
          && let Some(ref_module) = module_graph.module_identifier_by_dependency_id(dep_id)
          {
            let target_exports_info = module_graph
              .get_prefetched_exports_info(ref_module, PrefetchExportsInfoMode::Default);
            let target_export_info =
              target_exports_info.get_export_info_without_mut_module_graph(name);
            let (ref_module_id, atom) = if let Some(GetTargetResult::Target(target)) = get_target(
              &target_export_info,
              module_graph,
              Rc::new(|_| true),
              &mut Default::default(),
            ) && let Some(export) = &target.export
              && let Some(atom) = export.first()
            {
              (target.module, atom.clone())
            } else {
              (*ref_module, name.clone())
            };

            let ref_module = module_graph
              .module_by_identifier(&ref_module_id)
              .expect("should have module");
            let Some(side_effects_free) = &ref_module.build_info().side_effects_free else {
              used_symbol.push(symbol);
              break;
            };
            if !side_effects_free.contains(&atom) {
              used_symbol.push(symbol);
              break;
            }
          } else {
            // depend on in_pure callee, just make it used
            used_symbol.push(symbol);
            break;
          }
        }
      }

      for symbol in used_symbol {
        state.inner_graph.insert(*symbol, InnerGraphMapValue::True);
      }

      InnerGraphParserPlugin::infer_dependency_usage(state)
    })
    .collect::<Vec<_>>();

  let module_graph = compilation.get_module_graph_mut();

  for finalized in finalized_vec {
    for (op, used_by_exports) in finalized {
      match op {
        InnerGraphUsageOperation::PureExpression(dep_id) => {
          let dep = module_graph.dependency_by_id_mut(&dep_id);
          if let Some(dep) = dep.downcast_mut::<PureExpressionDependency>() {
            dep.set_used_by_exports(Some(used_by_exports));
          }
        }
        InnerGraphUsageOperation::ESMImportSpecifier(dep_id) => {
          let dep = module_graph.dependency_by_id_mut(&dep_id);
          if let Some(dep) = dep.downcast_mut::<ESMImportSpecifierDependency>() {
            dep.set_used_by_exports(Some(used_by_exports));
          }
        }
        InnerGraphUsageOperation::URLDependency(dep_id) => {
          let dep = module_graph.dependency_by_id_mut(&dep_id);
          if let Some(dep) = dep.downcast_mut::<URLDependency>() {
            dep.set_used_by_exports(Some(used_by_exports));
          }
        }
      }
    }
  }

  Ok(())
}

impl Plugin for InnerGraphPlugin {
  fn name(&self) -> &'static str {
    "InnerGraphPlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    Ok(())
  }
}
