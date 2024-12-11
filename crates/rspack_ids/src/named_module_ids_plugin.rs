use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use rspack_collections::{IdentifierIndexSet, IdentifierMap, IdentifierSet};
use rspack_core::{
  incremental::{IncrementalPasses, Mutation, Mutations},
  ApplyContext, ChunkGraph, CompilationModuleIds, CompilerOptions, Logger, ModuleGraph, ModuleId,
  ModuleIdentifier, Plugin, PluginContext,
};
use rspack_error::Result;
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
  module_ids: &mut IdentifierMap<ModuleId>,
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
    else if let Some(item) = used_ids.get(name.as_str()) {
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
    if let Some(item) = used_ids.get(name.as_str()) {
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
    } else if items.len() == 1 && !used_ids.contains_key(name.as_str()) {
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
        let mut formatted_name = format!("{name}{}", itoa!(i));
        while name_to_items_keys.contains(&formatted_name)
          && used_ids.contains_key(formatted_name.as_str())
        {
          i += 1;
          formatted_name = format!("{name}{}", itoa!(i));
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
fn module_ids(&self, compilation: &mut rspack_core::Compilation) -> Result<()> {
  let mut module_ids = std::mem::take(&mut compilation.module_ids);
  let mut used_ids: FxHashMap<ModuleId, ModuleIdentifier> = module_ids
    .iter()
    .map(|(&module, id)| (id.clone(), module))
    .collect();
  let module_graph = compilation.get_module_graph();
  let mut modules: IdentifierSet = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::MODULE_IDS)
  {
    mutations
      .iter()
      .rfold(IdentifierSet::default(), |mut acc, mutation| {
        match mutation {
          Mutation::ModuleBuild { module } => {
            acc.insert(*module);
          }
          Mutation::ModuleRemove { module } => {
            // Delete from used_ids even the module is updated module, so we can reuse its module_id
            if let Some(id) = ChunkGraph::get_module_id(&module_ids, *module) {
              used_ids.remove(id);
            }
            // Keep the module_id for updated module (revoke first, then rebuild)
            if !acc.contains(module) {
              module_ids.remove(module);
            }
          }
          _ => {}
        };
        acc
      })
  } else {
    module_graph.modules().keys().copied().collect()
  };

  modules.retain(|m| {
    let m = module_graph
      .module_by_identifier(m)
      .expect("should have module");
    m.need_id()
      && compilation
        .chunk_graph
        .get_number_of_module_chunks(m.identifier())
        != 0
  });
  let modules_len = modules.len();

  let context: &str = compilation.options.context.as_ref();
  let mut mutations = compilation
    .incremental
    .can_write_mutations()
    .then(Mutations::default);

  let unnamed_modules = assign_named_module_ids(
    modules,
    context,
    &module_graph,
    &mut used_ids,
    &mut module_ids,
    &mut mutations,
  );

  let unnamed_modules_len = unnamed_modules.len();
  if !unnamed_modules.is_empty() {
    let mut next_id = 0;
    for module in unnamed_modules {
      let mut id = next_id.to_string();
      while used_ids.contains_key(id.as_str()) {
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
    .can_read_mutations(IncrementalPasses::MODULE_IDS)
    && let Some(mutations) = &mutations
  {
    let logger = compilation.get_logger("rspack.incremental.moduleIds");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules_len,
      module_graph.modules().len(),
    ));
    logger.log(format!(
      "{} modules are updated by set_module_id, with {} unnamed modules",
      mutations.len(),
      unnamed_modules_len,
    ));
  }

  if let Some(compilation_mutations) = compilation.incremental.mutations_write()
    && let Some(mutations) = mutations
  {
    compilation_mutations.extend(mutations);
  }

  compilation.module_ids = module_ids;
  Ok(())
}

impl Plugin for NamedModuleIdsPlugin {
  fn name(&self) -> &'static str {
    "NamedModuleIdsPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .module_ids
      .tap(module_ids::new(self));
    Ok(())
  }
}
