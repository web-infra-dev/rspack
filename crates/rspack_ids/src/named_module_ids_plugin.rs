use rayon::prelude::*;
use rspack_collections::{IdentifierIndexSet, IdentifierSet};
use rspack_core::{
  ChunkGraph, CompilationModuleIds, Logger, ModuleGraph, ModuleId, ModuleIdentifier,
  ModuleIdsArtifact, Plugin,
  chunk_graph_module::{ModuleIdMap, ModuleIdSet},
  incremental::{self, IncrementalPasses, Mutation, Mutations},
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::{comparators::compare_ids, itoa};

use crate::id_helpers::{
  get_long_module_name, get_short_module_name, should_assign_module_id_without_chunk,
};

fn add_conflicted_module_name(
  name_to_item: &mut ModuleIdMap<ModuleIdentifier>,
  conflict_name_to_items: &mut ModuleIdMap<IdentifierIndexSet>,
  name: ModuleId,
  item: ModuleIdentifier,
) {
  if let Some(existing) = name_to_item.remove(&name) {
    conflict_name_to_items
      .entry(name.clone())
      .or_default()
      .insert(existing);
  }
  conflict_name_to_items.entry(name).or_default().insert(item);
}

#[tracing::instrument(skip_all)]
fn assign_named_module_ids(
  modules: IdentifierSet,
  context: &str,
  module_graph: &ModuleGraph,
  used_ids: &mut ModuleIdMap<ModuleIdentifier>,
  module_ids: &mut ModuleIdsArtifact,
  mutations: &mut Option<Mutations>,
) -> Vec<ModuleIdentifier> {
  let item_name_pair: Vec<_> = modules
    .into_par_iter()
    .map(|item| {
      let module = module_graph
        .module_by_identifier(&item)
        .expect("should have module");
      let name = ModuleId::from(get_short_module_name(module, context));
      (item, name)
    })
    .collect();
  // name_to_item keeps the common unique-name path allocation-light.
  // conflict_name_to_items stores only names that need long-name or suffix fallback.
  // name_to_items_keys is built lazily only when suffixed ids must avoid other pending names.
  let mut name_to_item: ModuleIdMap<ModuleIdentifier> = ModuleIdMap::default();
  let mut conflict_name_to_items: ModuleIdMap<IdentifierIndexSet> = ModuleIdMap::default();
  let mut needs_name_to_items_keys = false;
  for (item, name) in item_name_pair {
    if name.as_str().is_empty() {
      add_conflicted_module_name(&mut name_to_item, &mut conflict_name_to_items, name, item);
    } else if let Some(used_item) = used_ids.get(&name) {
      add_conflicted_module_name(
        &mut name_to_item,
        &mut conflict_name_to_items,
        name.clone(),
        item,
      );
      add_conflicted_module_name(
        &mut name_to_item,
        &mut conflict_name_to_items,
        name,
        *used_item,
      );
    } else if conflict_name_to_items.contains_key(&name) {
      add_conflicted_module_name(&mut name_to_item, &mut conflict_name_to_items, name, item);
    } else if let Some(existing) = name_to_item.insert(name.clone(), item) {
      add_conflicted_module_name(
        &mut name_to_item,
        &mut conflict_name_to_items,
        name.clone(),
        existing,
      );
      add_conflicted_module_name(&mut name_to_item, &mut conflict_name_to_items, name, item);
    }
  }

  let item_name_pair: Vec<_> = conflict_name_to_items
    .into_iter()
    .flat_map(|(name, items)| items.into_iter().map(move |item| (name.clone(), item)))
    .par_bridge()
    .map(|(name, item)| {
      let module = module_graph
        .module_by_identifier(&item)
        .expect("should have module");
      let long_name = ModuleId::from(get_long_module_name(name.as_str(), module, context));
      (item, long_name)
    })
    .collect();
  let mut conflict_name_to_items: ModuleIdMap<IdentifierIndexSet> = ModuleIdMap::default();
  for (item, name) in item_name_pair {
    if let Some(used_item) = used_ids.get(&name) {
      add_conflicted_module_name(
        &mut name_to_item,
        &mut conflict_name_to_items,
        name.clone(),
        item,
      );
      add_conflicted_module_name(
        &mut name_to_item,
        &mut conflict_name_to_items,
        name,
        *used_item,
      );
      needs_name_to_items_keys = true;
    } else if conflict_name_to_items.contains_key(&name) {
      add_conflicted_module_name(&mut name_to_item, &mut conflict_name_to_items, name, item);
      needs_name_to_items_keys = true;
    } else if let Some(existing) = name_to_item.insert(name.clone(), item) {
      add_conflicted_module_name(
        &mut name_to_item,
        &mut conflict_name_to_items,
        name.clone(),
        existing,
      );
      add_conflicted_module_name(&mut name_to_item, &mut conflict_name_to_items, name, item);
      needs_name_to_items_keys = true;
    }
  }

  let name_to_items_keys = needs_name_to_items_keys.then(|| {
    let mut keys = name_to_item.keys().cloned().collect::<ModuleIdSet>();
    keys.extend(conflict_name_to_items.keys().cloned());
    keys
  });
  let mut unnamed_items = vec![];

  for (name, item) in name_to_item {
    if name.as_str().is_empty() {
      unnamed_items.push(item)
    } else {
      if ChunkGraph::set_module_id(module_ids, item, name.clone())
        && let Some(mutations) = mutations
      {
        mutations.add(Mutation::ModuleSetId { module: item });
      }
      used_ids.insert(name, item);
    }
  }

  for (name, mut items) in conflict_name_to_items {
    if name.as_str().is_empty() {
      for item in items {
        unnamed_items.push(item)
      }
    } else {
      items.sort_unstable_by(|a, b| compare_ids(a, b));
      let mut i = 0;
      for item in items {
        let mut i_buffer = itoa::Buffer::new();
        let mut formatted_name = ModuleId::from(format!("{name}{}", i_buffer.format(i)));
        // Suffixed ids must skip both ids already assigned and ids that are pending
        // as natural names for another module.
        while used_ids.contains_key(&formatted_name)
          || name_to_items_keys
            .as_ref()
            .is_some_and(|keys| keys.contains(&formatted_name))
        {
          i += 1;
          let mut i_buffer = itoa::Buffer::new();
          formatted_name = ModuleId::from(format!("{name}{}", i_buffer.format(i)));
        }
        if ChunkGraph::set_module_id(module_ids, item, formatted_name.clone())
          && let Some(mutations) = mutations
        {
          mutations.add(Mutation::ModuleSetId { module: item });
        }
        used_ids.insert(formatted_name, item);
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
  let mut used_ids: ModuleIdMap<ModuleIdentifier> = module_ids
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
        && (compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_number_of_module_chunks(*module_identifier)
          != 0
          || should_assign_module_id_without_chunk(module.as_ref()))
    })
    .map(|(m, _)| *m)
    .collect();
  let modules_len = modules.len();

  let context: &str = compilation.options.context.as_ref();
  let mut mutations = compilation
    .incremental
    .mutations_writable()
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
      let mut id = ModuleId::from(next_id.to_string());
      while used_ids.contains_key(&id) {
        next_id += 1;
        id = ModuleId::from(next_id.to_string());
      }
      if ChunkGraph::set_module_id(&mut module_ids, module, id)
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
