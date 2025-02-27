use std::collections::HashSet as RawHashSet;
use std::hash::{BuildHasherDefault, Hash};
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use indexmap::{IndexMap as RawIndexMap, IndexSet as RawIndexSet};
use itertools::Itertools;
use num_bigint::BigUint;
use rspack_collections::{
  impl_item_ukey, Database, DatabaseItem, IdentifierHasher, Ukey, UkeyIndexMap, UkeyIndexSet,
  UkeyMap, UkeySet,
};
use rspack_collections::{IdentifierIndexSet, IdentifierMap};
use rspack_error::{error, Diagnostic, Error, Result};
use rspack_util::itoa;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};

use super::incremental::ChunkCreateData;
use crate::dependencies_block::AsyncDependenciesToInitialChunkError;
use crate::incremental::{IncrementalPasses, Mutation};
use crate::{
  assign_depths, get_entry_runtime, merge_runtime, AsyncDependenciesBlockIdentifier, ChunkGroup,
  ChunkGroupKind, ChunkGroupOptions, ChunkGroupUkey, ChunkLoading, ChunkUkey, Compilation,
  ConnectionState, DependenciesBlock, DependencyId, DependencyLocation, EntryDependency,
  EntryRuntime, GroupOptions, Logger, ModuleDependency, ModuleGraph, ModuleIdentifier, RuntimeSpec,
  SyntheticDependencyLocation,
};

type IndexMap<K, V, H = FxHasher> = RawIndexMap<K, V, BuildHasherDefault<H>>;
type IndexSet<K, H = FxHasher> = RawIndexSet<K, BuildHasherDefault<H>>;

#[derive(Debug, Clone)]
pub struct ChunkGroupInfo {
  pub initialized: bool,
  pub ukey: CgiUkey,
  pub chunk_group: ChunkGroupUkey,
  pub chunk_loading: bool,
  pub async_chunks: bool,
  pub runtime: RuntimeSpec,
  pub min_available_modules: BigUint,
  pub min_available_modules_init: bool,
  pub available_modules_to_be_merged: Vec<BigUint>,

  pub skipped_items: IdentifierIndexSet,
  pub skipped_module_connections: IndexSet<(ModuleIdentifier, Vec<DependencyId>)>,
  // set of children chunk groups, that will be revisited when available_modules shrink
  pub children: UkeyIndexSet<CgiUkey>,
  pub parents: IndexMap<CgiUkey, Vec<AsyncDependenciesBlockIdentifier>>,
  // set of chunk groups that are the source for min_available_modules
  pub available_sources: UkeyIndexSet<CgiUkey>,
  // set of chunk groups which depend on the this chunk group as available_source
  pub available_children: UkeyIndexSet<CgiUkey>,

  // set of modules available including modules from this chunk group
  // A derived attribute, therefore utilizing interior mutability to manage updates
  resulting_available_modules: Option<BigUint>,

  pub outgoing_blocks:
    RawHashSet<AsyncDependenciesBlockIdentifier, BuildHasherDefault<IdentifierHasher>>,
}

impl DatabaseItem for ChunkGroupInfo {
  type ItemUkey = CgiUkey;

  fn ukey(&self) -> Self::ItemUkey {
    self.ukey
  }
}

impl ChunkGroupInfo {
  pub fn new(
    chunk_group: ChunkGroupUkey,
    runtime: RuntimeSpec,
    chunk_loading: bool,
    async_chunks: bool,
  ) -> Self {
    Self {
      initialized: false,
      ukey: CgiUkey::new(),
      chunk_group,
      chunk_loading,
      async_chunks,
      runtime,
      min_available_modules: Default::default(),
      min_available_modules_init: false,
      available_modules_to_be_merged: Default::default(),
      skipped_items: Default::default(),
      skipped_module_connections: Default::default(),
      children: Default::default(),
      parents: Default::default(),
      available_sources: Default::default(),
      available_children: Default::default(),
      resulting_available_modules: Default::default(),
      outgoing_blocks: Default::default(),
    }
  }

  fn calculate_resulting_available_modules(
    &mut self,
    compilation: &Compilation,
    mask_by_chunk: &UkeyMap<ChunkUkey, BigUint>,
  ) -> BigUint {
    let resulting_available_modules = &mut self.resulting_available_modules;
    if let Some(resulting_available_modules) = resulting_available_modules.clone() {
      return resulting_available_modules;
    }

    let mut new_resulting_available_modules = self.min_available_modules.clone();
    let chunk_group = compilation
      .chunk_group_by_ukey
      .expect_get(&self.chunk_group);

    // add the modules from the chunk group to the set
    for chunk in &chunk_group.chunks {
      let mask = mask_by_chunk
        .get(chunk)
        .expect("chunk must in mask_by_chunk");
      new_resulting_available_modules |= mask
    }

    *resulting_available_modules = Some(new_resulting_available_modules.clone());
    new_resulting_available_modules
  }

