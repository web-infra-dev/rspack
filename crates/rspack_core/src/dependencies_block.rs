use std::sync::atomic::{AtomicU32, Ordering};

use once_cell::sync::Lazy;
use rspack_identifier::{Identifiable, Identifier};

use crate::{BoxDependency, DependencyId, GroupOptions};

pub trait DependenciesBlock {
  fn add_block(&mut self, block: AsyncDependenciesBlockId);

  fn get_blocks(&self) -> &[AsyncDependenciesBlockId];

  fn add_dependency(&mut self, dependency: DependencyId);

  fn get_dependencies(&self) -> &[DependencyId];
}

static ASYNC_DEPENDENCIES_BLOCK_ID: Lazy<AtomicU32> = Lazy::new(|| AtomicU32::new(0));

fn get_async_dependencies_id() -> AsyncDependenciesBlockId {
  AsyncDependenciesBlockId::from(
    ASYNC_DEPENDENCIES_BLOCK_ID
      .fetch_add(1, Ordering::Relaxed)
      .to_string(),
  )
}

pub type AsyncDependenciesBlockId = Identifier;

#[derive(Debug, Clone)]
pub struct AsyncDependenciesBlock {
  id: AsyncDependenciesBlockId,
  group_options: Option<GroupOptions>,
  blocks: Vec<AsyncDependenciesBlock>,
  block_ids: Vec<AsyncDependenciesBlockId>,
  dependency_ids: Vec<DependencyId>,
  dependencies: Vec<BoxDependency>,
}

impl Default for AsyncDependenciesBlock {
  fn default() -> Self {
    Self {
      id: get_async_dependencies_id(),
      group_options: Default::default(),
      blocks: Default::default(),
      block_ids: Default::default(),
      dependency_ids: Default::default(),
      dependencies: Default::default(),
    }
  }
}

impl AsyncDependenciesBlock {
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

  pub fn add_block(&mut self, block: AsyncDependenciesBlock) {
    self.block_ids.push(block.id);
    self.blocks.push(block);
  }

  pub fn take_blocks(&mut self) -> Vec<AsyncDependenciesBlock> {
    std::mem::take(&mut self.blocks)
  }
}

impl Identifiable for AsyncDependenciesBlock {
  fn identifier(&self) -> Identifier {
    self.id
  }
}

impl DependenciesBlock for AsyncDependenciesBlock {
  fn add_block(&mut self, block: AsyncDependenciesBlockId) {
    self.block_ids.push(block)
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockId] {
    &self.block_ids
  }

  fn add_dependency(&mut self, dependency: DependencyId) {
    self.dependency_ids.push(dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependency_ids
  }
}
