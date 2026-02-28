use rayon::prelude::*;
use rspack_collections::{IdentifierIndexSet, IdentifierSet};
use rspack_core::{
  ChunkGraph, CompilationModuleIds, Logger, ModuleGraph, ModuleId, ModuleIdentifier,
  ModuleIdsArtifact, Plugin,
  incremental::{self, IncrementalPasses, Mutation, Mutations},
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::{comparators::compare_ids, itoa};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::id_helpers::{get_long_module_name, get_short_module_name};

#[tracing::instrument(skip_all)]
fn assign_named_module_ids(
  modules: IdentifierSet,
  context: &str,
  module_graph: &ModuleGraph,
  used_ids: &mut FxHashMap<ModuleId, ModuleIdentifier>,
  module_ids: &mut ModuleIdsArtifact,
  mutations: &mut Option<Mutations>,
) -> Vec<ModuleIdentifier> {
  let item_name_pair: Vec<_> = modules
    .into_par_iter()
    .map(|item| {
      let module = module_graph
        .module_by_identifier(&item)
        .expect("should have module");
      let name = get_short_module_name(module, context);
      (item, name)
    })
    .collect();
  let mut name_to_items: FxHashMap<String, IdentifierIndexSet> = FxHashMap::default();
  let mut invalid_and_repeat_names: FxHashSet<String> = std::iter::once(String::new()).collect();
  for (item, name) in item_name_pair {
    let items = name_to_items.entry(name.clone()).or_default();
    items.insert(item);
    // If the short module id is conflict, then we need to rename all the conflicting modules to long module id
    if items.len() > 1 {
      invalid_and_repeat_names.insert(name);
    }
    // Also rename the conflicting modules in used_ids
    else if let Some(item) = used_ids.get(&name.as_str().into()) {
      items.insert(*item);
      invalid_and_repeat_names.insert(name);
    }
  }

  let item_name_pair: Vec<_> = invalid_and_repeat_names
    .iter()
    .flat_map(|name| {
      let mut res = vec![];
      for item in name_to_items.remove(name).unwrap_or_default() {
        res.push((name.clone(), item));
      }
      res
    })
    .par_bridge()
    .map(|(name, item)| {
      let module = module_graph
        .module_by_identifier(&item)
        .expect("should have module");
      let long_name = get_long_module_name(&name, module, context);
      (item, long_name)
    })
    .collect();
  for (item, name) in item_name_pair {
    let items = name_to_items.entry(name.clone()).or_default();
    items.insert(item);
    // Also rename the conflicting modules in used_ids
    if let Some(item) = used_ids.get(&name.as_str().into()) {
      items.insert(*item);
    }
  }

  let name_to_items_keys = name_to_items.keys().cloned().collect::<FxHashSet<_>>();
  let mut unnamed_items = vec![];

  for (name, mut items) in name_to_items {
    if name.is_empty() {
      for item in items {
        unnamed_items.push(item)
      }
    } else if items.len() == 1 && !used_ids.contains_key(&name.as_str().into()) {
      let item = items[0];
      let name: ModuleId = name.into();
      if ChunkGraph::set_module_id(module_ids, item, name.clone())
        && let Some(mutations) = mutations
      {
        mutations.add(Mutation::ModuleSetId { module: item });
      }
      used_ids.insert(name, item);
    } else {
      items.sort_unstable_by(|a, b| compare_ids(a, b));
      let mut i = 0;
      for item in items {
        let mut i_buffer = itoa::Buffer::new();
        let mut formatted_name = format!("{name}{}", i_buffer.format(i));
        while name_to_items_keys.contains(&formatted_name)
          && used_ids.contains_key(&formatted_name.as_str().into())
        {
          i += 1;
          let mut i_buffer = itoa::Buffer::new();
          formatted_name = format!("{name}{}", i_buffer.format(i));
        }
        let name: ModuleId = formatted_name.into();
        if ChunkGraph::set_module_id(module_ids, item, name.clone())
          && let Some(mutations) = mutations
        {
          mutations.add(Mutation::ModuleSetId { module: item });
        }
        used_ids.insert(name, item);
        i += 1;
      }
    }
  }
  unnamed_items.sort_unstable_by(|a, b| compare_ids(a, b));
  unnamed_items
}

#[plugin]
#[derive(Debug, Default)]
pub struct NamedModuleIdsPlugin;

#[plugin_hook(CompilationModuleIds for NamedModuleIdsPlugin)]
async fn module_ids(
  &self,
  compilation: &rspack_core::Compilation,
  module_ids_artifact: &mut ModuleIdsArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<()> {
  let mut module_ids = std::mem::take(module_ids_artifact);
  let mut used_ids: FxHashMap<ModuleId, ModuleIdentifier> = module_ids
    .iter()
    .map(|(&module, id)| (id.clone(), module))
    .collect();
  let module_graph = compilation.get_module_graph();
  if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::MODULE_IDS)
    && !module_ids.is_empty()
  {
    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::MODULE_IDS, %mutations);
    mutations.iter().for_each(|mutation| {
      match mutation {
        Mutation::ModuleUpdate { module } => {
          // Delete from used_ids even the module is updated module, so we can reuse its module_id
          if let Some(id) = ChunkGraph::get_module_id(&module_ids, *module) {
            used_ids.remove(id);
          }
        }
        Mutation::ModuleRemove { module } => {
          if let Some(id) = ChunkGraph::get_module_id(&module_ids, *module) {
            used_ids.remove(id);
          }
          module_ids.remove(module);
        }
        _ => {}
      }
    });
  }

  let modules: IdentifierSet = module_graph
    .modules()
    .filter(|&(module_identifier, module)| {
      let not_used =
        if let Some(module_id) = ChunkGraph::get_module_id(&module_ids, *module_identifier) {
          !used_ids.contains_key(module_id)
        } else {
          true
        };
      not_used
        && module.need_id()
        && compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_number_of_module_chunks(*module_identifier)
          != 0
    })
    .map(|(m, _)| *m)
    .collect();
  let modules_len = modules.len();

  let context: &str = compilation.options.context.as_ref();
  let mut mutations = compilation
    .incremental
    .mutations_writeable()
    .then(Mutations::default);

  let unnamed_modules = assign_named_module_ids(
    modules,
    context,
    module_graph,
    &mut used_ids,
    &mut module_ids,
    &mut mutations,
  );

  let unnamed_modules_len = unnamed_modules.len();
  if !unnamed_modules.is_empty() {
    let mut next_id = 0;
    for module in unnamed_modules {
      let mut id = next_id.to_string();
      while used_ids.contains_key(&id.as_str().into()) {
        next_id += 1;
        id = next_id.to_string();
      }
      if ChunkGraph::set_module_id(&mut module_ids, module, id.into())
        && let Some(mutations) = &mut mutations
      {
        mutations.add(Mutation::ModuleSetId { module });
      }
      next_id += 1;
    }
  }

  if compilation
    .incremental
    .mutations_readable(IncrementalPasses::MODULE_IDS)
    && let Some(mutations) = &mutations
  {
    let logger = compilation.get_logger("rspack.incremental.moduleIds");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules_len,
      module_graph.modules_len(),
    ));
    logger.log(format!(
      "{} modules are updated by set_module_id, with {} unnamed modules",
      mutations.len(),
      unnamed_modules_len,
    ));
  }

  if let Some(mut compilation_mutations) = compilation.incremental.mutations_write()
    && let Some(mutations) = mutations
  {
    compilation_mutations.extend(mutations);
  }

  *module_ids_artifact = module_ids;
  Ok(())
}

impl Plugin for NamedModuleIdsPlugin {
  fn name(&self) -> &'static str {
    "NamedModuleIdsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.module_ids.tap(module_ids::new(self));
    Ok(())
  }
}
