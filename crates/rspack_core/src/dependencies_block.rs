use std::{borrow::Cow, hash::Hash};

use rspack_cacheable::cacheable;
use rspack_collections::Identifier;
use rspack_util::ext::DynHash;

use crate::{
  BoxDependency, Compilation, DependencyId, DependencyLocation, GroupOptions, ModuleIdentifier,
  RuntimeSpec,
};

pub trait DependenciesBlock {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier);

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier];

  fn add_dependency_id(&mut self, dependency: DependencyId);

  fn remove_dependency_id(&mut self, _dependency: DependencyId);

  fn get_dependencies(&self) -> &[DependencyId];
}

pub fn dependencies_block_update_hash(
  deps: &[DependencyId],
  blocks: &[AsyncDependenciesBlockIdentifier],
  hasher: &mut dyn std::hash::Hasher,
  compilation: &Compilation,
  runtime: Option<&RuntimeSpec>,
) {
  let mg = compilation.get_module_graph();
  for dep_id in deps {
    let dep = mg.dependency_by_id(dep_id);
    if let Some(dep) = dep.as_dependency_code_generation() {
      dep.update_hash(hasher, compilation, runtime);
    }
  }
  for block_id in blocks {
    let block = mg.block_by_id_expect(block_id);
    block.update_hash(hasher, compilation, runtime);
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct AsyncDependenciesBlockIdentifier(Identifier);

impl From<String> for AsyncDependenciesBlockIdentifier {
  fn from(value: String) -> Self {
    Self(value.into())
  }
}

impl From<Identifier> for AsyncDependenciesBlockIdentifier {
  fn from(value: Identifier) -> Self {
    Self(value)
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct AsyncDependenciesBlock {
  id: AsyncDependenciesBlockIdentifier,
  group_options: Option<GroupOptions>,
  // Vec<Box<T: Sized>> makes sense if T is a large type (see #3530, 1st comment).
  // #3530: https://github.com/rust-lang/rust-clippy/issues/3530
  #[allow(clippy::vec_box)]
  #[cacheable(with=::rspack_cacheable::with::AsCacheable, omit_bounds)]
  blocks: Vec<Box<AsyncDependenciesBlock>>,
  block_ids: Vec<AsyncDependenciesBlockIdentifier>,
  dependency_ids: Vec<DependencyId>,
  dependencies: Vec<BoxDependency>,
  loc: Option<DependencyLocation>,
  parent: ModuleIdentifier,
  request: Option<String>,
}

impl AsyncDependenciesBlock {
  /// modifier should be Dependency.span in most of time
  pub fn new(
    parent: ModuleIdentifier,
    loc: Option<DependencyLocation>,
    modifier: Option<&str>,
    dependencies: Vec<BoxDependency>,
    request: Option<String>,
  ) -> Self {
    let loc_str: Cow<str> = loc
      .clone()
      .map_or_else(|| "".into(), |loc| format!("|loc={loc}").into());

    let modifier_str: Cow<str> = modifier.map_or_else(
      || "".into(),
      |modifier| format!("|modifier={modifier}").into(),
    );

    Self {
      id: format!(
        "{parent}|dep={}{}{}",
        dependencies.iter().fold(String::default(), |mut s, dep| {
          s += dep.resource_identifier().unwrap_or_default();
          s
        }),
        loc_str,
        modifier_str
      )
      .into(),
      group_options: Default::default(),
      blocks: Default::default(),
      block_ids: Default::default(),
      dependency_ids: dependencies.iter().map(|dep| *dep.id()).collect(),
      dependencies,
      loc,
      parent,
      request,
    }
  }

  pub fn identifier(&self) -> AsyncDependenciesBlockIdentifier {
    self.id
  }

  pub fn set_group_options(&mut self, group_options: GroupOptions) {
    self.group_options = Some(group_options)
  }

  pub fn get_group_options(&self) -> Option<&GroupOptions> {
    self.group_options.as_ref()
  }

  pub fn take_dependencies(&mut self) -> Vec<BoxDependency> {
    std::mem::take(&mut self.dependencies)
  }

  pub fn get_dependency_mut(&mut self, idx: usize) -> Option<&mut BoxDependency> {
    self.dependencies.get_mut(idx)
  }

  pub fn add_block(&mut self, _block: AsyncDependenciesBlock) {
    unimplemented!("Nested block are not implemented");
    // self.block_ids.push(block.id);
    // self.blocks.push(block);
  }

  pub fn take_blocks(&mut self) -> Vec<Box<AsyncDependenciesBlock>> {
    std::mem::take(&mut self.blocks)
  }

  pub fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  pub fn parent(&self) -> &ModuleIdentifier {
    &self.parent
  }

  pub fn request(&self) -> &Option<String> {
    &self.request
  }

  pub fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) {
    self.group_options.dyn_hash(hasher);
    if let Some(chunk_group) = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_block_chunk_group(
        &self.id,
        &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      )
    {
      chunk_group.id(compilation).dyn_hash(hasher);
    }
    dependencies_block_update_hash(
      self.get_dependencies(),
      self.get_blocks(),
      hasher,
      compilation,
      runtime,
    );
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

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependency_ids.retain(|dep| dep != &dependency);
  }

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependency_ids
  }
}

#[derive(Debug)]
pub struct AsyncDependenciesToInitialChunkError(pub String, pub Option<DependencyLocation>);

impl From<AsyncDependenciesToInitialChunkError> for rspack_error::Error {
  fn from(value: AsyncDependenciesToInitialChunkError) -> rspack_error::Error {
    let mut error = rspack_error::error!(
      "It's not allowed to load an initial chunk on demand. The chunk name \"{}\" is already used by an entrypoint.",
      value.0
    );
    error.code = Some("AsyncDependencyToInitialChunkError".into());
    error
  }
}
