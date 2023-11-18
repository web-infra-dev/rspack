use serde::Serialize;
use ustr::Ustr;

use crate::{BoxDependency, Compilation, DependencyId, GroupOptions, ModuleIdentifier};

pub trait DependenciesBlock {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier);

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier];

  fn add_dependency_id(&mut self, dependency: DependencyId);

  fn get_dependencies(&self) -> &[DependencyId];
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct AsyncDependenciesBlockIdentifier {
  from: ModuleIdentifier,
  modifier: Ustr,
}

impl AsyncDependenciesBlockIdentifier {
  pub fn new(from: ModuleIdentifier, modifier: Ustr) -> Self {
    Self { from, modifier }
  }
}

#[derive(Debug, Clone)]
pub struct AsyncDependenciesBlock {
  id: AsyncDependenciesBlockIdentifier,
  group_options: Option<GroupOptions>,
  blocks: Vec<AsyncDependenciesBlock>,
  block_ids: Vec<AsyncDependenciesBlockIdentifier>,
  dependency_ids: Vec<DependencyId>,
  dependencies: Vec<BoxDependency>,
}

impl AsyncDependenciesBlock {
  /// modifier should be Dependency.span in most of time
  pub fn new(from: ModuleIdentifier, modifier: impl AsRef<str>) -> Self {
    Self {
      id: AsyncDependenciesBlockIdentifier::new(from, modifier.as_ref().into()),
      group_options: Default::default(),
      blocks: Default::default(),
      block_ids: Default::default(),
      dependency_ids: Default::default(),
      dependencies: Default::default(),
    }
  }
}

impl AsyncDependenciesBlock {
  // represent an unique AsyncDependenciesBlock, we use this as the block_promise_key at codegen
  // why not Id(u32)? Id(u32) is unstable since module is built concurrently
  // why not ChunkGroup.id? ChunkGroup.id = ChunkGroup.chunks.map(c => c.id).join("+"), probably will break incremental build, same reason we create __webpack_require__.el
  pub fn identifier(&self) -> AsyncDependenciesBlockIdentifier {
    self.id
  }

  pub fn block_promise_key(&self, compilation: &Compilation) -> String {
    let module_id = compilation
      .chunk_graph
      .get_module_id(self.id.from)
      .as_ref()
      .expect("should have module_id");
    let key = format!("{}@{}", module_id, self.id.modifier);
    serde_json::to_string(&key).expect("AsyncDependenciesBlock.id should be able to json to_string")
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
}

impl DependenciesBlock for AsyncDependenciesBlock {
  fn add_block_id(&mut self, _block: AsyncDependenciesBlockIdentifier) {
    unimplemented!("Nested block are not implemented");
    // self.block_ids.push(block);
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.block_ids
  }

  fn add_dependency_id(&mut self, dependency: DependencyId) {
    self.dependency_ids.push(dependency)
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependency_ids
  }
}
