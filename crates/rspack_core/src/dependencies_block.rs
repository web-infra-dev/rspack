use std::hash::{Hash, Hasher};

use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
};
use serde::Serialize;
use ustr::Ustr;

use crate::{
  update_hash::{UpdateHashContext, UpdateRspackHash},
  BoxDependency, Compilation, DependencyId, GroupOptions, ModuleIdentifier,
};

pub trait DependenciesBlock {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier);

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier];

  fn add_dependency_id(&mut self, dependency: DependencyId);

  fn get_dependencies(&self) -> &[DependencyId];
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
pub struct AsyncDependenciesBlockIdentifier {
  pub from: ModuleIdentifier,
  modifier: Ustr,
}

impl AsyncDependenciesBlockIdentifier {
  pub fn new(from: ModuleIdentifier, modifier: Ustr) -> Self {
    Self { from, modifier }
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

#[derive(Debug, Clone, Copy)]
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
  id: AsyncDependenciesBlockIdentifier,
  group_options: Option<GroupOptions>,
  blocks: Vec<AsyncDependenciesBlock>,
  block_ids: Vec<AsyncDependenciesBlockIdentifier>,
  dependency_ids: Vec<DependencyId>,
  dependencies: Vec<BoxDependency>,
  loc: Option<DependencyLocation>,
}

impl AsyncDependenciesBlock {
  /// modifier should be Dependency.span in most of time
  pub fn new(
    from: ModuleIdentifier,
    modifier: impl AsRef<str>,
    loc: Option<DependencyLocation>,
  ) -> Self {
    Self {
      id: AsyncDependenciesBlockIdentifier::new(from, modifier.as_ref().into()),
      group_options: Default::default(),
      blocks: Default::default(),
      block_ids: Default::default(),
      dependency_ids: Default::default(),
      dependencies: Default::default(),
      loc,
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
