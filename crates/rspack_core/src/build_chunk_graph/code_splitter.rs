use std::sync::Arc;

use anyhow::anyhow;
use rspack_error::{internal_error, Result};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::remove_parent_modules::RemoveParentModulesContext;
use crate::{
  AsyncDependenciesBlockId, ChunkGroup, ChunkGroupInfo, ChunkGroupKind, ChunkGroupOptions,
  ChunkGroupUkey, ChunkLoading, ChunkUkey, Compilation, DependenciesBlock, Logger,
  ModuleDependency, ModuleGraphConnection, ModuleIdentifier, RuntimeSpec, IS_NEW_TREESHAKING,
};

pub(super) struct CodeSplitter<'me> {
  pub(super) compilation: &'me mut Compilation,
  next_free_module_pre_order_index: u32,
  next_free_module_post_order_index: u32,
  queue: Vec<QueueAction>,
  queue_delayed: Vec<QueueAction>,
  block_chunk_groups: HashMap<AsyncDependenciesBlockId, ChunkGroupUkey>,
  named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  named_async_entrypoints: HashMap<String, ChunkGroupUkey>,
  block_modules_map: HashMap<DependenciesBlockIdentifier, Vec<ModuleIdentifier>>,
  pub(super) remove_parent_modules_context: RemoveParentModulesContext,
}

