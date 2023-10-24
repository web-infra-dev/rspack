use std::sync::Arc;

use anyhow::anyhow;
use rspack_error::Result;
use rspack_identifier::{IdentifierMap, IdentifierSet};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::remove_parent_modules::RemoveParentModulesContext;
use crate::{
  ChunkGroup, ChunkGroupInfo, ChunkGroupKind, ChunkGroupOptions, ChunkGroupOptionsKindRef,
  ChunkGroupUkey, ChunkLoading, ChunkUkey, Compilation, Logger, ModuleIdentifier, RuntimeSpec,
};

pub(super) struct CodeSplitter<'me> {
  pub(super) compilation: &'me mut Compilation,
  next_free_module_pre_order_index: u32,
  next_free_module_post_order_index: u32,
  queue: Vec<QueueItem>,
  queue_delayed: Vec<QueueItem>,
  split_point_modules: IdentifierSet,
  pub(super) remove_parent_modules_context: RemoveParentModulesContext,
  depended_modules_cache: IdentifierMap<Vec<ModuleIdentifier>>,
}

impl<'me> CodeSplitter<'me> {
  pub fn new(compilation: &'me mut Compilation) -> Self {
    CodeSplitter {
      compilation,
      next_free_module_pre_order_index: 0,
      next_free_module_post_order_index: 0,
      queue: Default::default(),
      queue_delayed: Default::default(),
      split_point_modules: Default::default(),
      remove_parent_modules_context: Default::default(),
      depended_modules_cache: Default::default(),
    }
  }

  fn prepare_input_entrypoints_and_modules(
    &mut self,
  ) -> Result<HashMap<ChunkGroupUkey, Vec<ModuleIdentifier>>> {
    let compilation = &mut self.compilation;
    let module_graph = &compilation.module_graph;

    let mut input_entrypoints_and_modules: HashMap<ChunkGroupUkey, Vec<ModuleIdentifier>> =
      HashMap::default();

    for (name, entry_data) in &compilation.entries {
      let options = &entry_data.options;
      let dependencies = [
        compilation.global_entry.dependencies.clone(),
        entry_data.dependencies.clone(),
      ]
      .concat();
      let module_identifiers = dependencies
        .iter()
        .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
        .collect::<Vec<_>>();

      let chunk = Compilation::add_named_chunk(
        name.to_string(),
        &mut compilation.chunk_by_ukey,
        &mut compilation.named_chunks,
      );
      if let Some(filename) = &entry_data.options.filename {
        chunk.filename_template = Some(filename.clone());
      }
      chunk.chunk_reasons.push(format!("Entrypoint({name})",));
      self
        .remove_parent_modules_context
        .add_root_chunk(chunk.ukey);

      compilation.chunk_graph.add_chunk(chunk.ukey);

      let mut entrypoint = ChunkGroup::new(
        ChunkGroupKind::new_entrypoint(true, options.clone()),
        HashSet::from_iter([Arc::from(
          options.runtime.clone().unwrap_or_else(|| name.to_string()),
        )]),
        ChunkGroupInfo {
          chunk_loading: !matches!(
            options
              .chunk_loading
              .as_ref()
              .unwrap_or(&compilation.options.output.chunk_loading),
            ChunkLoading::Disable
          ),
          async_chunks: options
            .async_chunks
            .unwrap_or(compilation.options.output.async_chunks),
        },
      );
      if options.runtime.is_none() {
        entrypoint.set_runtime_chunk(chunk.ukey);
      }
      entrypoint.set_entry_point_chunk(chunk.ukey);
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

        compilation
          .chunk_group_by_ukey
          .get(&ukey)
          .ok_or_else(|| anyhow::format_err!("no chunk group found"))?
      };

      for module_identifier in module_identifiers {
        compilation.chunk_graph.add_module(*module_identifier);

        input_entrypoints_and_modules
          .entry(entrypoint.ukey)
          .or_default()
          .push(*module_identifier);

        compilation.chunk_graph.connect_chunk_and_entry_module(
          chunk.ukey,
          *module_identifier,
          entrypoint.ukey,
        );
      }
    }

