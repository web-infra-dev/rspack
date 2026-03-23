use std::{borrow::Cow, fmt::Debug, rc::Rc};

use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  BoxModule, Compilation, CompilationOptimizeDependencies, ConnectionState, DependencyExtraMeta,
  DependencyId, ExportsInfoArtifact, FactoryMeta, GetTargetResult, ImportPhase, Logger, Mode,
  ModuleFactoryCreateData, ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphConnection,
  ModuleIdentifier, NormalModuleCreateData, NormalModuleFactoryModule, Plugin,
  PrefetchExportsInfoMode, RayonConsumer, ResolvedExportInfoTarget, SideEffectsDoOptimize,
  SideEffectsDoOptimizeMoveTarget, SideEffectsOptimizeArtifact,
  build_module_graph::BuildModuleGraphArtifact,
  can_move_target, get_target,
  incremental::{self, IncrementalPasses, Mutation},
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::{AssertUtf8, Utf8Path};
use rustc_hash::FxHashMap;
use sugar_path::SugarPath;
use swc_core::ecma::ast::*;

use crate::dependency::{ESMExportImportedSpecifierDependency, ESMImportSpecifierDependency};

#[derive(Clone, Debug)]
enum SideEffects {
  Bool(bool),
  String(String),
  Array(Vec<String>),
}

type EvaluationPreservingConnections =
  FxHashMap<(ModuleIdentifier, ModuleIdentifier), Vec<DependencyId>>;

impl SideEffects {
  pub fn from_description(description: &serde_json::Value) -> Option<Self> {
    description.get("sideEffects").and_then(|value| {
      if let Some(b) = value.as_bool() {
        Some(SideEffects::Bool(b))
      } else if let Some(s) = value.as_str() {
        Some(SideEffects::String(s.to_owned()))
      } else if let Some(vec) = value.as_array() {
        let mut side_effects = vec![];
        for value in vec {
          if let Some(str) = value.as_str() {
            side_effects.push(str.to_string());
          } else {
            return None;
          }
        }
        Some(SideEffects::Array(side_effects))
      } else {
        None
      }
    })
  }
}

fn get_side_effects_from_package_json(side_effects: SideEffects, relative_path: &Utf8Path) -> bool {
  match side_effects {
    SideEffects::Bool(s) => s,
    SideEffects::String(s) => glob_match_with_normalized_pattern(&s, relative_path.as_str()),
    SideEffects::Array(patterns) => patterns
      .iter()
      .any(|pattern| glob_match_with_normalized_pattern(pattern, relative_path.as_str())),
  }
}

fn glob_match_with_normalized_pattern(pattern: &str, string: &str) -> bool {
  let trim_start = pattern.trim_start_matches("./");
  let normalized_glob = if trim_start.contains('/') {
    trim_start.to_string()
  } else {
    String::from("**/") + trim_start
  };
  fast_glob::glob_match(&normalized_glob, string.trim_start_matches("./"))
}

fn is_retargetable_dependency(dependency_id: &DependencyId, module_graph: &ModuleGraph) -> bool {
  let dep = module_graph.dependency_by_id(dependency_id);
  if dep.is::<ESMExportImportedSpecifierDependency>() {
    return true;
  }

  dep
    .downcast_ref::<ESMImportSpecifierDependency>()
    .is_some_and(|dep| !dep.namespace_object_as_context && !dep.get_ids(module_graph).is_empty())
}