impl<'me> CodeSplitter<'me> {
  pub fn new(compilation: &'me mut Compilation) -> Self {
    CodeSplitter {
      compilation,
      next_free_module_pre_order_index: 0,
      next_free_module_post_order_index: 0,
      queue: Default::default(),
      queue_delayed: Default::default(),
      block_chunk_groups: Default::default(),
      named_chunk_groups: Default::default(),
      named_async_entrypoints: Default::default(),
      block_modules_map: Default::default(),
      remove_parent_modules_context: Default::default(),
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
        ChunkGroupKind::new_entrypoint(true, Box::new(options.clone())),
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

    let mut runtime_chunks = HashSet::default();
    let mut runtime_error = None;
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
          Some(ukey) => {
            if !runtime_chunks.contains(ukey) {
              // TODO: add dependOn error message once we implement dependeOn
              // Did you mean to use 'dependOn: {}' instead to allow using entrypoint '{name}' within the runtime of entrypoint '{runtime}'? For this '{runtime}' must always be loaded when '{name}' is used.
              runtime_error = Some(internal_error!(
"Entrypoint '{name}' has a 'runtime' option which points to another entrypoint named '{runtime}'.
It's not valid to use other entrypoints as runtime chunk.
Or do you want to use the entrypoints '{name}' and '{runtime}' independently on the same page with a shared runtime? In this case give them both the same value for the 'runtime' option. It must be a name not already used by an entrypoint."
              ));
              let entry_chunk = entry_point.get_entry_point_chunk();
              entry_point.set_runtime_chunk(entry_chunk);
              continue;
            }
            compilation
              .chunk_by_ukey
              .get_mut(ukey)
              .ok_or_else(|| anyhow!("no chunk found"))?
          }
          None => {
            let chunk = Compilation::add_named_chunk(
              runtime.to_string(),
              &mut compilation.chunk_by_ukey,
              &mut compilation.named_chunks,
            );
            chunk.prevent_integration = true;
            chunk.chunk_reasons.push(format!("RuntimeChunk({name})",));
            compilation.chunk_graph.add_chunk(chunk.ukey);
            runtime_chunks.insert(chunk.ukey);
            chunk
          }
        };

        entry_point.unshift_chunk(chunk);
        chunk.add_group(entry_point.ukey);
        entry_point.set_runtime_chunk(chunk.ukey);
      }
    }

    if let Some(err) = runtime_error {
      compilation.push_batch_diagnostic(err.into());
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

      let chunk = chunk_group.get_entry_point_chunk();
      for module in modules {
        self
          .queue
          .push(QueueAction::AddAndEnterModule(AddAndEnterModule {
            chunk,
            chunk_group: chunk_group.ukey,
            module,
          }));
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
    self.compilation.chunk_graph.connect_chunk_and_entry_module(
      item.chunk,
      item.module,
      item.chunk_group,
    );
    self.add_and_enter_module(&AddAndEnterModule {
      module: item.module,
      chunk_group: item.chunk_group,
      chunk: item.chunk,
    })
  }

  fn add_and_enter_module(&mut self, item: &AddAndEnterModule) {
    tracing::trace!("add_and_enter_module {:?}", item);
    if self
      .compilation
      .chunk_graph
      .is_module_in_chunk(&item.module, item.chunk)
    {
      return;
    }

    self.compilation.chunk_graph.add_module(item.module);
    self
      .compilation
      .chunk_graph
      .connect_chunk_and_module(item.chunk, item.module);
    self.enter_module(&EnterModule {
      module: item.module,
      chunk_group: item.chunk_group,
      chunk: item.chunk,
    })
  }

  fn enter_module(&mut self, item: &EnterModule) {
    tracing::trace!("enter_module {:?}", item);
    let chunk_group = self
      .compilation
      .chunk_group_by_ukey
      .get_mut(&item.chunk_group)
      .expect("chunk group not found");

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
      let module = self
        .compilation
        .module_graph
        .module_graph_module_by_identifier_mut(&item.module)
        .unwrap_or_else(|| panic!("No module found {:?}", &item.module));

      if module.pre_order_index.is_none() {
        module.pre_order_index = Some(self.next_free_module_pre_order_index);
        self.next_free_module_pre_order_index += 1;
      }
    }

    self.queue.push(QueueAction::LeaveModule(LeaveModule {
      module: item.module,
      chunk_group: item.chunk_group,
    }));
    self.process_block(&ProcessBlock {
      block: item.module.into(),
      chunk_group: item.chunk_group,
      chunk: item.chunk,
    })
  }

  fn leave_module(&mut self, item: &LeaveModule) {
    tracing::trace!("leave_module {:?}", item);
    let chunk_group = self
      .compilation
      .chunk_group_by_ukey
      .get_mut(&item.chunk_group)
      .expect("chunk group not found");

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

    let module = self
      .compilation
      .module_graph
      .module_graph_module_by_identifier_mut(&item.module)
      .unwrap_or_else(|| panic!("no module found: {:?}", &item.module));

    if module.post_order_index.is_none() {
      module.post_order_index = Some(self.next_free_module_post_order_index);
      self.next_free_module_post_order_index += 1;
    }
  }

  fn process_entry_block(&mut self, item: &ProcessEntryBlock) {
    tracing::trace!("process_entry_block {:?}", item);
    let modules = self.get_block_modules(item.block.into());
    for module in modules {
      self.queue.push(QueueAction::AddAndEnterEntryModule(
        AddAndEnterEntryModule {
          chunk: item.chunk,
          chunk_group: item.chunk_group,
          module,
        },
      ));
    }
    let blocks = self
      .compilation
      .module_graph
      .block_by_id(&item.block)
      .expect("should have block")
      .get_blocks()
      .to_vec();
    for block in blocks {
      self.iterator_block(block, item.chunk_group, item.chunk);
    }
  }

  fn process_block(&mut self, item: &ProcessBlock) {
    tracing::trace!("process_block {:?}", item);
    let modules = self.get_block_modules(item.block);
    for module in modules.into_iter().rev() {
      self
        .queue
        .push(QueueAction::AddAndEnterModule(AddAndEnterModule {
          chunk: item.chunk,
          chunk_group: item.chunk_group,
          module,
        }));
    }
    let blocks = match &item.block {
      DependenciesBlockIdentifier::Module(m) => self
        .compilation
        .module_graph
        .module_by_identifier(m)
        .expect("should have module")
        .get_blocks()
        .to_vec(),
      DependenciesBlockIdentifier::AsyncDependenciesBlock(a) => self
        .compilation
        .module_graph
        .block_by_id(a)
        .expect("should have block")
        .get_blocks()
        .to_vec(),
    };
    for block in blocks.into_iter().rev() {
      self.iterator_block(block, item.chunk_group, item.chunk);
    }
  }

  fn iterator_block(
    &mut self,
    block_id: AsyncDependenciesBlockId,
    item_chunk_group_ukey: ChunkGroupUkey,
    item_chunk_ukey: ChunkUkey,
  ) {
    let cgi = self.block_chunk_groups.get(&block_id);
    let mut entrypoint = None;
    let mut c = None;
    let block = self
      .compilation
      .module_graph
      .block_by_id(&block_id)
      .expect("should have block");
    let item_chunk_group = self
      .compilation
      .chunk_group_by_ukey
      .get_mut(&item_chunk_group_ukey)
      .expect("chunk group not found");
    let chunk = if let Some(chunk_name) = block.get_group_options().and_then(|x| x.name()) {
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
      .add_chunk_relation(item_chunk_ukey, chunk.ukey);
    self.compilation.chunk_graph.add_chunk(chunk.ukey);

    let entry_options = block.get_group_options().and_then(|o| o.entry_options());
    if let Some(cgi) = cgi {
      if entry_options.is_some() {
        entrypoint = Some(*cgi);
      } else {
        c = Some(*cgi);
      }
    } else {
      let chunk_name = block.get_group_options().and_then(|o| o.name());
      if let Some(entry_options) = entry_options {
        let chunk_group =
          if let Some(cgi) = chunk_name.and_then(|name| self.named_async_entrypoints.get(name)) {
            self
              .compilation
              .chunk_graph
              .connect_block_and_chunk_group(block_id, *cgi);
            *cgi
          } else {
            if let Some(filename) = &entry_options.filename {
              chunk.filename_template = Some(filename.clone());
            }
            chunk
              .chunk_reasons
              .push(format!("AsyncEntrypoint({:?})", block_id));
            self
              .remove_parent_modules_context
              .add_root_chunk(chunk.ukey);
            let mut entrypoint = ChunkGroup::new(
              ChunkGroupKind::new_entrypoint(false, Box::new(entry_options.clone())),
              RuntimeSpec::from_iter([entry_options
                .runtime
                .as_deref()
                .expect("should have runtime for AsyncEntrypoint")
                .into()]),
              ChunkGroupInfo {
                chunk_loading: entry_options
                  .chunk_loading
                  .as_ref()
                  .map(|x| !matches!(x, ChunkLoading::Disable))
                  .unwrap_or(item_chunk_group.info.chunk_loading),
                async_chunks: entry_options
                  .async_chunks
                  .unwrap_or(item_chunk_group.info.async_chunks),
              },
            );
            entrypoint.set_runtime_chunk(chunk.ukey);
            entrypoint.set_entry_point_chunk(chunk.ukey);
            self.compilation.async_entrypoints.push(entrypoint.ukey);

            if let Some(name) = entrypoint.kind.name() {
              self
                .named_async_entrypoints
                .insert(name.to_owned(), entrypoint.ukey);
              self
                .compilation
                .named_chunk_groups
                .insert(name.to_owned(), entrypoint.ukey);
            }

            item_chunk_group.children.insert(entrypoint.ukey);
            entrypoint.parents.insert(item_chunk_group.ukey);

            entrypoint.connect_chunk(chunk);

            self
              .compilation
              .chunk_graph
              .connect_block_and_chunk_group(block_id, entrypoint.ukey);

            let ukey = entrypoint.ukey;
            self.compilation.chunk_group_by_ukey.add(entrypoint);
            ukey
          };
        entrypoint = Some(chunk_group);
        self
          .queue_delayed
          .push(QueueAction::ProcessEntryBlock(ProcessEntryBlock {
            block: block_id,
            chunk_group,
            chunk: chunk.ukey,
          }));
      } else if !item_chunk_group.info.async_chunks || !item_chunk_group.info.chunk_loading {
        self.queue.push(QueueAction::ProcessBlock(ProcessBlock {
          block: block_id.into(),
          chunk_group: item_chunk_group_ukey,
          chunk: item_chunk_ukey,
        }));
      } else {
        let chunk_group =
          if let Some(cgi) = chunk_name.and_then(|name| self.named_chunk_groups.get(name)) {
            // TODO: AsyncDependencyToInitialChunkError
            *cgi
          } else {
            chunk
              .chunk_reasons
              .push(format!("DynamicImport({:?})", block_id));
            let mut chunk_group = ChunkGroup::new(
              ChunkGroupKind::Normal {
                options: ChunkGroupOptions::default()
                  .name_optional(block.get_group_options().and_then(|x| x.name())),
              },
              item_chunk_group.runtime.clone(),
              ChunkGroupInfo {
                chunk_loading: item_chunk_group.info.chunk_loading,
                async_chunks: item_chunk_group.info.async_chunks,
              },
            );

            if let Some(name) = chunk_group.kind.name() {
              self
                .named_async_entrypoints
                .insert(name.to_owned(), chunk_group.ukey);
              self
                .compilation
                .named_chunk_groups
                .insert(name.to_owned(), chunk_group.ukey);
            }

            item_chunk_group.children.insert(chunk_group.ukey);
            chunk_group.parents.insert(item_chunk_group.ukey);

            chunk_group.connect_chunk(chunk);

            self
              .compilation
              .chunk_graph
              .connect_block_and_chunk_group(block_id, chunk_group.ukey);

            let ukey = chunk_group.ukey;
            self.compilation.chunk_group_by_ukey.add(chunk_group);
            ukey
          };
        c = Some(chunk_group);
      }
    }
    if let Some(c) = c {
      self
        .queue_delayed
        .push(QueueAction::ProcessBlock(ProcessBlock {
          block: block_id.into(),
          chunk_group: c,
          chunk: chunk.ukey,
        }));
    } else if let Some(_entrypoint) = entrypoint {
      // TODO: chunk_group.add_async_entrypoint(entrypoint)
    }
  }

  fn get_block_modules(&mut self, module: DependenciesBlockIdentifier) -> Vec<ModuleIdentifier> {
    if let Some(modules) = self.block_modules_map.get(&module) {
      return modules.clone();
    }
    self.extract_block_modules(*module.as_module().expect(
      "block_modules_map must not empty when calling get_block_modules(AsyncDependenciesBlock)",
    ));
    self
      .block_modules_map
      .get(&module)
      .expect("block_modules_map.get(module) must not empty after extract_block_modules")
      .clone()
  }

  fn extract_block_modules(&mut self, module: ModuleIdentifier) {
    self.block_modules_map.insert(module.into(), Vec::new());

    let mgm = self
      .compilation
      .module_graph
      .module_graph_module_by_identifier(&module)
      .unwrap_or_else(|| panic!("no module found: {:?}", &module));
    let dependencies: Vec<&dyn ModuleDependency> =
      if IS_NEW_TREESHAKING.load(std::sync::atomic::Ordering::Relaxed) {
        mgm
          .outgoing_connections_unordered(&self.compilation.module_graph)
          .expect("should have outgoing connections")
          .filter_map(|con: &ModuleGraphConnection| {
            // TODO: runtime opt
            let active_state = con.get_active_state(&self.compilation.module_graph, None);
            match active_state {
              crate::ConnectionState::Bool(false) => None,
              _ => Some(con.dependency_id),
            }
          })
          .filter_map(|dep_id| self.compilation.module_graph.dependency_by_id(&dep_id))
          .filter_map(|dep| dep.as_module_dependency())
          .collect()
      } else {
        mgm
          .all_dependencies
          .iter()
          .filter_map(|dep_id| self.compilation.module_graph.dependency_by_id(&dep_id))
          .filter_map(|dep| dep.as_module_dependency())
          .collect()
      };
    for dep in dependencies {
      if dep.weak() {
        continue;
      }
      let dep_id = dep.id();
      let block_id = if let Some(block) = self.compilation.module_graph.get_parent_block(dep_id) {
        (*block).into()
      } else {
        module.into()
      };
      let modules = self.block_modules_map.entry(block_id).or_default();
      modules.push(
        *self
          .compilation
          .module_graph
          .module_identifier_by_dependency_id(dep_id)
          .expect("should have module_identifier"),
      );
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
  chunk_group: ChunkGroupUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone)]
struct AddAndEnterModule {
  module: ModuleIdentifier,
  chunk_group: ChunkGroupUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone)]
