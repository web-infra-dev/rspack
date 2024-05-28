use std::cell::RefCell;
use std::hash::{BuildHasherDefault, Hash};
use std::sync::Arc;

use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use num_bigint::BigUint;
use rspack_database::{Database, Ukey};
use rspack_error::{error, Error, Result};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};

use crate::dependencies_block::AsyncDependenciesToInitialChunkError;
use crate::{
  add_connection_states, assign_depth, assign_depths, get_entry_runtime,
  AsyncDependenciesBlockIdentifier, ChunkGroup, ChunkGroupKind, ChunkGroupOptions, ChunkGroupUkey,
  ChunkLoading, ChunkUkey, Compilation, ConnectionId, ConnectionState, DependenciesBlock,
  EntryRuntime, GroupOptions, Logger, ModuleGraph, ModuleIdentifier, RuntimeSpec,
};

#[derive(Debug, Clone)]
pub struct ChunkGroupInfo {
  pub ukey: CgiUkey,
  pub chunk_group: ChunkGroupUkey,
  pub chunk_loading: bool,
  pub async_chunks: bool,
  pub runtime: RuntimeSpec,
  pub min_available_modules: BigUint,
  pub min_available_modules_init: bool,
  pub available_modules_to_be_merged: Vec<BigUint>,

  pub skipped_items: IndexSet<ModuleIdentifier>,
  pub skipped_module_connections:
    IndexSet<(ModuleIdentifier, Vec<ConnectionId>), BuildHasherDefault<FxHasher>>,
  // set of children chunk groups, that will be revisited when available_modules shrink
  pub children: IndexSet<CgiUkey>,
  // set of chunk groups that are the source for min_available_modules
  pub available_sources: IndexSet<CgiUkey>,
  // set of chunk groups which depend on the this chunk group as available_source
  pub available_children: IndexSet<CgiUkey>,

  // set of modules available including modules from this chunk group
  // A derived attribute, therefore utilizing interior mutability to manage updates
  resulting_available_modules: RefCell<Option<BigUint>>,
}

impl ChunkGroupInfo {
  pub fn new(
    chunk_group: ChunkGroupUkey,
    runtime: RuntimeSpec,
    chunk_loading: bool,
    async_chunks: bool,
  ) -> Self {
    Self {
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
      available_sources: Default::default(),
      available_children: Default::default(),
      resulting_available_modules: Default::default(),
    }
  }

  fn calculate_resulting_available_modules(
    &self,
    compilation: &Compilation,
    mask_by_chunk: &HashMap<ChunkUkey, BigUint>,
  ) -> BigUint {
    let mut resulting_available_modules = self.resulting_available_modules.borrow_mut();
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

  fn invalidate_resulting_available_modules(&self) {
    let mut resulting_available_modules = self.resulting_available_modules.borrow_mut();
    *resulting_available_modules = None;
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct OptionalRuntimeSpec(pub Vec<Arc<str>>);

impl From<Option<RuntimeSpec>> for OptionalRuntimeSpec {
  fn from(value: Option<RuntimeSpec>) -> Self {
    let mut vec = value.unwrap_or_default().into_iter().collect::<Vec<_>>();
    vec.sort();
    Self(vec)
  }
}

type CgiUkey = Ukey<ChunkGroupInfo>;

type BlockModulesRuntimeMap = IndexMap<
  OptionalRuntimeSpec,
  IndexMap<
    DependenciesBlockIdentifier,
    Vec<(ModuleIdentifier, ConnectionState, Vec<ConnectionId>)>,
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

pub(super) struct CodeSplitter<'me> {
  chunk_group_info_map: HashMap<ChunkGroupUkey, CgiUkey>,
  chunk_group_infos: Database<ChunkGroupInfo>,
  outdated_order_index_chunk_groups: HashSet<CgiUkey>,
  block_by_cgi: HashMap<CgiUkey, AsyncDependenciesBlockIdentifier>,
  pub(super) compilation: &'me mut Compilation,
  next_free_module_pre_order_index: u32,
  next_free_module_post_order_index: u32,
  next_chunk_group_index: u32,
  queue: Vec<QueueAction>,
  queue_delayed: Vec<QueueAction>,
  queue_connect: IndexMap<CgiUkey, IndexSet<CgiUkey>>,
  chunk_groups_for_combining: IndexSet<CgiUkey>,
  outdated_chunk_group_info: IndexSet<CgiUkey>,
  chunk_groups_for_merging: IndexSet<CgiUkey>,
  block_chunk_groups: HashMap<AsyncDependenciesBlockIdentifier, CgiUkey>,
  named_chunk_groups: HashMap<String, CgiUkey>,
  named_async_entrypoints: HashMap<String, CgiUkey>,
  block_modules_runtime_map: BlockModulesRuntimeMap,
  ordinal_by_module: HashMap<ModuleIdentifier, u64>,
  mask_by_chunk: HashMap<ChunkUkey, BigUint>,
}

fn add_chunk_in_group(group_options: Option<&GroupOptions>) -> ChunkGroup {
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
  );
  let kind = ChunkGroupKind::Normal { options };
  ChunkGroup::new(kind)
}

fn get_active_state_of_connections(
  connections: &[ConnectionId],
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
) -> ConnectionState {
  let mut iter = connections.iter();
  let id = iter.next().expect("should have connection");
  let mut merged = module_graph
    .connection_by_connection_id(id)
    .expect("should have connection")
    .get_active_state(module_graph, runtime);
  if merged.is_true() {
    return merged;
  }
  for c in iter {
    let c = module_graph
      .connection_by_connection_id(c)
      .expect("should have connection");
    merged = add_connection_states(merged, c.get_active_state(module_graph, runtime));
    if merged.is_true() {
      return merged;
    }
  }
  merged
}

impl<'me> CodeSplitter<'me> {
  pub fn new(compilation: &'me mut Compilation) -> Self {
    // This optimization is inspired from  https://github.com/webpack/webpack/pull/18090 by https://github.com/dmichon-msft
    // Thanks!
    let mut ordinal_by_module = HashMap::default();
    for (index, m) in compilation.get_module_graph().modules().keys().enumerate() {
      ordinal_by_module.insert(*m, index as u64);
    }

    let module_graph = compilation.get_module_graph();
    let mut mask_by_chunk = HashMap::default();
    for chunk in compilation.chunk_by_ukey.keys() {
      let mut mask = BigUint::from(0u32);
      for module in compilation
        .chunk_graph
        .get_chunk_modules(chunk, &module_graph)
      {
        let module_id = module.identifier();
        let module_ordinal = ordinal_by_module.get(&module_id).unwrap_or_else(|| {
          panic!(
            "expected a module ordinal for identifier '{}', but none was found.",
            &module_id
          )
        });
        mask.set_bit(*module_ordinal, true);
      }
      mask_by_chunk.insert(*chunk, mask);
    }

    CodeSplitter {
      chunk_group_info_map: Default::default(),
      chunk_group_infos: Default::default(),
      outdated_order_index_chunk_groups: Default::default(),
      block_by_cgi: Default::default(),
      compilation,
      next_free_module_pre_order_index: 0,
      next_free_module_post_order_index: 0,
      next_chunk_group_index: 0,
      queue: Default::default(),
      queue_delayed: Default::default(),
      queue_connect: Default::default(),
      chunk_groups_for_combining: Default::default(),
      outdated_chunk_group_info: Default::default(),
      chunk_groups_for_merging: Default::default(),
      block_chunk_groups: Default::default(),
      named_chunk_groups: Default::default(),
      named_async_entrypoints: Default::default(),
      block_modules_runtime_map: Default::default(),
      ordinal_by_module,
      mask_by_chunk,
    }
  }