  fn invalidate_resulting_available_modules(&mut self) {
    let resulting_available_modules = &mut self.resulting_available_modules;
    *resulting_available_modules = None;
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct OptionalRuntimeSpec(pub Vec<Arc<str>>);

impl From<Option<RuntimeSpec>> for OptionalRuntimeSpec {
  fn from(value: Option<RuntimeSpec>) -> Self {
    let mut vec = value.unwrap_or_default().into_iter().collect::<Vec<_>>();
    vec.sort();
    Self(vec)
  }
}

static NEXT_CGI_UKEY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CgiUkey(Ukey, std::marker::PhantomData<ChunkGroupInfo>);

// safety: CgiUkey is just numeric identifier, it's safe to impl Send and Sync
unsafe impl Send for CgiUkey {}
unsafe impl Sync for CgiUkey {}

impl_item_ukey!(CgiUkey);

impl CgiUkey {
  pub fn new() -> Self {
    Self(
      NEXT_CGI_UKEY
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        .into(),
      std::marker::PhantomData,
    )
  }
}

pub(crate) type BlockModulesRuntimeMap = IndexMap<
  OptionalRuntimeSpec,
  IndexMap<
    DependenciesBlockIdentifier,
    Vec<(ModuleIdentifier, ConnectionState, Vec<DependencyId>)>,
  >,
>;

// Queue is used to debug code-splitting,
// we store every op.
// #[derive(Default)]
// struct Queue {
//   inner: Vec<QueueAction>,
//   records: Vec<String>,
// }

// impl Queue {
//   fn push(&mut self, item: QueueAction) {
//     self.inner.push(item);
//   }

//   fn len(&self) -> usize {
//     self.inner.len()
//   }

//   fn reverse(&mut self) {
//     self.inner.reverse();
//   }

//   fn is_empty(&self) -> bool {
//     self.inner.is_empty()
//   }

//   fn pop(&mut self) -> Option<QueueAction> {
//     let item = self.inner.pop();
//     if let Some(item) = &item {
//       let res = match item {
//         QueueAction::AddAndEnterEntryModule(item) => {
//           format!("add_enter_entry: {}", item.module)
//         }
//         QueueAction::AddAndEnterModule(item) => {
//           format!("add_enter: {}|{:?}", item.module, item.orig)
//         }
//         QueueAction::_EnterModule(_) => todo!(),
//         QueueAction::ProcessBlock(item) => {
//           format!("process_block: {:?}", item.block)
//         }
//         QueueAction::ProcessEntryBlock(item) => {
//           format!("process_entry_block: {:?}", item.block,)
//         }
//         QueueAction::LeaveModule(item) => {
//           format!("leave: {}", item.module,)
//         }
//       };

//       let cwd = std::env::current_dir()
//         .unwrap()
//         .parent()
//         .unwrap()
//         .parent()
//         .unwrap()
//         .to_string_lossy()
//         .to_string();
//       self
//         .records
//         .push(LOC_RE.replace(&res, "").replace(&cwd, "").to_string());
//     }

//     item
//   }
// }

#[derive(Clone, Debug, Default)]
pub(crate) struct CodeSplitter {
  pub(crate) chunk_group_info_map: UkeyMap<ChunkGroupUkey, CgiUkey>,
  pub(crate) chunk_group_infos: Database<ChunkGroupInfo>,
  outdated_order_index_chunk_groups: HashSet<CgiUkey>,
  pub(crate) blocks_by_cgi: UkeyMap<CgiUkey, HashSet<DependenciesBlockIdentifier>>,
  pub(crate) runtime_chunks: UkeySet<ChunkUkey>,
  next_free_module_pre_order_index: u32,
  next_free_module_post_order_index: u32,
  next_chunk_group_index: u32,
  queue: Vec<QueueAction>,
  queue_delayed: Vec<QueueAction>,
  queue_connect: UkeyIndexMap<CgiUkey, IndexSet<(CgiUkey, Option<ProcessBlock>)>>,
  chunk_groups_for_combining: UkeyIndexSet<CgiUkey>,
  pub(crate) outdated_chunk_group_info: UkeyIndexSet<CgiUkey>,
  chunk_groups_for_merging: IndexSet<(CgiUkey, Option<ProcessBlock>)>,
  pub(crate) block_chunk_groups: HashMap<DependenciesBlockIdentifier, CgiUkey>,
  pub(crate) named_chunk_groups: HashMap<String, CgiUkey>,
  pub(crate) named_async_entrypoints: HashMap<String, CgiUkey>,
  pub(crate) block_modules_runtime_map: BlockModulesRuntimeMap,
  pub(crate) ordinal_by_module: IdentifierMap<u64>,
  pub(crate) mask_by_chunk: UkeyMap<ChunkUkey, BigUint>,

  stat_processed_queue_items: u32,
  stat_processed_blocks: u32,
  stat_connected_chunk_groups: u32,
  stat_processed_chunk_groups_for_merging: u32,
  stat_merged_available_module_sets: u32,
  stat_chunk_group_info_updated: u32,
  stat_child_chunk_groups_reconnected: u32,
  pub(crate) stat_chunk_group_created: u32,

  // incremental stat
  pub(crate) stat_invalidated_chunk_group: u32,
  pub(crate) stat_invalidated_caches: u32,
  pub(crate) stat_use_cache: u32,
  pub(crate) stat_cache_miss_by_cant_rebuild: u32,
  pub(crate) stat_cache_miss_by_available_modules: u32,

  // represents the edge of how chunk is created
  pub(crate) edges: HashMap<AsyncDependenciesBlockIdentifier, ModuleIdentifier>,

  // created from edges
  pub(crate) chunk_caches: HashMap<AsyncDependenciesBlockIdentifier, ChunkCreateData>,
}

fn add_chunk_in_group(
  group_options: Option<&GroupOptions>,
  module_id: ModuleIdentifier,
  loc: Option<DependencyLocation>,
  request: Option<String>,
) -> ChunkGroup {
  let options = ChunkGroupOptions::new(
    group_options
      .and_then(|x| x.name())
      .map(|name| name.to_string()),
    group_options
      .and_then(|x| x.normal_options())
      .and_then(|x| x.preload_order),
    group_options
      .and_then(|x| x.normal_options())
      .and_then(|x| x.prefetch_order),
    group_options
      .and_then(|x| x.normal_options())
      .and_then(|x| x.fetch_priority),
  );
  let kind = ChunkGroupKind::Normal { options };
  let mut chunk_group = ChunkGroup::new(kind);
  chunk_group.add_origin(Some(module_id), loc, request);
  chunk_group
}

fn get_active_state_of_connections(
  connections: &[DependencyId],
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
) -> ConnectionState {
  let mut iter = connections.iter();
  let id = iter.next().expect("should have connection");
  let mut merged = module_graph
    .connection_by_dependency_id(id)
    .expect("should have connection")
    .active_state(module_graph, runtime);
  if merged.is_true() {
    return merged;
  }
  for c in iter {
    let c = module_graph
      .connection_by_dependency_id(c)
      .expect("should have connection");
    merged = merged + c.active_state(module_graph, runtime);
    if merged.is_true() {
      return merged;
    }
  }
  merged
}

impl CodeSplitter {
  pub fn get_module_ordinal(&self, module_id: ModuleIdentifier) -> u64 {
    *self.ordinal_by_module.get(&module_id).unwrap_or_else(|| {
      panic!(
        "expected a module ordinal for identifier '{}', but none was found.",
        &module_id
      )
    })
  }

  pub fn prepare_entry_input(
    &mut self,
    name: &str,
    compilation: &mut Compilation,
  ) -> Result<(ChunkGroupUkey, Vec<ModuleIdentifier>)> {
    let Some(entry_data) = compilation.entries.get(name) else {
      return Err(rspack_error::error!(format!(
        "entry '{}' is not found",
        name
      )));
    };

    let mut modules = vec![];
    let options = &entry_data.options;
    let dependencies = [
      compilation.global_entry.dependencies.clone(),
      entry_data.dependencies.clone(),
    ]
    .concat();
    let requests = dependencies
      .iter()
      .map(|dep_id| {
        let module_graph = compilation.get_module_graph();
        let dep = module_graph.dependency_by_id(dep_id);
        let mut request = None;
        if let Some(dep) = dep {
          if let Some(d) = dep.as_any().downcast_ref::<EntryDependency>() {
            request = Some(d.request().to_string());
          }
        }
        request
      })
      .collect::<Vec<_>>();
    let module_identifiers = dependencies
      .iter()
      .filter_map(|dep| {
        compilation
          .get_module_graph()
          .module_identifier_by_dependency_id(dep)
          .copied()
      })
      .collect::<Vec<_>>();

    let (chunk_ukey, created) = Compilation::add_named_chunk(
      name.to_string(),
      &mut compilation.chunk_by_ukey,
      &mut compilation.named_chunks,
    );
    if created && let Some(mutations) = compilation.incremental.mutations_write() {
      mutations.add(Mutation::ChunkAdd { chunk: chunk_ukey });
    }
    self.mask_by_chunk.insert(chunk_ukey, BigUint::from(0u32));
    let runtime = get_entry_runtime(name, options, &compilation.entries);
    let chunk = compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);
    if let Some(filename) = &entry_data.options.filename {
      chunk.set_filename_template(Some(filename.clone()));
    }

    compilation.chunk_graph.add_chunk(chunk.ukey());

    let mut entrypoint = ChunkGroup::new(ChunkGroupKind::new_entrypoint(
      true,
      Box::new(options.clone()),
    ));

    for request in requests {
      let loc = Some(DependencyLocation::Synthetic(
        SyntheticDependencyLocation::new(name),
      ));
      entrypoint.add_origin(None, loc, request);
    }

    let chunk_group_info = {
      self.stat_chunk_group_created += 1;
      let mut cgi = ChunkGroupInfo::new(
        entrypoint.ukey,
        runtime,
        !matches!(
          options
            .chunk_loading
            .as_ref()
            .unwrap_or(&compilation.options.output.chunk_loading),
          ChunkLoading::Disable
        ),
        options
          .async_chunks
          .unwrap_or(compilation.options.output.async_chunks),
      );
      cgi.min_available_modules_init = true;
      cgi
    };

    if options.depend_on.is_none() && !matches!(&options.runtime, Some(EntryRuntime::String(_))) {
      entrypoint.set_runtime_chunk(chunk.ukey());
    }

    entrypoint.set_entry_point_chunk(chunk.ukey());
    entrypoint.connect_chunk(chunk);

    compilation
      .named_chunk_groups
      .insert(name.to_string(), entrypoint.ukey);

    compilation
      .entrypoints
      .insert(name.to_string(), entrypoint.ukey);

    let entrypoint = {
      let ukey = entrypoint.ukey;
      compilation.chunk_group_by_ukey.add(entrypoint);
      compilation.chunk_group_by_ukey.expect_get(&ukey)
    };

    for module_identifier in module_identifiers.iter() {
      compilation.chunk_graph.add_module(*module_identifier);

      modules.push(*module_identifier);

      compilation.chunk_graph.connect_chunk_and_entry_module(
        chunk.ukey(),
        *module_identifier,
        entrypoint.ukey,
      );
    }

    let module_graph = compilation.get_module_graph();
    let global_included_modules = compilation
      .global_entry
      .include_dependencies
      .iter()
      .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
      .copied()
      .sorted_unstable();
    let included_modules = entry_data
      .include_dependencies
      .iter()
      .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
      .copied()
      .sorted_unstable();
    let included_modules = global_included_modules.chain(included_modules);

    modules.extend(included_modules);

    let cgi = chunk_group_info.ukey;
    self
      .chunk_group_infos
      .entry(cgi)
      .or_insert(chunk_group_info);

    self.chunk_group_info_map.insert(entrypoint.ukey, cgi);
    if let Some(name) = entrypoint.name() {
      self.named_chunk_groups.insert(name.to_string(), cgi);
    }

    Ok((entrypoint.ukey, modules))
  }

  pub fn set_entry_runtime_and_depend_on(
    &mut self,
    name: &str,
    compilation: &mut Compilation,
  ) -> Result<()> {
    let Some(entry_data) = compilation.entries.get(name) else {
      return Err(rspack_error::error!(format!(
        "entry '{}' is not found",
        name
      )));
    };

    let options = &entry_data.options;
    let runtime = &options.runtime;
    let depend_on = &options.depend_on;

    // delay push diagnostics to avoid borrow checker error
    let mut runtime_errors = vec![];

    if depend_on.is_some() && runtime.is_some() {
      runtime_errors.push(
        Diagnostic::from(error!(
          "Entrypoint '{name}' has 'dependOn' and 'runtime' specified. This is not valid.
Entrypoints that depend on other entrypoints do not have their own runtime.
They will use the runtime(s) from referenced entrypoints instead.
Remove the 'runtime' option from the entrypoint."
        ))
        .with_chunk(compilation.entrypoints.get(name).map(|key| {
          compilation
            .chunk_group_by_ukey
            .expect_get(key)
            .get_entry_point_chunk()
            .as_u32()
        })),
      );
    }

    if let Some(depend_on) = &options.depend_on {
      let ukey = compilation
        .entrypoints
        .get(name)
        .ok_or_else(|| error!("no entrypoints found"))?;

      let mut entry_point_runtime = None;
      let mut depend_on_entries = vec![];
      let mut entry_point_parents = vec![];
      let mut has_error = false;

      {
        let entry_point = compilation.chunk_group_by_ukey.expect_get(ukey);
        let entry_point_chunk = compilation
          .chunk_by_ukey
          .expect_get(&entry_point.get_entry_point_chunk());
        let referenced_chunks =
          entry_point_chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey);

        for dep in depend_on {
          if let Some(dependency_ukey) = compilation.entrypoints.get(dep) {
            let dependency_chunk_ukey = compilation
              .chunk_group_by_ukey
              .expect_get(dependency_ukey)
              .get_entry_point_chunk();
            if referenced_chunks.contains(&dependency_chunk_ukey) {
              runtime_errors.push(Diagnostic::from(
                error!(
                  "Entrypoints '{name}' and '{dep}' use 'dependOn' to depend on each other in a circular way."
                ),
              ).with_chunk(Some(entry_point.get_entry_point_chunk().as_u32())));
              entry_point_runtime = Some(entry_point_chunk.ukey());
              has_error = true;
              break;
            }
            depend_on_entries.push(dependency_ukey);
          } else {
            panic!("Entry {name} depends on {dep}, but this entry was not found");
          }
        }
      }

      if has_error {
        let entry_point = compilation.chunk_group_by_ukey.expect_get_mut(ukey);
        entry_point.set_runtime_chunk(entry_point_runtime.expect("Should set runtime chunk"));
      } else {
        {
          for depend in depend_on_entries {
            let depend_chunk_group = compilation.chunk_group_by_ukey.expect_get_mut(depend);
            if depend_chunk_group.add_child(*ukey) {
              entry_point_parents.push(*depend);
            }
          }
        }
        let entry_point = compilation.chunk_group_by_ukey.expect_get_mut(ukey);
        for parent in entry_point_parents {
          entry_point.add_parent(parent);
        }
      }
    } else if let Some(EntryRuntime::String(runtime)) = &options.runtime {
      let ukey = compilation
        .entrypoints
        .get(name)
        .ok_or_else(|| error!("no entrypoints found"))?;

      let entry_point = compilation.chunk_group_by_ukey.expect_get_mut(ukey);

      let chunk = match compilation.named_chunks.get(runtime) {
        Some(ukey) => {
          if !self.runtime_chunks.contains(ukey) {
            let entry_chunk = entry_point.get_entry_point_chunk();
            runtime_errors.push(Diagnostic::from(
              error!(
                "Entrypoint '{name}' has a 'runtime' option which points to another entrypoint named '{runtime}'.
It's not valid to use other entrypoints as runtime chunk.
Did you mean to use 'dependOn: \"{runtime}\"' instead to allow using entrypoint '{name}' within the runtime of entrypoint '{runtime}'? For this '{runtime}' must always be loaded when '{name}' is used.
Or do you want to use the entrypoints '{name}' and '{runtime}' independently on the same page with a shared runtime? In this case give them both the same value for the 'runtime' option. It must be a name not already used by an entrypoint."
                              ),
            ).with_chunk(Some(entry_chunk.as_u32())));
            entry_point.set_runtime_chunk(entry_chunk);
          }
          compilation.chunk_by_ukey.expect_get_mut(ukey)
        }
        None => {
          let (chunk_ukey, created) = Compilation::add_named_chunk(
            runtime.to_string(),
            &mut compilation.chunk_by_ukey,
            &mut compilation.named_chunks,
          );
          if created && let Some(mutations) = compilation.incremental.mutations_write() {
            mutations.add(Mutation::ChunkAdd { chunk: chunk_ukey });
          }
          self.mask_by_chunk.insert(chunk_ukey, BigUint::from(0u32));
          let chunk = compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);
          chunk.set_prevent_integration(true);
          compilation.chunk_graph.add_chunk(chunk.ukey());
          self.runtime_chunks.insert(chunk.ukey());
          chunk
        }
      };

      entry_point.unshift_chunk(chunk);
      chunk.add_group(entry_point.ukey);
      entry_point.set_runtime_chunk(chunk.ukey());
    }