fn is_evaluation_preserving_connection(
  connection: &ModuleGraphConnection,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> bool {
  if !connection
    .active_state(
      module_graph,
      None,
      module_graph_cache,
      exports_info_artifact,
    )
    .is_true()
  {
    return false;
  }

  let dep = module_graph.dependency_by_id(&connection.dependency_id);
  if dep.lazy().is_some() || dep.get_phase() != ImportPhase::Evaluation {
    return false;
  }

  dep
    .get_module_evaluation_side_effects_state(
      module_graph,
      module_graph_cache,
      &mut Default::default(),
      &mut Default::default(),
    )
    .is_true()
}

fn get_side_effects_state_map(
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
) -> IdentifierMap<ConnectionState> {
  module_graph
    .modules_par()
    .map(|(module_identifier, module)| {
      (
        *module_identifier,
        module.get_side_effects_connection_state(
          module_graph,
          module_graph_cache,
          &mut Default::default(),
          &mut Default::default(),
        ),
      )
    })
    .collect()
}

fn get_evaluation_preserving_connections(
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
  side_effects_state_map: &IdentifierMap<ConnectionState>,
) -> EvaluationPreservingConnections {
  let mut connections = EvaluationPreservingConnections::default();

  for (original_module, _) in module_graph.modules() {
    if !side_effects_state_map[original_module].is_true() {
      continue;
    }

    for connection in module_graph.get_outgoing_connections(original_module) {
      if is_evaluation_preserving_connection(
        connection,
        module_graph,
        module_graph_cache,
        exports_info_artifact,
      ) {
        connections
          .entry((*original_module, *connection.module_identifier()))
          .or_default()
          .push(connection.dependency_id);
      }
    }
  }

  connections
}

fn is_evaluation_preserved(
  original_module: ModuleIdentifier,
  target_module: ModuleIdentifier,
  excluded_dependency: DependencyId,
  evaluation_preserving_connections: &EvaluationPreservingConnections,
) -> bool {
  evaluation_preserving_connections
    .get(&(original_module, target_module))
    .is_some_and(|dependencies| dependencies.iter().any(|dep| dep != &excluded_dependency))
}

fn can_skip_target_module(
  original_module: ModuleIdentifier,
  target_module: ModuleIdentifier,
  excluded_dependency: DependencyId,
  side_effects_state_map: &IdentifierMap<ConnectionState>,
  evaluation_preserving_connections: &EvaluationPreservingConnections,
) -> bool {
  if side_effects_state_map[&target_module] == ConnectionState::Active(false) {
    return true;
  }

  is_evaluation_preserved(
    original_module,
    target_module,
    excluded_dependency,
    evaluation_preserving_connections,
  )
}

fn is_self_side_effect_free_module(
  module_identifier: &ModuleIdentifier,
  module_graph: &ModuleGraph,
) -> bool {
  let Some(module) = module_graph.module_by_identifier(module_identifier) else {
    return false;
  };

  if let Some(side_effect_free) = module.factory_meta().and_then(|meta| meta.side_effect_free) {
    return side_effect_free;
  }

  module.build_meta().side_effect_free == Some(true)
}

fn can_skip_resolved_target(
  target: &ResolvedExportInfoTarget,
  side_effects_state_map: &IdentifierMap<ConnectionState>,
  evaluation_preserving_connections: &EvaluationPreservingConnections,
  module_graph: &ModuleGraph,
) -> bool {
  if is_side_effect_free_target(target, side_effects_state_map) {
    return true;
  }

  if !is_self_side_effect_free_module(&target.module, module_graph) {
    return false;
  }

  let Some(original_module) = module_graph.get_parent_module(&target.dependency).copied() else {
    return false;
  };

  is_evaluation_preserved(
    original_module,
    target.module,
    target.dependency,
    evaluation_preserving_connections,
  )
}

fn is_side_effect_free_target(
  target: &ResolvedExportInfoTarget,
  side_effects_state_map: &IdentifierMap<ConnectionState>,
) -> bool {
  side_effects_state_map[&target.module] == ConnectionState::Active(false)
}

fn has_compatible_target_phase(
  source_dependency_id: DependencyId,
  target: &ResolvedExportInfoTarget,
  module_graph: &ModuleGraph,
) -> bool {
  module_graph
    .dependency_by_id(&source_dependency_id)
    .get_phase()
    == module_graph
      .dependency_by_id(&target.dependency)
      .get_phase()
}

fn refresh_do_optimizes(
  do_optimizes: &SideEffectsOptimizeArtifact,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
  enable_preserved_evaluation_retarget: bool,
) -> (SideEffectsOptimizeArtifact, Option<DependencyId>) {
  let side_effects_state_map = get_side_effects_state_map(module_graph, module_graph_cache);
  let evaluation_preserving_connections = if enable_preserved_evaluation_retarget {
    get_evaluation_preserving_connections(
      module_graph,
      module_graph_cache,
      exports_info_artifact,
      &side_effects_state_map,
    )
  } else {
    Default::default()
  };
  let mut refreshed_do_optimizes = SideEffectsOptimizeArtifact::default();
  let mut serial_dependency = None;

  let mut dependency_ids: Vec<_> = do_optimizes.keys().copied().collect();
  dependency_ids.sort_unstable();

  for dependency_id in dependency_ids {
    let Some(connection) = module_graph.connection_by_dependency_id(&dependency_id) else {
      continue;
    };

    let Some(do_optimize) = can_optimize_connection(
      connection,
      &side_effects_state_map,
      &evaluation_preserving_connections,
      module_graph,
      exports_info_artifact,
    ) else {
      continue;
    };

    if serial_dependency.is_none()
      && side_effects_state_map[connection.module_identifier()] != ConnectionState::Active(false)
    {
      serial_dependency = Some(dependency_id);
    }

    refreshed_do_optimizes.insert(dependency_id, do_optimize);
  }

  (refreshed_do_optimizes, serial_dependency)
}

pub trait ClassExt {
  fn class_key(&self) -> Option<&PropName>;
  fn is_static(&self) -> bool;
}

impl ClassExt for ClassMember {
  fn class_key(&self) -> Option<&PropName> {
    match self {
      ClassMember::Constructor(c) => Some(&c.key),
      ClassMember::Method(m) => Some(&m.key),
      ClassMember::PrivateMethod(_) => None,
      ClassMember::ClassProp(c) => Some(&c.key),
      ClassMember::PrivateProp(_) => None,
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => None,
      ClassMember::StaticBlock(_) => None,
      ClassMember::AutoAccessor(a) => match a.key {
        Key::Private(_) => None,
        Key::Public(ref public) => Some(public),
      },
    }
  }

  fn is_static(&self) -> bool {
    match self {
      ClassMember::Constructor(_cons) => false,
      ClassMember::Method(m) => m.is_static,
      ClassMember::PrivateMethod(m) => m.is_static,
      ClassMember::ClassProp(p) => p.is_static,
      ClassMember::PrivateProp(p) => p.is_static,
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => false,
      ClassMember::StaticBlock(_) => true,
      ClassMember::AutoAccessor(a) => a.is_static,
    }
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct SideEffectsFlagPlugin;

#[plugin_hook(NormalModuleFactoryModule for SideEffectsFlagPlugin,tracing=false)]
async fn nmf_module(
  &self,
  _data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  if let Some(has_side_effects) = create_data.side_effects {
    module.set_factory_meta(FactoryMeta {
      side_effect_free: Some(!has_side_effects),
    });
    return Ok(());
  }
  let resource_data = &create_data.resource_resolve_data;
  let Some(resource_path) = resource_data.path() else {
    return Ok(());
  };
  let Some(description) = resource_data.description() else {
    return Ok(());
  };
  let package_path = description.path();
  let Some(side_effects) = SideEffects::from_description(description.json()) else {
    return Ok(());
  };
  let relative_path = resource_path
    .as_std_path()
    .relative(package_path)
    .assert_utf8();
  let has_side_effects = get_side_effects_from_package_json(side_effects, relative_path.as_path());
  module.set_factory_meta(FactoryMeta {
    side_effect_free: Some(!has_side_effects),
  });
  Ok(())
}

#[plugin_hook(CompilationOptimizeDependencies for SideEffectsFlagPlugin,tracing=false)]
async fn optimize_dependencies(
  &self,
  compilation: &Compilation,
  side_effects_optimize_artifact: &mut SideEffectsOptimizeArtifact,
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  let logger = compilation.get_logger("rspack.SideEffectsFlagPlugin");
  let start = logger.time("update connections");

  let module_graph = build_module_graph_artifact.get_module_graph();
  let enable_preserved_evaluation_retarget = matches!(compilation.options.mode, Mode::Production);

  let side_effects_state_map =
    get_side_effects_state_map(module_graph, &compilation.module_graph_cache_artifact);
  let evaluation_preserving_connections = if enable_preserved_evaluation_retarget {
    get_evaluation_preserving_connections(
      module_graph,
      &compilation.module_graph_cache_artifact,
      exports_info_artifact,
      &side_effects_state_map,
    )
  } else {
    Default::default()
  };

  let inner_start = logger.time("prepare connections");
  let modules: IdentifierSet = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::OPTIMIZE_DEPENDENCIES)
    && !side_effects_optimize_artifact.is_empty()
  {
    side_effects_optimize_artifact.retain(|dependency_id, do_optimize| {
      let dep_exist = module_graph
        .connection_by_dependency_id(dependency_id)
        .is_some();
      let target_module_exist = module_graph
        .module_by_identifier(&do_optimize.target_module)
        .is_some();
      dep_exist && target_module_exist
    });

    fn affected_incoming_modules(
      module: &ModuleIdentifier,
      module_graph: &ModuleGraph,
      modules: &mut IdentifierSet,
    ) {
      for connection in module_graph.get_incoming_connections(module) {
        let Some(original_module) = connection.original_module_identifier else {
          continue;
        };
        if modules.contains(&original_module) {
          continue;
        }
        if is_retargetable_dependency(&connection.dependency_id, module_graph)
          && modules.insert(original_module)
        {
          affected_incoming_modules(&original_module, module_graph, modules);
        }
      }
    }

    let modules: IdentifierSet = mutations.iter().fold(
      IdentifierSet::default(),
      |mut modules, mutation| match mutation {
        Mutation::ModuleAdd { module } | Mutation::ModuleUpdate { module } => {
          if modules.insert(*module) {
            affected_incoming_modules(module, module_graph, &mut modules);
          }
          modules.extend(
            module_graph
              .get_outgoing_connections(module)
              .map(|connection| *connection.module_identifier()),
          );
          modules
        }
        _ => modules,
      },
    );

    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::OPTIMIZE_DEPENDENCIES, %mutations, ?modules);
    let logger = compilation.get_logger("rspack.incremental.optimizeDependencies");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules.len(),
      module_graph.modules_len()
    ));

    modules
  } else {
    module_graph.modules_keys().copied().collect()
  };
  logger.time_end(inner_start);

  let inner_start = logger.time("find optimizable connections");
  modules
    .par_iter()
    .flat_map(|module| {
      module_graph
        .get_outgoing_connections(module)
        .collect::<Vec<_>>()
    })
    .map(|connection| {
      (
        connection.dependency_id,
        can_optimize_connection(
          connection,
          &side_effects_state_map,
          &evaluation_preserving_connections,
          module_graph,
          exports_info_artifact,
        ),
      )
    })
    .consume(|(dep_id, can_optimize)| {
      if let Some(do_optimize) = can_optimize {
        side_effects_optimize_artifact.insert(dep_id, do_optimize);
      } else {
        side_effects_optimize_artifact.remove(&dep_id);
      }
    });
  logger.time_end(inner_start);

  let mut pending_optimizes = side_effects_optimize_artifact.clone();

  let inner_start = logger.time("do optimize connections");
  let mut do_optimized_count = 0;
  while !pending_optimizes.is_empty() {
    let module_graph = build_module_graph_artifact.get_module_graph();
    let (do_optimizes, serial_dependency) = refresh_do_optimizes(
      &pending_optimizes,
      module_graph,
      &compilation.module_graph_cache_artifact,
      exports_info_artifact,
      enable_preserved_evaluation_retarget,
    );
    if do_optimizes.is_empty() {
      break;
    }

    pending_optimizes = do_optimizes.clone();
    let module_graph = build_module_graph_artifact.get_module_graph_mut();

    if let Some(serial_dependency) = serial_dependency {
      // Preserved-evaluation rewrites can invalidate sibling candidates in the same source module.
      do_optimized_count += 1;
      let do_optimize = do_optimizes
        .get(&serial_dependency)
        .expect("serial dependency should exist")
        .clone();
      do_optimize_connection(
        serial_dependency,
        do_optimize,
        module_graph,
        exports_info_artifact,
      );
    } else {
      do_optimized_count += do_optimizes.len();
      for (dependency, do_optimize) in do_optimizes {
        do_optimize_connection(dependency, do_optimize, module_graph, exports_info_artifact);
      }
    }
  }
  logger.time_end(inner_start);

  logger.time_end(start);
  logger.log(format!("optimized {do_optimized_count} connections"));
  Ok(None)
}