  fn prepare_input_entrypoints_and_modules(
    &mut self,
  ) -> Result<IndexMap<ChunkGroupUkey, Vec<ModuleIdentifier>>> {
    let mut input_entrypoints_and_modules: IndexMap<ChunkGroupUkey, Vec<ModuleIdentifier>> =
      IndexMap::default();
    let mut assign_depths_map: std::collections::HashMap<
      rspack_identifier::Identifier,
      usize,
      BuildHasherDefault<FxHasher>,
    > = HashMap::default();

    let entries = self.compilation.entries.clone();
    for (name, entry_data) in entries {
      let options = &entry_data.options;
      let dependencies = [
        self.compilation.global_entry.dependencies.clone(),
        entry_data.dependencies.clone(),
      ]
      .concat();
      let module_identifiers = dependencies
        .iter()
        .filter_map(|dep| {
          self
            .compilation
            .get_module_graph()
            .module_identifier_by_dependency_id(dep)
            .cloned()
        })
        .collect::<Vec<_>>();

      let chunk_ukey = Compilation::add_named_chunk(
        name.to_string(),
        &mut self.compilation.chunk_by_ukey,
        &mut self.compilation.named_chunks,
      );
      self.mask_by_chunk.insert(chunk_ukey, BigUint::from(0u32));
      let runtime = get_entry_runtime(&name, options, &self.compilation.entries);
      let chunk = self.compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);
      if let Some(filename) = &entry_data.options.filename {
        chunk.filename_template = Some(filename.clone().into());
      }
      chunk.chunk_reasons.push(format!("Entrypoint({name})",));

      self.compilation.chunk_graph.add_chunk(chunk.ukey);

      let mut entrypoint = ChunkGroup::new(ChunkGroupKind::new_entrypoint(
        true,
        Box::new(options.clone()),
      ));

      let chunk_group_info = {
        let mut cgi = ChunkGroupInfo::new(
          entrypoint.ukey,
          runtime,
          !matches!(
            options
              .chunk_loading
              .as_ref()
              .unwrap_or(&self.compilation.options.output.chunk_loading),
            ChunkLoading::Disable
          ),
          options
            .async_chunks
            .unwrap_or(self.compilation.options.output.async_chunks),
        );
        cgi.min_available_modules_init = true;
        cgi
      };

      if options.depend_on.is_none() && !matches!(&options.runtime, Some(EntryRuntime::String(_))) {
        entrypoint.set_runtime_chunk(chunk.ukey);
      }
      entrypoint.set_entry_point_chunk(chunk.ukey);
      entrypoint.connect_chunk(chunk);

      self
        .compilation
        .named_chunk_groups
        .insert(name.to_string(), entrypoint.ukey);

      self
        .compilation
        .entrypoints
        .insert(name.to_string(), entrypoint.ukey);

      let entrypoint = {
        let ukey = entrypoint.ukey;
        self.compilation.chunk_group_by_ukey.add(entrypoint);
        self.compilation.chunk_group_by_ukey.expect_get(&ukey)
      };

      for module_identifier in module_identifiers.iter() {
        self.compilation.chunk_graph.add_module(*module_identifier);

        input_entrypoints_and_modules
          .entry(entrypoint.ukey)
          .or_default()
          .push(*module_identifier);

        self.compilation.chunk_graph.connect_chunk_and_entry_module(
          chunk.ukey,
          *module_identifier,
          entrypoint.ukey,
        );
      }
      assign_depths(
        &mut assign_depths_map,
        &self.compilation.get_module_graph(),
        module_identifiers.iter().collect_vec(),
      );

      let module_graph = self.compilation.get_module_graph();
      let global_included_modules = self
        .compilation
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
      for included_module in included_modules.clone() {
        assign_depth(
          &mut assign_depths_map,
          &self.compilation.get_module_graph(),
          included_module,
        );
      }
      input_entrypoints_and_modules
        .entry(entrypoint.ukey)
        .or_default()
        .extend(included_modules);

      let cgi = chunk_group_info.ukey;
      self
        .chunk_group_infos
        .entry(cgi)
        .or_insert(chunk_group_info);

      self.chunk_group_info_map.insert(entrypoint.ukey, cgi);
      if let Some(name) = entrypoint.name() {
        self.named_chunk_groups.insert(name.to_string(), cgi);
      }
    }