    compilation.extend_diagnostics(runtime_errors);
    Ok(())
  }

  pub fn prepare_input_entrypoints_and_modules(
    &mut self,
    compilation: &mut Compilation,
  ) -> Result<UkeyIndexMap<ChunkGroupUkey, Vec<ModuleIdentifier>>> {
    let mut input_entrypoints_and_modules: UkeyIndexMap<ChunkGroupUkey, Vec<ModuleIdentifier>> =
      UkeyIndexMap::default();
    let mut assign_depths_map = IdentifierMap::default();

    let entries = compilation.entries.keys().cloned().collect::<Vec<_>>();
    for name in &entries {
      let (entry_point, modules) = self.prepare_entry_input(name, compilation)?;
      assign_depths(
        &mut assign_depths_map,
        &compilation.get_module_graph(),
        modules.iter(),
      );
      input_entrypoints_and_modules.insert(entry_point, modules);
    }

    // Using this defer insertion strategies to workaround rustc borrow rules
    for (k, v) in assign_depths_map {
      compilation.get_module_graph_mut().set_depth(k, v);
    }

    for name in &entries {
      self.set_entry_runtime_and_depend_on(name, compilation)?;
    }

    Ok(input_entrypoints_and_modules)
  }

  pub fn prepare_entries(
    &mut self,
    input_entrypoints_and_modules: UkeyIndexMap<ChunkGroupUkey, Vec<ModuleIdentifier>>,
    compilation: &mut Compilation,
  ) -> Result<()> {
    let logger = compilation.get_logger("rspack.buildChunkGraph");
    let start = logger.time("prepare entrypoints");
    logger.time_end(start);

    for (chunk_group_ukey, modules) in input_entrypoints_and_modules {
      let chunk_group = compilation
        .chunk_group_by_ukey
        .expect_get_mut(&chunk_group_ukey);

      let cgi = self
        .chunk_group_info_map
        .get(&chunk_group.ukey)
        .expect("should have chunk group info");

      self.next_chunk_group_index += 1;
      chunk_group.index = Some(self.next_chunk_group_index);

      if !chunk_group.parents.is_empty() {
        // min_available_modules for child entrypoints are unknown yet, set to undefined.
        // This means no module is added until other sets are merged into
        // this min_available_modules (by the parent entrypoints)
        let chunk_group_info = self.chunk_group_infos.expect_get_mut(cgi);
        chunk_group_info.skipped_items = IdentifierIndexSet::from_iter(modules);
        self.chunk_groups_for_combining.insert(*cgi);
      } else {
        // The application may start here: We start with an empty list of available modules
        let chunk = chunk_group.get_entry_point_chunk();
        for module in modules {
          self
            .queue
            .push(QueueAction::AddAndEnterModule(AddAndEnterModule {
              chunk,
              chunk_group_info: *cgi,
              module,
            }));
        }
      }
    }

    // Fill available_sources with parent-child dependencies between entrypoints
    for cgi in &self.chunk_groups_for_combining {
      let chunk_group_info = self.chunk_group_infos.expect_get_mut(cgi);
      let chunk_group = compilation
        .chunk_group_by_ukey
        .expect_get(&chunk_group_info.chunk_group);
      chunk_group_info.available_sources.clear();
      for parent in chunk_group.parents_iterable() {
        if let Some(parent_chunk_group_info_ukey) = self.chunk_group_info_map.get(parent) {
          chunk_group_info
            .available_sources
            .insert(*parent_chunk_group_info_ukey);
        }
      }
      for parent in chunk_group.parents_iterable() {
        if let Some(parent_chunk_group_info_ukey) = self.chunk_group_info_map.get(parent) {
          let parent_chunk_group_info = self
            .chunk_group_infos
            .expect_get_mut(parent_chunk_group_info_ukey);
          parent_chunk_group_info.available_children.insert(*cgi);
        }
      }
    }

    Ok(())
  }

  // #[tracing::instrument(skip_all)]
  pub fn split(&mut self, compilation: &mut Compilation) -> Result<()> {
    let logger = compilation.get_logger("rspack.buildChunkGraph");

    // pop() is used to read from the queue
    // so it need to be reversed to be iterated in
    // correct order
    self.queue.reverse();

    let start = logger.time("process queue");
    // Iterative traversal of the Module graph
    // Recursive would be simpler to write but could result in Stack Overflows
    while !self.queue.is_empty() || !self.queue_connect.is_empty() || !self.queue_delayed.is_empty()
    {
      self.process_queue(compilation);

      if !self.chunk_groups_for_combining.is_empty() {
        self.process_chunk_groups_for_combining(compilation);
      }

      if !self.queue_connect.is_empty() {
        self.process_connect_queue(compilation);

        if !self.chunk_groups_for_merging.is_empty() {
          self.process_chunk_groups_for_merging(compilation);
        }
      }

      if !self.outdated_chunk_group_info.is_empty() {
        self.process_outdated_chunk_group_info(compilation);
      }

      if self.queue.is_empty() {
        let queue_delay = std::mem::take(&mut self.queue_delayed);
        self.queue = queue_delay;
        self.queue.reverse();
      }
    }
    logger.time_end(start);

    let start = logger.time("extend chunkGroup runtime");
    for (chunk_group, cgi) in &self.chunk_group_info_map {
      let chunk_group = compilation.chunk_group_by_ukey.expect_get(chunk_group);
      let cgi = self.chunk_group_infos.expect_get(cgi);
      for chunk_ukey in chunk_group.chunks.iter() {
        if let Some(chunk) = compilation.chunk_by_ukey.get_mut(chunk_ukey) {
          chunk.set_runtime(merge_runtime(chunk.runtime(), &cgi.runtime));
        }
      }
    }
    logger.time_end(start);

    let outdated_order_index_chunk_groups =
      std::mem::take(&mut self.outdated_order_index_chunk_groups);

    for outdated in outdated_order_index_chunk_groups {
      let cgi = self.chunk_group_infos.expect_get(&outdated);
      let chunk_group_ukey = cgi.chunk_group;
      let runtime = cgi.runtime.clone();
      let chunk_group = compilation
        .chunk_group_by_ukey
        .expect_get_mut(&chunk_group_ukey);

      chunk_group.next_pre_order_index = 0;
      chunk_group.next_post_order_index = 0;

      let chunk_group = compilation
        .chunk_group_by_ukey
        .expect_get(&chunk_group_ukey);

      let module_graph = compilation.get_module_graph();

      let roots = if let Some(blocks) = self.blocks_by_cgi.get(&cgi.ukey) {
        let mut blocks = blocks.iter().copied().collect::<Vec<_>>();
        blocks.sort_unstable();

        // if one chunk group has multiple blocks, the modules order indices are very likely incorrect
        // in every block. For example:
        // one chunk import 'a' first then import 'b', while in other chunk import 'b' first then import 'a'.
        // User use webpackChunkName to merge these 2 chunks, the `a` and `b` will be in the same chunk, their
        // orders cannot be determined.
        // Only use the first block to calculate the order indices.
        let Some(DependenciesBlockIdentifier::AsyncDependenciesBlock(block)) =
          blocks.first().copied()
        else {
          continue;
        };

        let Some(block) = module_graph.block_by_id(&block) else {
          continue;
        };
        let root_modules = block
          .get_dependencies()
          .iter()
          .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
          .copied()
          .collect::<Vec<_>>();

        Some(root_modules)
      } else if chunk_group.kind.is_entrypoint() && chunk_group.is_initial() {
        let entry_chunk_ukey = chunk_group.get_entry_point_chunk();
        let entry_modules = compilation
          .chunk_graph
          .get_chunk_entry_modules(&entry_chunk_ukey);
        Some(entry_modules)
      } else {
        None
      };

      let Some(roots) = roots else {
        continue;
      };

      let mut visited = IdentifierIndexSet::default();

      let mut ctx = (0, 0, Default::default());
      for root in roots {
        self.calculate_order_index(root, &runtime, &mut visited, &mut ctx, compilation);

        let chunk_group = compilation
          .chunk_group_by_ukey
          .expect_get_mut(&chunk_group_ukey);
        for (id, (pre, post)) in &ctx.2 {
          chunk_group.module_pre_order_indices.insert(*id, *pre);
          chunk_group.module_post_order_indices.insert(*id, *post);
        }
      }
    }

    logger.log(format!(
      "{} queue items processed ({} blocks)",
      itoa!(self.stat_processed_queue_items),
      itoa!(self.stat_processed_blocks)
    ));
    logger.log(format!(
      "{} chunk groups connected",
      itoa!(self.stat_connected_chunk_groups),
    ));
    logger.log(format!(
      "{} chunk groups processed for merging ({} module sets)",
      itoa!(self.stat_processed_chunk_groups_for_merging),
      itoa!(self.stat_merged_available_module_sets),
    ));
    logger.log(format!(
      "{} chunk group info updated ({} already connected chunk groups reconnected)",
      itoa!(self.stat_chunk_group_info_updated),
      itoa!(self.stat_child_chunk_groups_reconnected),
    ));

    if compilation
      .incremental
      .can_read_mutations(IncrementalPasses::BUILD_CHUNK_GRAPH)
    {
      let logger = compilation.get_logger("rspack.incremental.buildChunkGraph");
      logger.log(format!(
        "{} chunk group created",
        self.stat_chunk_group_created,
      ));
      logger.log(format!(
        "{} chunk cache invalidated",
        self.stat_invalidated_caches,
      ));
      logger.log(format!(
        "{} chunk group invalidated",
        self.stat_invalidated_chunk_group,
      ));
      logger.log(format!(
        "{} chunk group created from cache",
        self.stat_use_cache,
      ));
      logger.log(format!(
        "{} cache missed by cannot rebuild",
        self.stat_cache_miss_by_cant_rebuild,
      ));
      logger.log(format!(
        "{} cache missed by incorrect available modules",
        self.stat_cache_miss_by_available_modules,
      ));
      self.update_cache(compilation);
    }

    Ok(())
  }

  fn calculate_order_index(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: &RuntimeSpec,
    visited: &mut IdentifierIndexSet,
    ctx: &mut (usize, usize, IndexMap<ModuleIdentifier, (usize, usize)>),
    compilation: &mut Compilation,
  ) {
    let block_modules =
      self.get_block_modules(module_identifier.into(), Some(runtime), compilation);
    if visited.contains(&module_identifier) {
      return;
    }
    visited.insert(module_identifier);

    let indices = ctx.2.entry(module_identifier).or_default();

    indices.0 = ctx.0;
    ctx.0 += 1;

    for (module, state, _) in block_modules.iter() {
      if state.is_false() {
        continue;
      }

      self.calculate_order_index(*module, runtime, visited, ctx, compilation);
    }

    let indices = ctx.2.entry(module_identifier).or_default();

    indices.1 = ctx.1;
    ctx.1 += 1;
  }

  pub(crate) fn process_queue(&mut self, compilation: &mut Compilation) {
    tracing::trace!("process_queue");
    while let Some(action) = self.queue.pop() {
      self.stat_processed_queue_items += 1;

      match action {
        QueueAction::AddAndEnterEntryModule(i) => self.add_and_enter_entry_module(&i, compilation),
        QueueAction::AddAndEnterModule(i) => self.add_and_enter_module(&i, compilation),
        QueueAction::_EnterModule(i) => self.enter_module(&i, compilation),
        QueueAction::ProcessBlock(i) => self.process_block(&i, compilation),
        QueueAction::ProcessEntryBlock(i) => self.process_entry_block(&i, compilation),
        QueueAction::LeaveModule(i) => self.leave_module(&i, compilation),
      }
    }
  }

  fn add_and_enter_entry_module(
    &mut self,
    item: &AddAndEnterEntryModule,
    compilation: &mut Compilation,
  ) {
    tracing::trace!("add_and_enter_entry_module {:?}", item);
    let module_ordinal = self.ordinal_by_module.get(&item.module).unwrap_or_else(|| {
      panic!(
        "expected a module ordinal for identifier '{}', but none was found.",
        &item.module
      )
    });
    let cgi = self
      .chunk_group_infos
      .expect_get_mut(&item.chunk_group_info);

    if compilation
      .chunk_graph
      .is_module_in_chunk(&item.module, item.chunk)
    {
      return;
    }

    if cgi.min_available_modules.bit(*module_ordinal) {
      cgi.skipped_items.insert(item.module);
      return;
    }

    compilation.chunk_graph.connect_chunk_and_entry_module(
      item.chunk,
      item.module,
      cgi.chunk_group,
    );
    let chunk_mask = self
      .mask_by_chunk
      .get_mut(&item.chunk)
      .expect("chunk must in mask_by_chunk");
    chunk_mask.set_bit(*module_ordinal, true);

    self.add_and_enter_module(
      &AddAndEnterModule {
        module: item.module,
        chunk_group_info: item.chunk_group_info,
        chunk: item.chunk,
      },
      compilation,
    )
  }

  fn add_and_enter_module(&mut self, item: &AddAndEnterModule, compilation: &mut Compilation) {
    tracing::trace!("add_and_enter_module {:?}", item);
    let cgi = self
      .chunk_group_infos
      .expect_get_mut(&item.chunk_group_info);

    if compilation
      .chunk_graph
      .is_module_in_chunk(&item.module, item.chunk)
    {
      return;
    }

    // if this module in parent chunks
    let module_ordinal = self.ordinal_by_module.get(&item.module).unwrap_or_else(|| {
      panic!(
        "expected a module ordinal for identifier '{}', but none was found.",
        &item.module
      )
    });

    if cgi.min_available_modules.bit(*module_ordinal) {
      cgi.skipped_items.insert(item.module);
      return;
    }

    compilation
      .chunk_graph
      .connect_chunk_and_module(item.chunk, item.module);

    let chunk_mask = self
      .mask_by_chunk
      .get_mut(&item.chunk)
      .expect("chunk must in mask_by_chunk");
    chunk_mask.set_bit(*module_ordinal, true);

    self.enter_module(
      &EnterModule {
        module: item.module,
        chunk_group_info: item.chunk_group_info,
        chunk: item.chunk,
      },
      compilation,
    )
  }

  fn enter_module(&mut self, item: &EnterModule, compilation: &mut Compilation) {
    tracing::trace!("enter_module {:?}", item);
    let cgi = self.chunk_group_infos.expect_get(&item.chunk_group_info);

    let chunk_group = compilation
      .chunk_group_by_ukey
      .expect_get_mut(&cgi.chunk_group);

    #[allow(clippy::map_entry)]
    if !chunk_group
      .module_pre_order_indices
      .contains_key(&item.module)
    {
      chunk_group
        .module_pre_order_indices
        .insert(item.module, chunk_group.next_pre_order_index);
      chunk_group.next_pre_order_index += 1;
    }

    {
      let mut module_graph = compilation.get_module_graph_mut();
      let module = module_graph
        .module_graph_module_by_identifier_mut(&item.module)
        .unwrap_or_else(|| panic!("No module found {:?}", &item.module));

      if module.pre_order_index.is_none() {
        module.pre_order_index = Some(self.next_free_module_pre_order_index);
        self.next_free_module_pre_order_index += 1;
      }
    }

    self.queue.push(QueueAction::LeaveModule(LeaveModule {
      module: item.module,
      chunk_group_info: item.chunk_group_info,
    }));
    self.process_block(
      &ProcessBlock {
        module: item.module,
        chunk_group_info: item.chunk_group_info,
        chunk: item.chunk,
        block: item.module.into(),
      },
      compilation,
    )
  }

  fn leave_module(&mut self, item: &LeaveModule, compilation: &mut Compilation) {
    tracing::trace!("leave_module {:?}", item);
    let cgi = self.chunk_group_infos.expect_get(&item.chunk_group_info);

    let chunk_group = compilation
      .chunk_group_by_ukey
      .expect_get_mut(&cgi.chunk_group);

    #[allow(clippy::map_entry)]
    if !chunk_group
      .module_post_order_indices
      .contains_key(&item.module)
    {
      chunk_group
        .module_post_order_indices
        .insert(item.module, chunk_group.next_post_order_index);
      chunk_group.next_post_order_index += 1;
    }

    let mut module_graph = compilation.get_module_graph_mut();
    let module = module_graph
      .module_graph_module_by_identifier_mut(&item.module)
      .unwrap_or_else(|| panic!("no module found: {:?}", &item.module));

    if module.post_order_index.is_none() {
      module.post_order_index = Some(self.next_free_module_post_order_index);
      self.next_free_module_post_order_index += 1;
    }
  }

  fn process_entry_block(&mut self, item: &ProcessEntryBlock, compilation: &mut Compilation) {
    tracing::trace!("process_entry_block {:?}", item);

    self.stat_processed_blocks += 1;

    let chunk_group_info = self.chunk_group_infos.expect_get(&item.chunk_group_info);

    let runtime = chunk_group_info.runtime.clone();

    let modules = self.get_block_modules(item.block.into(), Some(&runtime), compilation);

    for (module, active_state, _) in modules {
      if active_state.is_true() {
        self.queue.push(QueueAction::AddAndEnterEntryModule(
          AddAndEnterEntryModule {
            chunk: item.chunk,
            chunk_group_info: item.chunk_group_info,
            module,
          },
        ));
      } else {
        self.queue.push(QueueAction::ProcessBlock(ProcessBlock {
          chunk: item.chunk,
          chunk_group_info: item.chunk_group_info,
          block: module.into(),
          module: item.module,
        }));
      }
    }
    let blocks = compilation
      .get_module_graph()
      .block_by_id(&item.block)
      .expect("should have block")
      .get_blocks()
      .to_vec();
    for block in blocks {
      self.make_chunk_group(
        block,
        item.module,
        item.chunk_group_info,
        item.chunk,
        compilation,
      );
    }
  }

  fn process_block(&mut self, item: &ProcessBlock, compilation: &mut Compilation) {
    tracing::trace!("process_block {:?}", item);

    self.stat_processed_blocks += 1;

    let chunk_group_info = self.chunk_group_infos.expect_get(&item.chunk_group_info);
    let runtime = chunk_group_info.runtime.clone();
    let min_available_modules = chunk_group_info.min_available_modules.clone();

    let block_modules = self.get_block_modules(item.block, Some(&runtime), compilation);

    for (module, active_state, connections) in block_modules.into_iter().rev() {
      if compilation
        .chunk_graph
        .is_module_in_chunk(&module, item.chunk)
      {
        // skip early if already connected
        continue;
      }

      let chunk_group_info = self
        .chunk_group_infos
        .expect_get_mut(&item.chunk_group_info);

      if !active_state.is_true() {
        chunk_group_info
          .skipped_module_connections
          .insert((module, connections));
        if active_state.is_false() {
          continue;
        }
      }

      let ordinal = self.ordinal_by_module.get(&module).unwrap_or_else(|| {
        panic!("expected a module ordinal for identifier '{module}', but none was found.")
      });
      if active_state.is_true() && min_available_modules.bit(*ordinal) {
        // already in parent chunks, skip it for now
        chunk_group_info.skipped_items.insert(module);
        continue;
      }

      if active_state.is_true() {
        // webpack use queueBuffer to rev
        self
          .queue
          .push(QueueAction::AddAndEnterModule(AddAndEnterModule {
            chunk: item.chunk,
            chunk_group_info: item.chunk_group_info,
            module,
          }));
      } else {
        self.queue.push(QueueAction::ProcessBlock(ProcessBlock {
          chunk: item.chunk,
          chunk_group_info: item.chunk_group_info,
          block: module.into(),
          module: item.module,
        }));
      }
    }
    let blocks = item.block.get_blocks(compilation);

    for block in blocks {
      self.make_chunk_group(
        block,
        item.module,
        item.chunk_group_info,
        item.chunk,
        compilation,
      );
    }
  }

  pub(crate) fn make_chunk_group(
    &mut self,
    block_id: AsyncDependenciesBlockIdentifier,
    module_id: ModuleIdentifier,
    item_chunk_group_info_ukey: CgiUkey,
    item_chunk_ukey: ChunkUkey,
    compilation: &mut Compilation,
  ) {
    self.edges.insert(block_id, module_id);

    let Some(item_chunk_group_info) = self.chunk_group_infos.get_mut(&item_chunk_group_info_ukey)
    else {
      return;
    };

    item_chunk_group_info.outgoing_blocks.insert(block_id);

    let item_chunk_group_info = self
      .chunk_group_infos
      .expect_get(&item_chunk_group_info_ukey);

    let item_chunk_group = item_chunk_group_info.chunk_group;
    let cgi = self
      .block_chunk_groups
      .get(&DependenciesBlockIdentifier::AsyncDependenciesBlock(
        block_id,
      ));
    let mut entrypoint: Option<ChunkGroupUkey> = None;
    let mut c: Option<ChunkGroupUkey> = None;

    let cgi = if let Some(cgi) = cgi {
      let cgi = self.chunk_group_infos.expect_get(cgi);
      let module_graph = compilation.get_module_graph();
      let block = module_graph
        .block_by_id(&block_id)
        .expect("should have block");
      let entry_options = block.get_group_options().and_then(|o| o.entry_options());
      if entry_options.is_some() {
        entrypoint = Some(cgi.chunk_group);
      } else {
        c = Some(cgi.chunk_group);
      }

      cgi.ukey
    } else {
      let chunk_ukey = if let Some(chunk_name) = compilation
        .get_module_graph()
        .block_by_id(&block_id)
        .unwrap_or_else(|| panic!("should have block: {block_id:?}"))
        .get_group_options()
        .and_then(|x| x.name())
      {
        let (chunk_ukey, created) = Compilation::add_named_chunk(
          chunk_name.to_string(),
          &mut compilation.chunk_by_ukey,
          &mut compilation.named_chunks,
        );
        if created && let Some(mutations) = compilation.incremental.mutations_write() {
          mutations.add(Mutation::ChunkAdd { chunk: chunk_ukey });
        }
        chunk_ukey
      } else {
        let chunk_ukey = Compilation::add_chunk(&mut compilation.chunk_by_ukey);
        if let Some(mutations) = compilation.incremental.mutations_write() {
          mutations.add(Mutation::ChunkAdd { chunk: chunk_ukey });
        }
        chunk_ukey
      };
      compilation.chunk_graph.add_chunk(chunk_ukey);
      self.mask_by_chunk.insert(chunk_ukey, BigUint::from(0u32));
      let module_graph = compilation.get_module_graph();
      let block = module_graph
        .block_by_id(&block_id)
        .expect("should have block");
      let chunk_name = block.get_group_options().and_then(|o| o.name());
      let entry_options = block.get_group_options().and_then(|o| o.entry_options());
      let request = block.request().clone();
      let loc = block.loc();

      let cgi = if let Some(entry_options) = entry_options {
        let cgi =
          if let Some(cgi) = chunk_name.and_then(|name| self.named_async_entrypoints.get(name)) {
            let cgi = self.chunk_group_infos.expect_get(cgi);

            compilation
              .chunk_group_by_ukey
              .expect_get_mut(&cgi.chunk_group)
              .add_origin(Some(module_id), loc, request);

            compilation
              .chunk_graph
              .connect_block_and_chunk_group(block_id, cgi.chunk_group);
            cgi.ukey
          } else {
            let entry_options = entry_options.clone();
            let chunk = compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);
            if let Some(filename) = &entry_options.filename {
              chunk.set_filename_template(Some(filename.clone()));
            }
            let mut entrypoint = ChunkGroup::new(ChunkGroupKind::new_entrypoint(
              false,
              Box::new(entry_options.clone()),
            ));

            self.stat_chunk_group_created += 1;
            let cgi = ChunkGroupInfo::new(
              entrypoint.ukey,
              RuntimeSpec::from_entry_options(&entry_options)
                .expect("should have runtime for AsyncEntrypoint"),
              entry_options
                .chunk_loading
                .as_ref()
                .map_or(item_chunk_group_info.chunk_loading, |x| {
                  !matches!(x, ChunkLoading::Disable)
                }),
              entry_options
                .async_chunks
                .unwrap_or(item_chunk_group_info.async_chunks),
            );
            let ukey = cgi.ukey;

            self.chunk_group_infos.entry(ukey).or_insert(cgi);

            entrypoint.set_runtime_chunk(chunk.ukey());
            entrypoint.set_entry_point_chunk(chunk.ukey());
            compilation.async_entrypoints.push(entrypoint.ukey);
            self.next_chunk_group_index += 1;
            entrypoint.index = Some(self.next_chunk_group_index);

            if let Some(name) = entrypoint.kind.name() {
              self.named_async_entrypoints.insert(name.to_owned(), ukey);
              compilation
                .named_chunk_groups
                .insert(name.to_owned(), entrypoint.ukey);
            }

            entrypoint.connect_chunk(chunk);

            self.chunk_group_info_map.insert(entrypoint.ukey, ukey);
            compilation
              .chunk_graph
              .connect_block_and_chunk_group(block_id, entrypoint.ukey);

            compilation.chunk_group_by_ukey.add(entrypoint);
            ukey
          };

        let cgi = self.chunk_group_infos.expect_get(&cgi);
        entrypoint = Some(cgi.chunk_group);

        self
          .queue_delayed
          .push(QueueAction::ProcessEntryBlock(ProcessEntryBlock {
            block: block_id,
            module: module_id,
            chunk_group_info: cgi.ukey,
            chunk: chunk_ukey,
          }));
        cgi.ukey
      } else if !item_chunk_group_info.async_chunks || !item_chunk_group_info.chunk_loading {
        self.queue.push(QueueAction::ProcessBlock(ProcessBlock {
          block: block_id.into(),
          module: module_id,
          chunk_group_info: item_chunk_group_info.ukey,
          chunk: item_chunk_ukey,
        }));
        return;
      } else {
        let cgi = if let Some(chunk_name) = chunk_name
          && let Some(cgi) = self.named_chunk_groups.get(chunk_name)
        {
          let mut cgi = self.chunk_group_infos.expect_get(cgi);
          let block = module_graph
            .block_by_id(&block_id)
            .expect("should have block");
          let request = block.request().clone();
          let loc = block.loc();

          if compilation
            .chunk_group_by_ukey
            .expect_get(&cgi.chunk_group)
            .is_initial()
          {
            let error = AsyncDependenciesToInitialChunkError(chunk_name.to_string(), loc.clone());
            compilation.push_diagnostic(Error::from(error).into());
            cgi = item_chunk_group_info;
          }

          compilation
            .chunk_group_by_ukey
            .expect_get_mut(&cgi.chunk_group)
            .add_origin(Some(module_id), loc, request);

          compilation
            .chunk_graph
            .connect_block_and_chunk_group(block_id, cgi.chunk_group);
          cgi
        } else {
          let mut chunk_group = add_chunk_in_group(
            block.get_group_options(),
            module_id,
            block.loc(),
            block.request().clone(),
          );
          let chunk = compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);

          self.stat_chunk_group_created += 1;
          let info = ChunkGroupInfo::new(
            chunk_group.ukey,
            item_chunk_group_info.runtime.clone(),
            item_chunk_group_info.chunk_loading,
            item_chunk_group_info.async_chunks,
          );

          let info_ukey = info.ukey;
          let info = self.chunk_group_infos.entry(info_ukey).or_insert(info);

          self.next_chunk_group_index += 1;
          chunk_group.index = Some(self.next_chunk_group_index);

          if let Some(name) = chunk_group.kind.name() {
            self.named_chunk_groups.insert(name.to_owned(), info_ukey);
            compilation
              .named_chunk_groups
              .insert(name.to_owned(), chunk_group.ukey);
          }

          chunk_group.connect_chunk(chunk);

          self
            .chunk_group_info_map
            .insert(chunk_group.ukey, info_ukey);

          compilation
            .chunk_graph
            .connect_block_and_chunk_group(block_id, chunk_group.ukey);

          compilation.chunk_group_by_ukey.add(chunk_group);
          info
        };
        c = Some(cgi.chunk_group);
        cgi.ukey
      };
      self.block_chunk_groups.insert(
        DependenciesBlockIdentifier::AsyncDependenciesBlock(block_id),
        cgi,
      );
      cgi
    };

    if let Some(c) = c {
      let connect_list = self
        .queue_connect
        .entry(item_chunk_group_info_ukey)
        .or_default();
      let c = compilation.chunk_group_by_ukey.expect_get(&c);

      connect_list.insert((
        cgi,
        Some(ProcessBlock {
          block: block_id.into(),
          module: module_id,
          chunk_group_info: cgi,
          chunk: c.chunks[0],
        }),
      ));
    } else if let Some(entrypoint) = entrypoint {
      let item_chunk_group = compilation
        .chunk_group_by_ukey
        .expect_get_mut(&item_chunk_group);
      item_chunk_group.add_async_entrypoint(entrypoint);
    }
  }

  fn get_block_modules(
    &mut self,
    module: DependenciesBlockIdentifier,
    runtime: Option<&RuntimeSpec>,
    compilation: &mut Compilation,
  ) -> Vec<(ModuleIdentifier, ConnectionState, Vec<DependencyId>)> {
    if let Some(modules) = self
      .block_modules_runtime_map
      .get::<OptionalRuntimeSpec>(&runtime.cloned().into())
      .and_then(|map| map.get(&module))
    {
      return modules.clone();
    }

    self.extract_block_modules(
      module.get_root_block(&compilation.get_module_graph()),
      runtime,
      compilation,
    );
    self
      .block_modules_runtime_map
      .get::<OptionalRuntimeSpec>(&runtime.cloned().into())
      .and_then(|map| map.get(&module))
      .unwrap_or_else(|| {
        panic!("block_modules_map.get({module:?}) must not empty after extract_block_modules")
      })
      .clone()
  }

  fn extract_block_modules(
    &mut self,
    module: ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
    compilation: &mut Compilation,
  ) {
    let module_graph = &compilation.get_module_graph();
    let map = self
      .block_modules_runtime_map
      .entry(runtime.cloned().into())
      .or_default();
    let block = module.into();
    map.insert(block, Vec::new());
    for b in block.get_blocks(compilation) {
      map.insert(b.into(), Vec::new());
    }

    // keep the dependency order sorted by span
    let mut connection_map: IndexMap<
      (DependenciesBlockIdentifier, ModuleIdentifier),
      Vec<DependencyId>,
    > = IndexMap::default();

    for dep_id in module_graph.get_outgoing_connections_in_order(&module) {
      let dep = module_graph
        .dependency_by_id(dep_id)
        .expect("should have dep");
      if dep.as_module_dependency().is_none() && dep.as_context_dependency().is_none() {
        continue;
      }
      if matches!(dep.as_module_dependency().map(|d| d.weak()), Some(true)) {
        continue;
      }
      // Dependency created but no module is available.
      // This could happen when module factorization is failed, but `options.bail` set to `false`
      let module_graph = compilation.get_module_graph();
      let Some(module_identifier) = module_graph.module_identifier_by_dependency_id(dep_id) else {
        continue;
      };
      let block_id = if let Some(block) = compilation.get_module_graph().get_parent_block(dep_id) {
        (*block).into()
      } else {
        module.into()
      };
      connection_map
        .entry((block_id, *module_identifier))
        .and_modify(|e| e.push(*dep_id))
        .or_insert_with(|| vec![*dep_id]);
    }

    for ((block_id, module_identifier), connections) in connection_map {
      let modules = map
        .get_mut(&block_id)
        .expect("should have modules in block_modules_runtime_map");
      let active_state =
        get_active_state_of_connections(&connections, runtime, &compilation.get_module_graph());
      modules.push((module_identifier, active_state, connections));
    }
  }

  fn process_connect_queue(&mut self, compilation: &mut Compilation) {
    // Figure out new parents for chunk groups
    // to get new available modules for these children
    for (chunk_group_info_ukey, targets) in self.queue_connect.drain(..) {
      let chunk_group_info = self
        .chunk_group_infos
        .expect_get_mut(&chunk_group_info_ukey);
      let chunk_group_ukey = chunk_group_info.chunk_group;

      // 1. Add new targets to the list of children
      chunk_group_info
        .children
        .extend(targets.iter().map(|(target, _)| target).copied());

      // 2. Calculate resulting available modules
      let resulting_available_modules =
        chunk_group_info.calculate_resulting_available_modules(compilation, &self.mask_by_chunk);

      let runtime = chunk_group_info.runtime.clone();

      let target_groups = targets.iter().map(|(chunk_group_info_ukey, _)| {
        let cgi = self.chunk_group_infos.expect_get(chunk_group_info_ukey);
        cgi.chunk_group
      });

      compilation
        .chunk_group_by_ukey
        .expect_get_mut(&chunk_group_ukey)
        .children
        .extend(target_groups.clone());

      self.stat_connected_chunk_groups += targets.len() as u32;

      // 3. Update chunk group info
      for (target_ukey, process_block) in targets {
        let target_cgi = self.chunk_group_infos.expect_get_mut(&target_ukey);
        if let Some(process_block) = &process_block {
          let blocks = target_cgi.parents.entry(chunk_group_info_ukey).or_default();
          blocks.push(
            process_block
              .block
              .as_async()
              .expect("should be AsyncDependenciesBlockIdentifier"),
          );
        }

        let target = compilation
          .chunk_group_by_ukey
          .expect_get_mut(&target_cgi.chunk_group);

        target.add_parent(chunk_group_ukey);

        target_cgi
          .available_modules_to_be_merged
          .push(resulting_available_modules.clone());
        self
          .chunk_groups_for_merging
          .insert((target_ukey, process_block));
        let mut updated = false;
        for r in runtime.iter() {
          updated |= target_cgi.runtime.insert(r.clone());
        }
        if updated {
          self.outdated_chunk_group_info.insert(target_ukey);
        }
      }
    }
  }

  fn process_outdated_chunk_group_info(&mut self, compilation: &Compilation) {
    self.stat_chunk_group_info_updated += self.outdated_chunk_group_info.len() as u32;

    // Revisit skipped elements
    for chunk_group_info_ukey in self.outdated_chunk_group_info.drain(..) {
      let cgi = self
        .chunk_group_infos
        .expect_get_mut(&chunk_group_info_ukey);
      let chunk_group = compilation.chunk_group_by_ukey.expect_get(&cgi.chunk_group);

      let origin_queue_len = self.queue.len();

      // 1. Reconsider skipped items
      let add_and_enter_modules = cgi
        .skipped_items
        .iter()
        .filter_map(|module| {
          let ordinal = self.ordinal_by_module.get(module).unwrap_or_else(|| {
            panic!("expected a module ordinal for identifier '{module}', but none was found.")
          });
          if !cgi.min_available_modules.bit(*ordinal) {
            Some(*module)
          } else {
            None
          }
        })
        .collect::<Vec<_>>();

      for m in add_and_enter_modules {
        cgi.skipped_items.shift_remove(&m);

        self
          .queue
          .push(QueueAction::AddAndEnterModule(AddAndEnterModule {
            module: m,
            chunk_group_info: cgi.ukey,
            chunk: chunk_group.chunks[0],
          }))
      }

      // 2. Reconsider skipped connections
      if !cgi.skipped_module_connections.is_empty() {
        let mut active_connections = Vec::new();
        for (i, (module, connections)) in cgi.skipped_module_connections.iter().enumerate() {
          let active_state = get_active_state_of_connections(
            connections,
            Some(&cgi.runtime),
            &compilation.get_module_graph(),
          );
          if active_state.is_false() {
            continue;
          }
          if active_state.is_true() {
            active_connections.push(i);
            let module_ordinal = self.ordinal_by_module.get(module).unwrap_or_else(|| {
              panic!("expected a module ordinal for identifier '{module}', but none was found.")
            });
            if cgi.min_available_modules.bit(*module_ordinal) {
              cgi.skipped_items.insert(*module);
              continue;
            }
          }
          self.queue.push(if active_state.is_true() {
            QueueAction::AddAndEnterModule(AddAndEnterModule {
              module: *module,
              chunk_group_info: chunk_group_info_ukey,
              chunk: chunk_group.chunks[0],
            })
          } else {
            QueueAction::ProcessBlock(ProcessBlock {
              block: (*module).into(),
              module: *module,
              chunk_group_info: chunk_group_info_ukey,
              chunk: chunk_group.chunks[0],
            })
          })
        }

        active_connections.reverse();
        for i in active_connections {
          cgi.skipped_module_connections.shift_remove_index(i);
        }
      }

      // 3. Reconsider children chunk groups
      if !cgi.children.is_empty() {
        self.stat_child_chunk_groups_reconnected += cgi.children.len() as u32;

        let connect_list = self.queue_connect.entry(chunk_group_info_ukey).or_default();
        connect_list.extend(cgi.children.iter().map(|child| (*child, None)));
      }

      // 4. Reconsider chunk groups for combining
      for cgi in &cgi.available_children {
        self.chunk_groups_for_combining.insert(*cgi);
      }

      if origin_queue_len != self.queue.len() {
        self.outdated_order_index_chunk_groups.insert(cgi.ukey);
      }
    }
  }

  fn process_chunk_groups_for_combining(&mut self, compilation: &mut Compilation) {
    self.chunk_groups_for_combining.retain(|info_ukey| {
      let info = self.chunk_group_infos.expect_get(info_ukey);
      info.available_sources.iter().all(|source_ukey| {
        let source = self.chunk_group_infos.expect_get(source_ukey);
        source.min_available_modules_init
      })
    });

    let mut min_available_modules_mappings = IndexMap::<CgiUkey, BigUint>::default();
    for info_ukey in &self.chunk_groups_for_combining {
      let info_ukey = *info_ukey;
      let info = self.chunk_group_infos.expect_get(&info_ukey);
      let mut available_modules = BigUint::from(0u32);

      // combine min_available_modules from all resulting_available_modules
      for source_ukey in info.available_sources.clone() {
        let source = self.chunk_group_infos.expect_get_mut(&source_ukey);
        let resulting_available_modules =
          source.calculate_resulting_available_modules(compilation, &self.mask_by_chunk);
        available_modules |= resulting_available_modules;
      }

      let info = self.chunk_group_infos.expect_get_mut(&info_ukey);
      min_available_modules_mappings.insert(info_ukey, available_modules);
      info.invalidate_resulting_available_modules();
      self.outdated_chunk_group_info.insert(info_ukey);
    }
    for (info_ukey, min_available_modules) in min_available_modules_mappings {
      let info = self.chunk_group_infos.expect_get_mut(&info_ukey);
      info.min_available_modules = min_available_modules.clone();
    }

    self.chunk_groups_for_combining.clear();
  }

  fn process_chunk_groups_for_merging(&mut self, compilation: &mut Compilation) {
    self.stat_processed_chunk_groups_for_merging += self.chunk_groups_for_merging.len() as u32;
    let chunk_groups_for_merging = std::mem::take(&mut self.chunk_groups_for_merging);
    for (info_ukey, process_block) in chunk_groups_for_merging {
      let cgi = self.chunk_group_infos.expect_get_mut(&info_ukey);

      let mut changed = false;

      if !cgi.available_modules_to_be_merged.is_empty() {
        let available_modules_to_be_merged =
          std::mem::take(&mut cgi.available_modules_to_be_merged);

        self.stat_merged_available_module_sets += available_modules_to_be_merged.len() as u32;

        for modules_to_be_merged in available_modules_to_be_merged {
          if !cgi.min_available_modules_init {
            cgi.min_available_modules_init = true;
            cgi.min_available_modules = modules_to_be_merged;
            changed = true;
            continue;
          }

          let orig = cgi.min_available_modules.clone();
          cgi.min_available_modules &= modules_to_be_merged;
          changed |= orig != cgi.min_available_modules;
        }
      }

      if changed {
        cgi.invalidate_resulting_available_modules();
        self.outdated_chunk_group_info.insert(info_ukey);
      }

      if let Some(process_block) = process_block {
        let initialized = cgi.initialized;
        let mut needs_walk = !initialized || changed;

        let blocks = self.blocks_by_cgi.entry(cgi.ukey).or_default();
        if blocks.insert(process_block.block) {
          needs_walk = true;
        }

        if needs_walk {
          cgi.initialized = true;

          // check if we can use cache to initialize it
          if !initialized && self.recover_from_cache(info_ukey, compilation) {
            self.stat_use_cache += 1;
            continue;
          }

          self
            .queue_delayed
            .push(QueueAction::ProcessBlock(process_block));
        }
      }
    }
  }
}