#[tracing::instrument(skip_all)]
fn do_optimize_connection(
  dependency: DependencyId,
  do_optimize: SideEffectsDoOptimize,
  module_graph: &mut ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
) -> (DependencyId, ModuleIdentifier) {
  let SideEffectsDoOptimize {
    ids,
    target_module,
    need_move_target,
  } = do_optimize;
  module_graph.do_update_module(&dependency, &target_module);
  module_graph.set_dependency_extra_meta(
    dependency,
    DependencyExtraMeta {
      ids,
      explanation: Some("(skipped side-effect-free modules)"),
    },
  );
  if let Some(SideEffectsDoOptimizeMoveTarget {
    export_info,
    target_export,
  }) = need_move_target
  {
    export_info
      .as_data_mut(exports_info_artifact)
      .do_move_target(dependency, target_export);
  }
  (dependency, target_module)
}

#[tracing::instrument("can_optimize_connection", level = "trace", skip_all)]
fn can_optimize_connection(
  connection: &ModuleGraphConnection,
  side_effects_state_map: &IdentifierMap<ConnectionState>,
  evaluation_preserving_connections: &EvaluationPreservingConnections,
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<SideEffectsDoOptimize> {
  let original_module = connection.original_module_identifier?;
  let dependency_id = connection.dependency_id;

  if !can_skip_target_module(
    original_module,
    *connection.module_identifier(),
    dependency_id,
    side_effects_state_map,
    evaluation_preserving_connections,
  ) {
    return None;
  }

  let dep = module_graph.dependency_by_id(&dependency_id);

  if dep.get_phase() != ImportPhase::Evaluation {
    return None;
  }

  if let Some(dep) = dep.downcast_ref::<ESMExportImportedSpecifierDependency>()
    && let Some(name) = &dep.name
  {
    let exports_info = exports_info_artifact
      .get_prefetched_exports_info(&original_module, PrefetchExportsInfoMode::Default);
    let export_info = exports_info.get_export_info_without_mut_module_graph(name);

    let target = can_move_target(
      &export_info,
      module_graph,
      exports_info_artifact,
      Rc::new(|target: &ResolvedExportInfoTarget| {
        can_skip_resolved_target(
          target,
          side_effects_state_map,
          evaluation_preserving_connections,
          module_graph,
        )
      }),
    )?;
    if !is_side_effect_free_target(&target, side_effects_state_map) {
      return None;
    }
    if !has_compatible_target_phase(dependency_id, &target, module_graph) {
      return None;
    }
    if !module_graph.can_update_module(&dependency_id, &target.module) {
      return None;
    }

    let ids = dep.get_ids(module_graph);
    let processed_ids = target.export.as_ref().map_or_else(
      || ids.get(1..).unwrap_or_default().to_vec(),
      |item| {
        let mut ret = item.clone();
        ret.extend_from_slice(ids.get(1..).unwrap_or_default());
        ret
      },
    );
    let need_move_target = match export_info {
      Cow::Borrowed(export_info) => Some(SideEffectsDoOptimizeMoveTarget {
        export_info: export_info.id(),
        target_export: target.export,
      }),
      Cow::Owned { .. } => None,
    };

    return Some(SideEffectsDoOptimize {
      ids: processed_ids,
      target_module: target.module,
      need_move_target,
    });
  }

  if let Some(dep) = dep.downcast_ref::<ESMImportSpecifierDependency>()
    && !dep.namespace_object_as_context
    && let ids = dep.get_ids(module_graph)
    && !ids.is_empty()
  {
    let exports_info = exports_info_artifact.get_prefetched_exports_info(
      connection.module_identifier(),
      PrefetchExportsInfoMode::Default,
    );
    let export_info = exports_info.get_export_info_without_mut_module_graph(&ids[0]);

    let Some(GetTargetResult::Target(target)) = get_target(
      &export_info,
      module_graph,
      exports_info_artifact,
      Rc::new(|target: &ResolvedExportInfoTarget| {
        can_skip_resolved_target(
          target,
          side_effects_state_map,
          evaluation_preserving_connections,
          module_graph,
        )
      }),
      &mut Default::default(),
    ) else {
      return None;
    };
    if !is_side_effect_free_target(&target, side_effects_state_map) {
      return None;
    }
    if !has_compatible_target_phase(dependency_id, &target, module_graph) {
      return None;
    }

    if !module_graph.can_update_module(&dependency_id, &target.module) {
      return None;
    }

    let processed_ids = target.export.map_or_else(
      || ids[1..].to_vec(),
      |mut item| {
        item.extend_from_slice(&ids[1..]);
        item
      },
    );

    return Some(SideEffectsDoOptimize {
      ids: processed_ids,
      target_module: target.module,
      need_move_target: None,
    });
  }

  None
}

impl Plugin for SideEffectsFlagPlugin {
  fn name(&self) -> &'static str {
    "SideEffectsFlagPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .module
      .tap(nmf_module::new(self));
    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));
    Ok(())
  }
}