struct EnterModule {
  module: ModuleIdentifier,
  chunk_group: ChunkGroupUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone)]
struct ProcessBlock {
  block: DependenciesBlockIdentifier,
  chunk_group: ChunkGroupUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone)]
struct ProcessEntryBlock {
  block: AsyncDependenciesBlockId,
  chunk_group: ChunkGroupUkey,
  chunk: ChunkUkey,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum DependenciesBlockIdentifier {
  Module(ModuleIdentifier),
  AsyncDependenciesBlock(AsyncDependenciesBlockId),
}

impl DependenciesBlockIdentifier {
  pub fn as_module(&self) -> Option<&ModuleIdentifier> {
    match self {
      DependenciesBlockIdentifier::Module(m) => Some(m),
      DependenciesBlockIdentifier::AsyncDependenciesBlock(_) => None,
    }
  }
}

impl From<ModuleIdentifier> for DependenciesBlockIdentifier {
  fn from(value: ModuleIdentifier) -> Self {
    Self::Module(value)
  }
}

impl From<AsyncDependenciesBlockId> for DependenciesBlockIdentifier {
  fn from(value: AsyncDependenciesBlockId) -> Self {
    Self::AsyncDependenciesBlock(value)
  }
}

#[derive(Debug, Clone)]
struct LeaveModule {
  module: ModuleIdentifier,
  chunk_group: ChunkGroupUkey,
}
