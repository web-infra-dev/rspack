use std::{
  hash::{Hash, Hasher},
  sync::atomic::{AtomicU32, Ordering},
};

use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
};
use serde::Serialize;

use crate::{
  update_hash::{UpdateHashContext, UpdateRspackHash},
  BoxDependency, Compilation, DependencyId, GroupOptions, ModuleIdentifier,
};

pub trait DependenciesBlock {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockId);

  fn get_blocks(&self) -> &[AsyncDependenciesBlockId];

  fn add_dependency_id(&mut self, dependency: DependencyId);

  fn get_dependencies(&self) -> &[DependencyId];
}

pub static ASYNC_DEPENDENCIES_BLOCK_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct AsyncDependenciesBlockId(u32);

impl AsyncDependenciesBlockId {
  pub fn new() -> Self {
    Self(ASYNC_DEPENDENCIES_BLOCK_ID.fetch_add(1, Ordering::Relaxed))
  }

  pub fn get<'a>(&self, compilation: &'a Compilation) -> Option<&'a AsyncDependenciesBlock> {
    compilation.module_graph.block_by_id(self)
  }

  pub fn expect_get<'a>(&self, compilation: &'a Compilation) -> &'a AsyncDependenciesBlock {
    compilation
      .module_graph
      .block_by_id(self)
      .expect("should have block")
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DependencyLocation {
  start: u32,
  end: u32,
}

impl DependencyLocation {
  pub fn new(start: u32, end: u32) -> Self {
    Self { start, end }
  }

  #[inline]
  pub fn start(&self) -> u32 {
    self.start
  }

  #[inline]
  pub fn end(&self) -> u32 {
    self.end
  }
}

#[derive(Debug, Clone)]
pub struct AsyncDependenciesBlock {
  id: AsyncDependenciesBlockId,
  group_options: Option<GroupOptions>,
  blocks: Vec<AsyncDependenciesBlock>,
  block_ids: Vec<AsyncDependenciesBlockId>,
  dependency_ids: Vec<DependencyId>,
  dependencies: Vec<BoxDependency>,
  loc: Option<DependencyLocation>,
  parent: ModuleIdentifier,
}

impl AsyncDependenciesBlock {
  /// modifier should be Dependency.span in most of time
  pub fn new(parent: ModuleIdentifier, loc: Option<DependencyLocation>) -> Self {
    Self {
      id: AsyncDependenciesBlockId::new(),
      group_options: Default::default(),
      blocks: Default::default(),
      block_ids: Default::default(),
      dependency_ids: Default::default(),
      dependencies: Default::default(),
      loc,
      parent,
    }
  }
}

impl AsyncDependenciesBlock {
  pub fn id(&self) -> AsyncDependenciesBlockId {
    self.id
  }

  pub fn set_group_options(&mut self, group_options: GroupOptions) {
    self.group_options = Some(group_options)
  }

  pub fn get_group_options(&self) -> Option<&GroupOptions> {
    self.group_options.as_ref()
  }

  pub fn add_dependency(&mut self, dependency: BoxDependency) {
    self.dependency_ids.push(*dependency.id());
    self.dependencies.push(dependency);
  }

  pub fn take_dependencies(&mut self) -> Vec<BoxDependency> {
    std::mem::take(&mut self.dependencies)
  }

  pub fn add_block(&mut self, _block: AsyncDependenciesBlock) {
    unimplemented!("Nested block are not implemented");
    // self.block_ids.push(block.id);
    // self.blocks.push(block);
  }

  pub fn take_blocks(&mut self) -> Vec<AsyncDependenciesBlock> {
    std::mem::take(&mut self.blocks)
  }

  pub fn loc(&self) -> Option<&DependencyLocation> {
    self.loc.as_ref()
  }

  pub fn parent(&self) -> &ModuleIdentifier {
    &self.parent
  }
}

impl DependenciesBlock for AsyncDependenciesBlock {
  fn add_block_id(&mut self, _block: AsyncDependenciesBlockId) {
    unimplemented!("Nested block are not implemented");
    // self.block_ids.push(block);
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockId] {
    &self.block_ids
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependency_ids.push(dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependency_ids
  }
}

impl UpdateRspackHash for AsyncDependenciesBlock {
  fn update_hash<H: Hasher>(&self, state: &mut H, context: &UpdateHashContext) {
    self.group_options.hash(state);
    if let Some(chunk_group) = context
      .compilation
      .chunk_graph
      .get_block_chunk_group(&self.id, &context.compilation.chunk_group_by_ukey)
    {
      chunk_group.id(context.compilation).hash(state);
    }
    for block in &self.blocks {
      block.update_hash(state, context);
    }
  }
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(AsyncDependencyToInitialChunkError))]
#[error("It's not allowed to load an initial chunk on demand. The chunk name \"{0}\" is already used by an entrypoint.")]
pub struct AsyncDependenciesToInitialChunkError(pub String, pub Option<DependencyLocation>);