#[cfg(test)]
mod test_side_effects {
  use super::*;

  fn get_side_effects_from_package_json_helper(
    side_effects_config: Vec<&str>,
    relative_path: &str,
  ) -> bool {
    assert!(!side_effects_config.is_empty());
    let relative_path = Utf8Path::new(relative_path);
    let side_effects = if side_effects_config.len() > 1 {
      SideEffects::Array(
        side_effects_config
          .into_iter()
          .map(String::from)
          .collect::<Vec<_>>(),
      )
    } else {
      SideEffects::String((&side_effects_config[0]).to_string())
    };

    get_side_effects_from_package_json(side_effects, relative_path)
  }

  #[test]
  fn cases() {
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/*.js"],
      "./src/x/y/z.js"
    ));
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/index.js", "./src/selection/index.js"],
      "./src/selection/index.js"
    ));
    assert!(!get_side_effects_from_package_json_helper(
      vec!["./src/**/*.js"],
      "./x.js"
    ));
    assert!(get_side_effects_from_package_json_helper(
      vec!["./**/src/x/y/z.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"./src/**/z.js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/z.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"./**/x/**/z.js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["./**/x/**/z.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"./**/src/**",
    assert!(get_side_effects_from_package_json_helper(
      vec!["./**/src/**"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"./**/src/*",
    assert!(!get_side_effects_from_package_json_helper(
      vec!["./src/x/y/z.js"],
      "./**/src/*"
    ));
    // 				"./src/x/y/z.js",
    // 				"*.js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["*.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"x/**/z.js",
    assert!(!get_side_effects_from_package_json_helper(
      vec!["./src/x/y/z.js"],
      "x/**/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"src/**/z.js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/z.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"src/**/{x,y,z}.js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["src/**/{x,y,z}.js"],
      "./src/x/y/z.js"
    ));
    // 				"./src/x/y/z.js",
    // 				"src/**/[x-z].js",
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/[x-z].js"],
      "./src/x/y/z.js"
    ));
    // 		const array = ["./src/**/*.js", "./dirty.js"];
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/*.js", "./dirty.js"],
      "./src/x/y/z.js"
    ));
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/*.js", "./dirty.js"],
      "./dirty.js"
    ));
    assert!(!get_side_effects_from_package_json_helper(
      vec!["./src/**/*.js", "./dirty.js"],
      "./clean.js"
    ));
    assert!(get_side_effects_from_package_json_helper(
      vec!["./src/**/*/z.js"],
      "./src/x/y/z.js"
    ));
  }
}