#[derive(Debug, Clone)]
enum QueueAction {
  AddAndEnterEntryModule(AddAndEnterEntryModule),
  AddAndEnterModule(AddAndEnterModule),
  _EnterModule(EnterModule),
  ProcessBlock(ProcessBlock),
  ProcessEntryBlock(ProcessEntryBlock),
  LeaveModule(LeaveModule),
}

#[derive(Debug, Clone)]
struct AddAndEnterEntryModule {
  module: ModuleIdentifier,
  chunk_group_info: CgiUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone)]
struct AddAndEnterModule {
  module: ModuleIdentifier,
  chunk_group_info: CgiUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone)]
struct EnterModule {
  module: ModuleIdentifier,
  chunk_group_info: CgiUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ProcessBlock {
  module: ModuleIdentifier,
  block: DependenciesBlockIdentifier,
  chunk_group_info: CgiUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone)]
struct ProcessEntryBlock {
  module: ModuleIdentifier,
  block: AsyncDependenciesBlockIdentifier,
  chunk_group_info: CgiUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DependenciesBlockIdentifier {
  Module(ModuleIdentifier),
  AsyncDependenciesBlock(AsyncDependenciesBlockIdentifier),
}

impl DependenciesBlockIdentifier {
  pub fn get_root_block<'a>(&'a self, module_graph: &'a ModuleGraph) -> ModuleIdentifier {
    match self {
      DependenciesBlockIdentifier::Module(m) => *m,
      DependenciesBlockIdentifier::AsyncDependenciesBlock(id) => *module_graph
        .block_by_id(id)
        .expect("should have block")
        .parent(),
    }
  }