    for (name, entry_data) in &compilation.entries {
      let options = &entry_data.options;

      if let Some(runtime) = &options.runtime {
        let ukey = compilation
          .entrypoints
          .get(name)
          .ok_or_else(|| anyhow!("no entrypoints found"))?;

        let entry_point = compilation
          .chunk_group_by_ukey
          .get_mut(ukey)
          .ok_or_else(|| anyhow!("no chunk group found"))?;

        let chunk = match compilation.named_chunks.get(runtime) {
          Some(ukey) => compilation
            .chunk_by_ukey
            .get_mut(ukey)
            .ok_or_else(|| anyhow!("no chunk found"))?,
          None => {
            let chunk = Compilation::add_named_chunk(
              runtime.to_string(),
              &mut compilation.chunk_by_ukey,
              &mut compilation.named_chunks,
            );
            chunk.prevent_integration = true;
            chunk.chunk_reasons.push(format!("RuntimeChunk({name})",));
            compilation.chunk_graph.add_chunk(chunk.ukey);
            chunk
          }
        };

        entry_point.unshift_chunk(chunk);
        entry_point.set_runtime_chunk(chunk.ukey);
      }
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
        .get(&chunk_group)
        .ok_or_else(|| anyhow::format_err!("no chunk group found"))?;

      modules.iter().for_each(|block| {
        self
          .compilation
          .chunk_graph
          .connect_block_and_chunk_group(*block, chunk_group.ukey);
      });

      let chunk = chunk_group.get_entry_point_chunk();
      for module in modules {
        self.queue.push(QueueItem {
          action: QueueAction::AddAndEnter,
          chunk,
          chunk_group: chunk_group.ukey,
          module_identifier: module,
        });
      }
    }
    self.queue.reverse();

    let start = logger.time("process queue");
    while !self.queue.is_empty() || !self.queue_delayed.is_empty() {
      self.process_queue();
      if self.queue.is_empty() {
        self.queue = std::mem::take(&mut self.queue_delayed);
      }
    }
    logger.time_end(start);

    let start = logger.time("extend chunkGroup runtime");
    for chunk_group in self.compilation.chunk_group_by_ukey.values() {
      for chunk_ukey in chunk_group.chunks.iter() {
        self
          .compilation
          .chunk_by_ukey
          .entry(*chunk_ukey)
          .and_modify(|chunk| {
            chunk.runtime.extend(chunk_group.runtime.clone());
          });
      }
    }
    logger.time_end(start);

    let start = logger.time("remove parent modules");
    if self
      .compilation
      .options
      .optimization
      .remove_available_modules
    {
      self.remove_parent_modules();
    }
    logger.time_end(start);

    // make sure all module (weak dependency particularly) has a mgm
    for module_identifier in self.compilation.module_graph.modules().keys() {
      self.compilation.chunk_graph.add_module(*module_identifier)
    }