    // Using this defer insertion strategies to workaround rustc borrow rules
    for (k, v) in assign_depths_map {
      self.compilation.get_module_graph_mut().set_depth(k, v);
    }

    let mut runtime_chunks = IndexSet::<ChunkUkey>::default();
    let mut runtime_errors = vec![];
    for (name, entry_data) in &self.compilation.entries {
      let options = &entry_data.options;
      let runtime = &options.runtime;
      let depend_on = &options.depend_on;

      if depend_on.is_some() && runtime.is_some() {
        runtime_errors.push(Some(error!(
          "Entrypoint '{name}' has 'dependOn' and 'runtime' specified. This is not valid.
Entrypoints that depend on other entrypoints do not have their own runtime.
They will use the runtime(s) from referenced entrypoints instead.
Remove the 'runtime' option from the entrypoint."
        )));
      }

      if let Some(depend_on) = &options.depend_on {
        let ukey = self
          .compilation
          .entrypoints
          .get(name)
          .ok_or_else(|| error!("no entrypoints found"))?;

        let mut entry_point_runtime = None;
        let mut depend_on_entries = vec![];
        let mut entry_point_parents = vec![];
        let mut has_error = false;

        {
          let entry_point = self.compilation.chunk_group_by_ukey.expect_get(ukey);
          let entry_point_chunk = self
            .compilation
            .chunk_by_ukey
            .expect_get(&entry_point.get_entry_point_chunk());
          let referenced_chunks =
            entry_point_chunk.get_all_referenced_chunks(&self.compilation.chunk_group_by_ukey);

          for dep in depend_on {
            if let Some(dependency_ukey) = self.compilation.entrypoints.get(dep) {
              let dependency_chunk_ukey = self
                .compilation
                .chunk_group_by_ukey
                .expect_get(dependency_ukey)
                .get_entry_point_chunk();
              if referenced_chunks.contains(&dependency_chunk_ukey) {
                runtime_errors.push(Some(error!(
                "Entrypoints '{name}' and '{dep}' use 'dependOn' to depend on each other in a circular way."
              )));
                entry_point_runtime = Some(entry_point_chunk.ukey);
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
          let entry_point = self.compilation.chunk_group_by_ukey.expect_get_mut(ukey);
          entry_point.set_runtime_chunk(entry_point_runtime.expect("Should set runtime chunk"));
          continue;
        } else {
          {
            for depend in depend_on_entries {
              let depend_chunk_group = self.compilation.chunk_group_by_ukey.expect_get_mut(depend);
              if depend_chunk_group.add_child(*ukey) {
                entry_point_parents.push(*depend);
              }
            }
          }
          let entry_point = self.compilation.chunk_group_by_ukey.expect_get_mut(ukey);
          for parent in entry_point_parents {
            entry_point.add_parent(parent);
          }
        }
      } else if let Some(EntryRuntime::String(runtime)) = &options.runtime {
        let ukey = self
          .compilation
          .entrypoints
          .get(name)
          .ok_or_else(|| error!("no entrypoints found"))?;

        let entry_point = self.compilation.chunk_group_by_ukey.expect_get_mut(ukey);

        let chunk = match self.compilation.named_chunks.get(runtime) {
          Some(ukey) => {
            if !runtime_chunks.contains(ukey) {
              runtime_errors.push(Some(error!(
"Entrypoint '{name}' has a 'runtime' option which points to another entrypoint named '{runtime}'.
It's not valid to use other entrypoints as runtime chunk.
Did you mean to use 'dependOn: \"{runtime}\"' instead to allow using entrypoint '{name}' within the runtime of entrypoint '{runtime}'? For this '{runtime}' must always be loaded when '{name}' is used.
Or do you want to use the entrypoints '{name}' and '{runtime}' independently on the same page with a shared runtime? In this case give them both the same value for the 'runtime' option. It must be a name not already used by an entrypoint."
              )));
              let entry_chunk = entry_point.get_entry_point_chunk();
              entry_point.set_runtime_chunk(entry_chunk);
              continue;
            }
            self.compilation.chunk_by_ukey.expect_get_mut(ukey)
          }
          None => {
            let chunk_ukey = Compilation::add_named_chunk(
              runtime.to_string(),
              &mut self.compilation.chunk_by_ukey,
              &mut self.compilation.named_chunks,
            );
            self.mask_by_chunk.insert(chunk_ukey, BigUint::from(0u32));
            let chunk = self.compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);
            chunk.prevent_integration = true;
            chunk.chunk_reasons.push(format!("RuntimeChunk({name})",));
            self.compilation.chunk_graph.add_chunk(chunk.ukey);
            runtime_chunks.insert(chunk.ukey);
            chunk
          }
        };

        entry_point.unshift_chunk(chunk);
        chunk.add_group(entry_point.ukey);
        entry_point.set_runtime_chunk(chunk.ukey);
      }
    }

    for err in runtime_errors.into_iter().flatten() {
      self.compilation.push_diagnostic(err.into());
    }

    Ok(input_entrypoints_and_modules)
  }

  #[tracing::instrument(skip_all)]
  pub fn split(mut self) -> Result<()> {
    let logger = self.compilation.get_logger("rspack.buildChunkGraph");
    let start = logger.time("prepare entrypoints");
    let input_entrypoints_and_modules = self.prepare_input_entrypoints_and_modules()?;
    logger.time_end(start);

    for (chunk_group, modules) in input_entrypoints_and_modules {
      let chunk_group = self
        .compilation
        .chunk_group_by_ukey
        .expect_get_mut(&chunk_group);

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
        chunk_group_info.skipped_items = IndexSet::from_iter(modules);
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
      let chunk_group = self
        .compilation
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

    // pop() is used to read from the queue
    // so it need to be reversed to be iterated in
    // correct order
    self.queue.reverse();

    let start = logger.time("process queue");
    // Iterative traversal of the Module graph
    // Recursive would be simpler to write but could result in Stack Overflows
    while !self.queue.is_empty() || !self.queue_connect.is_empty() {
      self.process_queue();

      if !self.chunk_groups_for_combining.is_empty() {
        self.process_chunk_groups_for_combining();
      }

      if !self.queue_connect.is_empty() {
        self.process_connect_queue();

        if !self.chunk_groups_for_merging.is_empty() {
          self.process_chunk_groups_for_merging();
        }
      }

      if !self.outdated_chunk_group_info.is_empty() {
        self.process_outdated_chunk_group_info();
      }

      if self.queue.is_empty() {
        self.queue = std::mem::replace(&mut self.queue_delayed, self.queue);
        self.queue.reverse();
      }
    }
    logger.time_end(start);

    let start = logger.time("extend chunkGroup runtime");
    for (chunk_group, cgi) in &self.chunk_group_info_map {
      let chunk_group = self.compilation.chunk_group_by_ukey.expect_get(chunk_group);
      let cgi = self.chunk_group_infos.expect_get(cgi);
      for chunk_ukey in chunk_group.chunks.iter() {
        self
          .compilation
          .chunk_by_ukey
          .entry(*chunk_ukey)
          .and_modify(|chunk| {
            chunk.runtime.extend(cgi.runtime.clone());
          });
      }
    }
    logger.time_end(start);

    let outdated_order_index_chunk_groups =
      std::mem::take(&mut self.outdated_order_index_chunk_groups);

    for outdated in outdated_order_index_chunk_groups {
      let cgi = self.chunk_group_infos.expect_get(&outdated);
      let chunk_group_ukey = cgi.chunk_group;
      let runtime = cgi.runtime.clone();
      let chunk_group = self
        .compilation
        .chunk_group_by_ukey
        .expect_get_mut(&chunk_group_ukey);

      chunk_group.next_pre_order_index = 0;
      chunk_group.next_post_order_index = 0;

      let module_graph = self.compilation.get_module_graph();
      let Some(block) = self.block_by_cgi.get(&cgi.ukey).copied() else {
        continue;
      };
      let Some(block) = module_graph.block_by_id(&block) else {
        continue;
      };
      let blocks = block
        .get_dependencies()
        .iter()
        .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
        .copied()
        .collect::<Vec<_>>();

      let mut visited = IndexSet::default();

      for root in blocks {
        let mut ctx = (0, 0, Default::default());
        self.calculate_order_index(root, &runtime, &mut visited, &mut ctx);

        let chunk_group = self
          .compilation
          .chunk_group_by_ukey
          .expect_get_mut(&chunk_group_ukey);
        for (id, (pre, post)) in ctx.2 {
          chunk_group.module_pre_order_indices.insert(id, pre);
          chunk_group.module_post_order_indices.insert(id, post);
        }
      }
    }

    // make sure all module (weak dependency particularly) has a mgm
    let ids = self
      .compilation
      .get_module_graph()
      .modules()
      .keys()
      .cloned()
      .collect::<Vec<_>>();
    for module_identifier in ids {
      self.compilation.chunk_graph.add_module(module_identifier)
    }

    Ok(())
  }

  fn calculate_order_index(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: &RuntimeSpec,
    visited: &mut IndexSet<ModuleIdentifier>,
    ctx: &mut (usize, usize, IndexMap<ModuleIdentifier, (usize, usize)>),
  ) {
    let block_modules = self.get_block_modules(module_identifier.into(), Some(runtime));
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

      self.calculate_order_index(*module, runtime, visited, ctx);
    }

    let indices = ctx.2.entry(module_identifier).or_default();

    indices.1 = ctx.1;
    ctx.1 += 1;
  }

  fn process_queue(&mut self) {
    tracing::trace!("process_queue");
    while let Some(action) = self.queue.pop() {
      match action {
        QueueAction::AddAndEnterEntryModule(i) => self.add_and_enter_entry_module(&i),
        QueueAction::AddAndEnterModule(i) => self.add_and_enter_module(&i),
        QueueAction::_EnterModule(i) => self.enter_module(&i),
        QueueAction::ProcessBlock(i) => self.process_block(&i),
        QueueAction::ProcessEntryBlock(i) => self.process_entry_block(&i),
        QueueAction::LeaveModule(i) => self.leave_module(&i),
      }
    }
  }

  fn add_and_enter_entry_module(&mut self, item: &AddAndEnterEntryModule) {
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

    if self
      .compilation
      .chunk_graph
      .is_module_in_chunk(&item.module, item.chunk)
    {
      return;
    }

    if cgi.min_available_modules.bit(*module_ordinal) {
      cgi.skipped_items.insert(item.module);
      return;
    }

    self.compilation.chunk_graph.connect_chunk_and_entry_module(
      item.chunk,
      item.module,
      cgi.chunk_group,
    );
    self.add_and_enter_module(&AddAndEnterModule {
      module: item.module,
      chunk_group_info: item.chunk_group_info,
      chunk: item.chunk,
    })
  }

  fn add_and_enter_module(&mut self, item: &AddAndEnterModule) {
    tracing::trace!("add_and_enter_module {:?}", item);
    let cgi = self
      .chunk_group_infos
      .expect_get_mut(&item.chunk_group_info);

    if self
      .compilation
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

    self.compilation.chunk_graph.add_module(item.module);
    self
      .compilation
      .chunk_graph
      .connect_chunk_and_module(item.chunk, item.module);

    let chunk_mask = self
      .mask_by_chunk
      .get_mut(&item.chunk)
      .expect("chunk must in mask_by_chunk");
    chunk_mask.set_bit(*module_ordinal, true);

    self.enter_module(&EnterModule {
      module: item.module,
      chunk_group_info: item.chunk_group_info,
      chunk: item.chunk,
    })
  }

  fn enter_module(&mut self, item: &EnterModule) {
    tracing::trace!("enter_module {:?}", item);
    let cgi = self.chunk_group_infos.expect_get(&item.chunk_group_info);

    let chunk_group = self
      .compilation
      .chunk_group_by_ukey
      .expect_get_mut(&cgi.chunk_group);

    if chunk_group
      .module_pre_order_indices
      .get(&item.module)
      .is_none()
    {
      chunk_group
        .module_pre_order_indices
        .insert(item.module, chunk_group.next_pre_order_index);
      chunk_group.next_pre_order_index += 1;
    }

    {
      let mut module_graph = self.compilation.get_module_graph_mut();
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
    self.process_block(&ProcessBlock {
      block: item.module.into(),
      chunk_group_info: item.chunk_group_info,
      chunk: item.chunk,
    })
  }

  fn leave_module(&mut self, item: &LeaveModule) {
    tracing::trace!("leave_module {:?}", item);
    let cgi = self.chunk_group_infos.expect_get(&item.chunk_group_info);

    let chunk_group = self
      .compilation
      .chunk_group_by_ukey
      .expect_get_mut(&cgi.chunk_group);

    if chunk_group
      .module_post_order_indices
      .get(&item.module)
      .is_none()
    {
      chunk_group
        .module_post_order_indices
        .insert(item.module, chunk_group.next_post_order_index);
      chunk_group.next_post_order_index += 1;
    }

    let mut module_graph = self.compilation.get_module_graph_mut();
    let module = module_graph
      .module_graph_module_by_identifier_mut(&item.module)
      .unwrap_or_else(|| panic!("no module found: {:?}", &item.module));

    if module.post_order_index.is_none() {
      module.post_order_index = Some(self.next_free_module_post_order_index);
      self.next_free_module_post_order_index += 1;
    }
  }

  fn process_entry_block(&mut self, item: &ProcessEntryBlock) {
    tracing::trace!("process_entry_block {:?}", item);
    let chunk_group_info = self.chunk_group_infos.expect_get(&item.chunk_group_info);

    let runtime = chunk_group_info.runtime.clone();

    let modules = self.get_block_modules(item.block.into(), Some(&runtime));

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
        }));
      }
    }
    let blocks = self
      .compilation
      .get_module_graph()
      .block_by_id(&item.block)
      .expect("should have block")
      .get_blocks()
      .to_vec();
    for block in blocks {
      self.iterator_block(block, item.chunk_group_info, item.chunk);
    }
  }

  fn process_block(&mut self, item: &ProcessBlock) {
    tracing::trace!("process_block {:?}", item);
    let item_chunk_group_info = self.chunk_group_infos.expect_get(&item.chunk_group_info);

    let runtime = item_chunk_group_info.runtime.clone();
    let modules = self.get_block_modules(item.block, Some(&runtime));
    for (module, active_state, connections) in modules.into_iter().rev() {
      if self
        .compilation
        .chunk_graph
        .is_module_in_chunk(&module, item.chunk)
      {
        continue;
      }

      if !active_state.is_true() {
        let cgi = self
          .chunk_group_infos
          .expect_get_mut(&item.chunk_group_info);
        cgi.skipped_module_connections.insert((module, connections));
        if active_state.is_false() {
          continue;
        }
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
        }));
      }
    }
    let blocks = item.block.get_blocks(self.compilation);
    for block in blocks {
      self.iterator_block(block, item.chunk_group_info, item.chunk);
    }
  }

  fn iterator_block(
    &mut self,
    block_id: AsyncDependenciesBlockIdentifier,
    item_chunk_group_info_ukey: CgiUkey,
    item_chunk_ukey: ChunkUkey,
  ) {
    let item_chunk_group_info = self
      .chunk_group_infos
      .expect_get(&item_chunk_group_info_ukey);

    let item_chunk_group = item_chunk_group_info.chunk_group;
    let cgi: Option<&Ukey<ChunkGroupInfo>> = self.block_chunk_groups.get(&block_id);

    let is_already_split = cgi.is_some();
    let mut entrypoint: Option<ChunkGroupUkey> = None;
    let mut c: Option<ChunkGroupUkey> = None;

    let chunk_ukey = if let Some(chunk_name) = self
      .compilation
      .get_module_graph()
      .block_by_id(&block_id)
      .expect("should have block")
      .get_group_options()
      .and_then(|x| x.name())
    {
      Compilation::add_named_chunk(
        chunk_name.to_string(),
        &mut self.compilation.chunk_by_ukey,
        &mut self.compilation.named_chunks,
      )
    } else {
      Compilation::add_chunk(&mut self.compilation.chunk_by_ukey)
    };
    self.compilation.chunk_graph.add_chunk(chunk_ukey);
    self.mask_by_chunk.insert(chunk_ukey, BigUint::from(0u32));

    let module_graph = self.compilation.get_module_graph();
    let block = module_graph
      .block_by_id(&block_id)
      .expect("should have block");
    let entry_options = block.get_group_options().and_then(|o| o.entry_options());
    let cgi = if let Some(cgi) = cgi {
      let cgi = self.chunk_group_infos.expect_get(cgi);
      if entry_options.is_some() {
        entrypoint = Some(cgi.chunk_group);
      } else {
        c = Some(cgi.chunk_group);
      }

      cgi.ukey
    } else {
      let chunk_name = block.get_group_options().and_then(|o| o.name());
      let cgi = if let Some(entry_options) = entry_options {
        let cgi =
          if let Some(cgi) = chunk_name.and_then(|name| self.named_async_entrypoints.get(name)) {
            let cgi = self.chunk_group_infos.expect_get(cgi);
            self
              .compilation
              .chunk_graph
              .connect_block_and_chunk_group(block_id, cgi.chunk_group);
            cgi.ukey
          } else {
            let entry_options = entry_options.clone();
            let chunk = self.compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);
            if let Some(filename) = &entry_options.filename {
              chunk.filename_template = Some(filename.clone().into());
            }
            chunk
              .chunk_reasons
              .push(format!("AsyncEntrypoint({:?})", block_id));
            let mut entrypoint = ChunkGroup::new(ChunkGroupKind::new_entrypoint(
              false,
              Box::new(entry_options.clone()),
            ));

            let cgi = ChunkGroupInfo::new(
              entrypoint.ukey,
              RuntimeSpec::from_entry_options(&entry_options)
                .expect("should have runtime for AsyncEntrypoint"),
              entry_options
                .chunk_loading
                .as_ref()
                .map(|x| !matches!(x, ChunkLoading::Disable))
                .unwrap_or(item_chunk_group_info.chunk_loading),
              entry_options
                .async_chunks
                .unwrap_or(item_chunk_group_info.async_chunks),
            );
            let ukey = cgi.ukey;

            self.chunk_group_infos.entry(ukey).or_insert(cgi);

            entrypoint.set_runtime_chunk(chunk.ukey);
            entrypoint.set_entry_point_chunk(chunk.ukey);
            self.compilation.async_entrypoints.push(entrypoint.ukey);
            self.next_chunk_group_index += 1;
            entrypoint.index = Some(self.next_chunk_group_index);

            if let Some(name) = entrypoint.kind.name() {
              self.named_async_entrypoints.insert(name.to_owned(), ukey);
              self
                .compilation
                .named_chunk_groups
                .insert(name.to_owned(), entrypoint.ukey);
            }

            entrypoint.connect_chunk(chunk);

            self.chunk_group_info_map.insert(entrypoint.ukey, ukey);
            self
              .compilation
              .chunk_graph
              .connect_block_and_chunk_group(block_id, entrypoint.ukey);

            self.compilation.chunk_group_by_ukey.add(entrypoint);
            ukey
          };

        let cgi = self.chunk_group_infos.expect_get(&cgi);
        entrypoint = Some(cgi.chunk_group);
        self
          .queue_delayed
          .push(QueueAction::ProcessEntryBlock(ProcessEntryBlock {
            block: block_id,
            chunk_group_info: cgi.ukey,
            chunk: chunk_ukey,
          }));
        cgi.ukey
      } else if !item_chunk_group_info.async_chunks || !item_chunk_group_info.chunk_loading {
        self.queue.push(QueueAction::ProcessBlock(ProcessBlock {
          block: block_id.into(),
          chunk_group_info: item_chunk_group_info.ukey,
          chunk: item_chunk_ukey,
        }));
        return;
      } else {
        let cgi = if let Some(chunk_name) = chunk_name
          && let Some(cgi) = self.named_chunk_groups.get(chunk_name)
        {
          let mut cgi = self.chunk_group_infos.expect_get(cgi);
          if self
            .compilation
            .chunk_group_by_ukey
            .expect_get(&cgi.chunk_group)
            .is_initial()
          {
            let error = AsyncDependenciesToInitialChunkError(
              chunk_name.to_string(),
              block.loc().map(ToOwned::to_owned),
            );
            self.compilation.push_diagnostic(Error::from(error).into());
            cgi = item_chunk_group_info;
          }

          self
            .compilation
            .chunk_graph
            .connect_block_and_chunk_group(block_id, cgi.chunk_group);
          cgi
        } else {
          let mut chunk_group = add_chunk_in_group(block.get_group_options());
          let chunk = self.compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);
          chunk
            .chunk_reasons
            .push(format!("DynamicImport({:?})", block_id));

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
            self
              .compilation
              .named_chunk_groups
              .insert(name.to_owned(), chunk_group.ukey);
          }

          chunk_group.connect_chunk(chunk);

          self
            .chunk_group_info_map
            .insert(chunk_group.ukey, info_ukey);

          self
            .compilation
            .chunk_graph
            .connect_block_and_chunk_group(block_id, chunk_group.ukey);

          self.compilation.chunk_group_by_ukey.add(chunk_group);
          info
        };
        c = Some(cgi.chunk_group);
        cgi.ukey
      };
      self.block_chunk_groups.insert(block_id, cgi);
      self.block_by_cgi.insert(cgi, block_id);
      cgi
    };

    if let Some(c) = c {
      let connect_list = self
        .queue_connect
        .entry(item_chunk_group_info_ukey)
        .or_default();
      connect_list.insert(cgi);

      // Inconsistent with webpack, webpack use minAvailableModules to avoid cycle, but calculate it is too complex
      if is_already_split {
        return;
      }

      let c = self.compilation.chunk_group_by_ukey.expect_get(&c);
      self
        .queue_delayed
        .push(QueueAction::ProcessBlock(ProcessBlock {
          block: block_id.into(),
          chunk_group_info: cgi,
          chunk: c.chunks[0],
        }));
    } else if let Some(entrypoint) = entrypoint {
      let item_chunk_group = self
        .compilation
        .chunk_group_by_ukey
        .expect_get_mut(&item_chunk_group);
      item_chunk_group.add_async_entrypoint(entrypoint);
    }
  }

  fn get_block_modules(
    &mut self,
    module: DependenciesBlockIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Vec<(ModuleIdentifier, ConnectionState, Vec<ConnectionId>)> {
    if let Some(modules) = self
      .block_modules_runtime_map
      .get::<OptionalRuntimeSpec>(&runtime.cloned().into())
      .and_then(|map| map.get(&module))
    {
      return modules.clone();
    }
    self.extract_block_modules(module.get_root_block(self.compilation), runtime);
    self
      .block_modules_runtime_map
      .get::<OptionalRuntimeSpec>(&runtime.cloned().into())
      .and_then(|map| map.get(&module))
      .unwrap_or_else(|| {
        panic!("block_modules_map.get({module:?}) must not empty after extract_block_modules")
      })
      .clone()
  }

  fn extract_block_modules(&mut self, module: ModuleIdentifier, runtime: Option<&RuntimeSpec>) {
    let module_graph = &self.compilation.get_module_graph();
    let map = self
      .block_modules_runtime_map
      .entry(runtime.cloned().into())
      .or_default();
    let block = module.into();
    map.insert(block, Vec::new());
    for b in block.get_blocks(self.compilation) {
      map.insert(b.into(), Vec::new());
    }

    let sorted_connections = module_graph
      .get_ordered_connections(&module)
      .expect("should have module")
      .into_iter()
      .map(|conn_id| {
        let conn = module_graph
          .connection_by_connection_id(conn_id)
          .expect("should have connection");

        let dep = module_graph
          .dependency_by_id(&conn.dependency_id)
          .expect("should have dependency");

        (dep, conn_id)
      });

    // keep the dependency order sorted by span
    let mut connection_map: IndexMap<
      (DependenciesBlockIdentifier, ModuleIdentifier),
      Vec<ConnectionId>,
      BuildHasherDefault<FxHasher>,
    > = IndexMap::default();

    for (dep, connection_id) in sorted_connections {
      if dep.as_module_dependency().is_none() && dep.as_context_dependency().is_none() {
        continue;
      }
      if matches!(dep.as_module_dependency().map(|d| d.weak()), Some(true)) {
        continue;
      }
      let dep_id = dep.id();
      // Dependency created but no module is available.
      // This could happen when module factorization is failed, but `options.bail` set to `false`
      let module_graph = self.compilation.get_module_graph();
      let Some(module_identifier) = module_graph.module_identifier_by_dependency_id(dep_id) else {
        continue;
      };
      let block_id =
        if let Some(block) = self.compilation.get_module_graph().get_parent_block(dep_id) {
          (*block).into()
        } else {
          module.into()
        };
      connection_map
        .entry((block_id, *module_identifier))
        .and_modify(|e| e.push(*connection_id))
        .or_insert_with(|| vec![*connection_id]);
    }

    for ((block_id, module_identifier), connections) in connection_map {
      let modules = map
        .get_mut(&block_id)
        .expect("should have modules in block_modules_runtime_map");
      let active_state = get_active_state_of_connections(
        &connections,
        runtime,
        &self.compilation.get_module_graph(),
      );
      modules.push((module_identifier, active_state, connections));
    }
  }

  fn process_connect_queue(&mut self) {
    for (chunk_group_info_ukey, targets) in self.queue_connect.drain(..) {
      let chunk_group_info = self
        .chunk_group_infos
        .expect_get_mut(&chunk_group_info_ukey);
      let chunk_group_ukey = chunk_group_info.chunk_group;

      let runtime = chunk_group_info.runtime.clone();

      let resulting_available_modules = chunk_group_info
        .calculate_resulting_available_modules(self.compilation, &self.mask_by_chunk);
      chunk_group_info.children.extend(targets.iter().cloned());

      for target in &targets {
        let target_cgi = self.chunk_group_infos.expect_get_mut(target);
        target_cgi
          .available_modules_to_be_merged
          .push(resulting_available_modules.clone());
      }

      let target_groups = targets.iter().map(|chunk_group_info_ukey| {
        let cgi = self.chunk_group_infos.expect_get(chunk_group_info_ukey);
        cgi.chunk_group
      });

      self
        .compilation
        .chunk_group_by_ukey
        .expect_get_mut(&chunk_group_ukey)
        .children
        .extend(target_groups.clone());

      for target_ukey in targets {
        let target_cgi = self.chunk_group_infos.expect_get_mut(&target_ukey);

        let target = self
          .compilation
          .chunk_group_by_ukey
          .expect_get_mut(&target_cgi.chunk_group);
        target.add_parent(chunk_group_ukey);
        self.chunk_groups_for_merging.insert(target_ukey);
        let mut updated = false;
        for r in runtime.iter() {
          updated = target_cgi.runtime.insert(r.clone());
        }
        if updated {
          self.outdated_chunk_group_info.insert(target_ukey);
        }
      }
    }
  }

  fn process_outdated_chunk_group_info(&mut self) {
    // Revisit skipped elements
    for chunk_group_info_ukey in self.outdated_chunk_group_info.drain(..) {
      let cgi = self
        .chunk_group_infos
        .expect_get_mut(&chunk_group_info_ukey);
      let chunk_group = self
        .compilation
        .chunk_group_by_ukey
        .expect_get(&cgi.chunk_group);

      let origin_queue_len = self.queue.len();
      let mut enter_modules = vec![];
      // 1. Reconsider skipped items
      for skipped in &cgi.skipped_items {
        let skipped_id = self.ordinal_by_module.get(skipped).unwrap_or_else(|| {
          panic!(
            "expected a module ordinal for identifier '{}', but none was found.",
            skipped
          )
        });
        if !cgi.min_available_modules.bit(*skipped_id) {
          enter_modules.push(*skipped);
        }
      }

      for m in &enter_modules {
        cgi.skipped_items.shift_remove(m);

        self
          .queue
          .push(QueueAction::AddAndEnterModule(AddAndEnterModule {
            module: *m,
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
            &self.compilation.get_module_graph(),
          );
          if active_state.is_false() {
            continue;
          }
          if active_state.is_true() {
            active_connections.push(i);
            let module_ordinal = self.ordinal_by_module.get(module).unwrap_or_else(|| {
              panic!(
                "expected a module ordinal for identifier '{}', but none was found.",
                module
              )
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
              chunk_group_info: chunk_group_info_ukey,
              chunk: chunk_group.chunks[0],
            })
          })
        }
        for i in active_connections {
          cgi.skipped_module_connections.shift_remove_index(i);
        }
      }

      // 3. Reconsider children chunk groups
      if !cgi.children.is_empty() {
        for child in cgi.children.iter() {
          let connect_list = self.queue_connect.entry(chunk_group_info_ukey).or_default();
          connect_list.insert(*child);
        }
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

  fn process_chunk_groups_for_combining(&mut self) {
    self.chunk_groups_for_combining.retain(|info_ukey| {
      let info = self.chunk_group_infos.expect_get(info_ukey);
      info.available_sources.iter().all(|source_ukey| {
        let source = self.chunk_group_infos.expect_get(source_ukey);
        source.min_available_modules_init
      })
    });

    let mut min_available_modules_mappings = IndexMap::<CgiUkey, BigUint>::default();
    for info_ukey in &self.chunk_groups_for_combining {
      let info = self.chunk_group_infos.expect_get(info_ukey);
      let mut available_modules = BigUint::from(0u32);
      // combine min_available_modules from all resulting_available_modules
      for source_ukey in &info.available_sources {
        let source = self.chunk_group_infos.expect_get(source_ukey);
        let resulting_available_modules =
          source.calculate_resulting_available_modules(self.compilation, &self.mask_by_chunk);
        available_modules |= resulting_available_modules;
      }
      min_available_modules_mappings.insert(*info_ukey, available_modules);
      info.invalidate_resulting_available_modules();
      self.outdated_chunk_group_info.insert(*info_ukey);
    }
    for (info_ukey, min_available_modules) in min_available_modules_mappings {
      let info = self.chunk_group_infos.expect_get_mut(&info_ukey);
      info.min_available_modules = min_available_modules;
    }

    self.chunk_groups_for_combining.clear();
  }

  fn process_chunk_groups_for_merging(&mut self) {
    for info_ukey in self.chunk_groups_for_merging.drain(..) {
      let cgi = self.chunk_group_infos.expect_get_mut(&info_ukey);

      let mut changed = false;

      if !cgi.available_modules_to_be_merged.is_empty() {
        let available_modules_to_be_merged =
          std::mem::take(&mut cgi.available_modules_to_be_merged);

        for modules_to_be_merged in available_modules_to_be_merged {
          if !cgi.min_available_modules_init {
            cgi.min_available_modules_init = true;
            cgi.min_available_modules = modules_to_be_merged;
            changed = true;
            continue;
          }

          let orig = cgi.min_available_modules.clone();
          cgi.min_available_modules &= modules_to_be_merged;
          changed = orig != cgi.min_available_modules;
        }
      }

      if changed {
        cgi.invalidate_resulting_available_modules();
        self.outdated_chunk_group_info.insert(info_ukey);
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

#[derive(Debug, Clone)]
struct ProcessBlock {
  block: DependenciesBlockIdentifier,
  chunk_group_info: CgiUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone)]
struct ProcessEntryBlock {
  block: AsyncDependenciesBlockIdentifier,
  chunk_group_info: CgiUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum DependenciesBlockIdentifier {
  Module(ModuleIdentifier),
  AsyncDependenciesBlock(AsyncDependenciesBlockIdentifier),
}

impl DependenciesBlockIdentifier {
  pub fn get_root_block<'a>(&'a self, compilation: &'a Compilation) -> ModuleIdentifier {
    match self {
      DependenciesBlockIdentifier::Module(m) => *m,
      DependenciesBlockIdentifier::AsyncDependenciesBlock(id) => *compilation
        .get_module_graph()
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