  pub fn get_blocks(&self, compilation: &Compilation) -> Vec<AsyncDependenciesBlockIdentifier> {
    match self {
      DependenciesBlockIdentifier::Module(m) => compilation
        .get_module_graph()
        .module_by_identifier(m)
        .expect("should have module")
        .get_blocks()
        .to_vec(),
      DependenciesBlockIdentifier::AsyncDependenciesBlock(a) => compilation
        .get_module_graph()
        .block_by_id(a)
        .expect("should have block")
        .get_blocks()
        .to_vec(),
    }
  }

  pub fn as_async(self) -> Option<AsyncDependenciesBlockIdentifier> {
    if let DependenciesBlockIdentifier::AsyncDependenciesBlock(id) = self {
      Some(id)
    } else {
      None
    }
  }
}

impl From<ModuleIdentifier> for DependenciesBlockIdentifier {
  fn from(value: ModuleIdentifier) -> Self {
    Self::Module(value)
  }
}

impl From<AsyncDependenciesBlockIdentifier> for DependenciesBlockIdentifier {
  fn from(value: AsyncDependenciesBlockIdentifier) -> Self {
    Self::AsyncDependenciesBlock(value)
  }
}

#[derive(Debug, Clone)]
struct LeaveModule {
  module: ModuleIdentifier,
  chunk_group_info: CgiUkey,
}