    Ok(())
  }

  fn process_queue(&mut self) {
    tracing::trace!("process_queue");
    while let Some(queue_item) = self.queue.pop() {
      match queue_item.action {
        QueueAction::AddAndEnter => self.add_and_enter_module(&queue_item),
        QueueAction::_Enter => self.enter_module(&queue_item),
        QueueAction::_ProcessModule => self.process_module(&queue_item),
        QueueAction::Leave => self.leave_module(&queue_item),
      }
    }
  }

  fn add_and_enter_module(&mut self, item: &QueueItem) {
    tracing::trace!("add_and_enter_module {:?}", item);
    if self
      .compilation
      .chunk_graph
      .is_module_in_chunk(&item.module_identifier, item.chunk)
    {
      return;
    }

    self
      .compilation
      .chunk_graph
      .add_module(item.module_identifier);

    self
      .compilation
      .chunk_graph
      .connect_chunk_and_module(item.chunk, item.module_identifier);
    self.enter_module(item)
  }

  fn enter_module(&mut self, item: &QueueItem) {
    tracing::trace!("enter_module {:?}", item);
    let chunk_group = self
      .compilation
      .chunk_group_by_ukey
      .get_mut(&item.chunk_group)
      .expect("chunk group not found");

    if chunk_group
      .module_pre_order_indices
      .get(&item.module_identifier)
      .is_none()
    {
      chunk_group
        .module_pre_order_indices
        .insert(item.module_identifier, chunk_group.next_pre_order_index);
      chunk_group.next_pre_order_index += 1;
    }

    {
      let module = self
        .compilation
        .module_graph
        .module_graph_module_by_identifier_mut(&item.module_identifier)
        .unwrap_or_else(|| panic!("No module found {:?}", &item.module_identifier));

      if module.pre_order_index.is_none() {
        module.pre_order_index = Some(self.next_free_module_pre_order_index);
        self.next_free_module_pre_order_index += 1;
      }
    }

    self.queue.push(QueueItem {
      action: QueueAction::Leave,
      ..item.clone()
    });
    self.process_module(item)
  }

  fn leave_module(&mut self, item: &QueueItem) {
    tracing::trace!("leave_module {:?}", item);
    let chunk_group = self
      .compilation
      .chunk_group_by_ukey
      .get_mut(&item.chunk_group)
      .expect("chunk group not found");

    if chunk_group
      .module_post_order_indices
      .get(&item.module_identifier)
      .is_none()
    {
      chunk_group
        .module_post_order_indices
        .insert(item.module_identifier, chunk_group.next_post_order_index);
      chunk_group.next_post_order_index += 1;
    }

    let module = self
      .compilation
      .module_graph
      .module_graph_module_by_identifier_mut(&item.module_identifier)
      .unwrap_or_else(|| panic!("no module found: {:?}", &item.module_identifier));

    if module.post_order_index.is_none() {
      module.post_order_index = Some(self.next_free_module_post_order_index);
      self.next_free_module_post_order_index += 1;
    }
  }

  fn process_module(&mut self, item: &QueueItem) {
    tracing::trace!("process_module {:?}", item);

    let mgm = self
      .compilation
      .module_graph
      .module_graph_module_by_identifier(&item.module_identifier)
      .unwrap_or_else(|| panic!("no module found: {:?}", &item.module_identifier));

    let queue_items = self
      .depended_modules_cache
      .entry(item.module_identifier)
      .or_insert_with(|| {
        mgm
          .depended_modules(&self.compilation.module_graph)
          .into_iter()
          .rev()
          .copied()
          .collect::<Vec<_>>()
      })
      .iter()
      .map(|module_identifier| QueueItem {
        action: QueueAction::AddAndEnter,
        chunk: item.chunk,
        chunk_group: item.chunk_group,
        module_identifier: *module_identifier,
      });
    self.queue.extend(queue_items.into_iter());

    for (module_identifier, group_options) in mgm
      .dynamic_depended_modules(&self.compilation.module_graph)
      .into_iter()
      .rev()
    {
      let item_chunk_group = self
        .compilation
        .chunk_group_by_ukey
        .get_mut(&item.chunk_group)
        .expect("chunk group not found");
      if !item_chunk_group.info.async_chunks || !item_chunk_group.info.chunk_loading {
        self.queue.push(QueueItem {
          action: QueueAction::AddAndEnter,
          chunk: item.chunk,
          chunk_group: item.chunk_group,
          module_identifier: *module_identifier,
        });
        continue;
      }

      let is_already_split_module = self.split_point_modules.contains(module_identifier);

      if is_already_split_module {
        let chunk_ukey = self
          .compilation
          .chunk_graph
          .split_point_module_identifier_to_chunk_ukey
          .get(module_identifier)
          .expect("split point module not found");
        let chunk = self
          .compilation
          .chunk_by_ukey
          .get(chunk_ukey)
          .expect("chunk not found");
        self
          .remove_parent_modules_context
          .add_chunk_relation(item.chunk, *chunk_ukey);
        item_chunk_group.children.extend(chunk.groups.clone());
        let runtime = item_chunk_group.runtime.clone();
        for chunk_group_ukey in chunk.groups.iter() {
          let chunk_group = self
            .compilation
            .chunk_group_by_ukey
            .get_mut(chunk_group_ukey)
            .expect("chunk group not found");
          chunk_group.parents.insert(item.chunk_group);
          chunk_group.runtime.extend(runtime.clone());
        }
        continue;
      } else {
        self.split_point_modules.insert(*module_identifier);
      }

      let chunk = if let Some(chunk_name) = group_options.as_ref().and_then(|x| x.name()) {
        Compilation::add_named_chunk(
          chunk_name.to_string(),
          &mut self.compilation.chunk_by_ukey,
          &mut self.compilation.named_chunks,
        )
      } else {
        Compilation::add_chunk(&mut self.compilation.chunk_by_ukey)
      };
      self
        .remove_parent_modules_context
        .add_chunk_relation(item.chunk, chunk.ukey);
      self.compilation.chunk_graph.add_chunk(chunk.ukey);

      self
        .compilation
        .chunk_graph
        .split_point_module_identifier_to_chunk_ukey
        .insert(*module_identifier, chunk.ukey);

      let mut chunk_group = if let Some(kind) = group_options.as_ref() && let &ChunkGroupOptionsKindRef::Entry(entry_options) = kind {
        if let Some(filename) = &entry_options.filename {
          chunk.filename_template = Some(filename.clone());
        }
        chunk.chunk_reasons.push(format!("AsyncEntrypoint({module_identifier})"));
        self
          .remove_parent_modules_context
          .add_root_chunk(chunk.ukey);
        let mut entrypoint = ChunkGroup::new(
          ChunkGroupKind::new_entrypoint(false, entry_options.clone()),
          RuntimeSpec::from_iter([entry_options.runtime.as_deref().expect("should have runtime for AsyncEntrypoint").into()]),
          ChunkGroupInfo {
            chunk_loading: entry_options.chunk_loading.as_ref().map(|x| !matches!(x, ChunkLoading::Disable)).unwrap_or(item_chunk_group.info.chunk_loading),
            async_chunks: entry_options.async_chunks.unwrap_or(item_chunk_group.info.async_chunks),
          },
        );
        entrypoint.set_runtime_chunk(chunk.ukey);
        entrypoint.set_entry_point_chunk(chunk.ukey);
        self.compilation.async_entrypoints.push(entrypoint.ukey);
        self.compilation.chunk_graph.add_module(*module_identifier);
        self.compilation.chunk_graph.connect_chunk_and_entry_module(
          chunk.ukey,
          *module_identifier,
          entrypoint.ukey,
        );
        entrypoint
      } else {
        chunk
          .chunk_reasons
          .push(format!("DynamicImport({module_identifier})"));
        ChunkGroup::new(
          ChunkGroupKind::Normal { options: ChunkGroupOptions::default().name_optional(group_options.as_ref().and_then(|x| x.name())) },
          item_chunk_group.runtime.clone(),
          ChunkGroupInfo {
            chunk_loading: item_chunk_group.info.chunk_loading,
            async_chunks: item_chunk_group.info.async_chunks,
          },
        )
      };

      if let Some(name) = chunk_group.kind.name() {
        self
          .compilation
          .named_chunk_groups
          .insert(name.to_owned(), chunk_group.ukey);
      }

      self
        .compilation
        .chunk_graph
        .connect_block_and_chunk_group(*module_identifier, chunk_group.ukey);

      item_chunk_group.children.insert(chunk_group.ukey);
      chunk_group.parents.insert(item_chunk_group.ukey);

      chunk_group.connect_chunk(chunk);

      let chunk_group = {
        let ukey = chunk_group.ukey;
        self.compilation.chunk_group_by_ukey.add(chunk_group);

        self
          .compilation
          .chunk_group_by_ukey
          .get(&ukey)
          .unwrap_or_else(|| panic!("chunk group not found: {ukey:?}"))
      };

      self.queue_delayed.push(QueueItem {
        action: QueueAction::AddAndEnter,
        chunk: chunk.ukey,
        chunk_group: chunk_group.ukey,
        module_identifier: *module_identifier,
      });
    }
  }
}

#[derive(Debug, Clone)]
struct QueueItem {
  action: QueueAction,
  chunk_group: ChunkGroupUkey,
  chunk: ChunkUkey,
  module_identifier: ModuleIdentifier,
}

#[derive(Debug, Clone)]
enum QueueAction {
  AddAndEnter,
  _Enter,
  _ProcessModule,
  Leave,
}

// struct chunkGroupInfoMap {}
