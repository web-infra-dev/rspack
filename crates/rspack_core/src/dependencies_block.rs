use std::{fmt::Write as _, hash::BuildHasherDefault};

use rspack_cacheable::cacheable;
use rspack_collections::{Identifier, IdentifierHasher};
use rspack_hash::{RspackHash, RspackHasher};

use crate::{
  BoxDependency, Compilation, DependencyId, DependencyLocation, DependencyRef, GroupOptions,
  ModuleIdentifier, RuntimeSpec,
};

pub trait DependenciesBlock {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier);

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier];

  fn add_dependency_id(&mut self, dependency: DependencyId);

  fn remove_dependency_id(&mut self, _dependency: DependencyId);

  fn get_dependencies(&self) -> &[DependencyId];
}

pub type AsyncDependenciesBlockIdentifierMap<V> = std::collections::HashMap<
  AsyncDependenciesBlockIdentifier,
  V,
  BuildHasherDefault<IdentifierHasher>,
>;
pub type AsyncDependenciesBlockIdentifierSet =
  std::collections::HashSet<AsyncDependenciesBlockIdentifier, BuildHasherDefault<IdentifierHasher>>;

pub fn dependencies_block_update_hash(
  deps: &[DependencyId],
  blocks: &[AsyncDependenciesBlockIdentifier],
  hasher: &mut RspackHasher,
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

impl rspack_hash::RspackHash for AsyncDependenciesBlockIdentifier {
  fn hash(&self, state: &mut RspackHasher) {
    self.0.as_str().hash(state);
  }
}

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
#[derive(Debug)]
pub struct AsyncDependenciesBlock {
  id: AsyncDependenciesBlockIdentifier,
  group_options: Option<GroupOptions>,
  // Vec<Box<T: Sized>> makes sense if T is a large type (see #3530, 1st comment).
  // #3530: https://github.com/rust-lang/rust-clippy/issues/3530
  #[allow(clippy::vec_box)]
  #[cacheable(omit_bounds)]
  blocks: Vec<Box<AsyncDependenciesBlock>>,
  block_ids: Vec<AsyncDependenciesBlockIdentifier>,
  dependency_ids: Vec<DependencyId>,
  dependencies: AsyncDependenciesBlockDependencies,
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
    let dependencies_resource_identifier_len = dependencies
      .iter()
      .filter_map(|dep| dep.resource_identifier())
      .map(str::len)
      .sum::<usize>();
    let modifier_len = modifier.map_or(0, |modifier| "|modifier=".len() + modifier.len());
    let mut id = String::with_capacity(
      parent.len() + "|dep=".len() + dependencies_resource_identifier_len + modifier_len,
    );
    id.push_str(parent.as_str());
    id.push_str("|dep=");

    let mut dependency_ids = Vec::with_capacity(dependencies.len());
    for dep in &dependencies {
      if let Some(resource_identifier) = dep.resource_identifier() {
        id.push_str(resource_identifier);
      }
      dependency_ids.push(*dep.id());
    }

    if let Some(loc) = loc.as_ref() {
      write!(id, "|loc={loc}").expect("write to String should not fail");
    }
    if let Some(modifier) = modifier {
      id.push_str("|modifier=");
      id.push_str(modifier);
    }

    Self {
      id: id.into(),
      group_options: Default::default(),
      blocks: Default::default(),
      block_ids: Default::default(),
      dependency_ids,
      dependencies: AsyncDependenciesBlockDependencies::Mutable(dependencies),
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

  pub fn take_dependencies(&mut self) -> Vec<DependencyRef> {
    match std::mem::replace(
      &mut self.dependencies,
      AsyncDependenciesBlockDependencies::Shared(Vec::new()),
    ) {
      AsyncDependenciesBlockDependencies::Mutable(dependencies) => {
        dependencies.into_iter().map(DependencyRef::from).collect()
      }
      AsyncDependenciesBlockDependencies::Shared(dependencies) => dependencies,
    }
  }

  pub fn get_dependency_mut(&mut self, idx: usize) -> Option<&mut BoxDependency> {
    match &mut self.dependencies {
      AsyncDependenciesBlockDependencies::Mutable(dependencies) => dependencies.get_mut(idx),
      AsyncDependenciesBlockDependencies::Shared(_) => None,
    }
  }

  pub fn dependencies_mut(&mut self) -> &mut [BoxDependency] {
    match &mut self.dependencies {
      AsyncDependenciesBlockDependencies::Mutable(dependencies) => dependencies,
      AsyncDependenciesBlockDependencies::Shared(_) => {
        panic!("async dependency block dependencies are frozen")
      }
    }
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
    hasher: &mut RspackHasher,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) {
    self.group_options.hash(hasher);
    if let Some(chunk_group) = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_block_chunk_group(
        &self.id,
        &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
      )
    {
      chunk_group.id(compilation).hash(hasher);
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

#[cacheable]
#[derive(Debug)]
enum AsyncDependenciesBlockDependencies {
  Mutable(Vec<BoxDependency>),
  Shared(Vec<DependencyRef>),
}

impl AsyncDependenciesBlockDependencies {
  fn freeze(&mut self) -> &[DependencyRef] {
    if let Self::Mutable(dependencies) = self {
      let dependencies = std::mem::take(dependencies)
        .into_iter()
        .map(DependencyRef::from)
        .collect();
      *self = Self::Shared(dependencies);
    }
    let Self::Shared(dependencies) = self else {
      unreachable!("mutable dependencies are frozen above")
    };
    dependencies
  }
}

/// Cache-owned structural representation for an async dependency block.
///
/// Build result blocks are consumed while they are inserted into a module graph.
/// Cache entries retain this projection and materialize fresh block containers
/// for each hit. Dependency references remain shared with the live module graph,
/// matching the module build cache and webpack's cached-module semantics;
/// interior state such as an unset lazy marker is intentionally retained across
/// hits.
#[cacheable]
#[derive(Debug)]
pub(crate) struct CachedAsyncDependenciesBlock {
  id: AsyncDependenciesBlockIdentifier,
  group_options: Option<GroupOptions>,
  #[cacheable(omit_bounds)]
  blocks: Vec<CachedAsyncDependenciesBlock>,
  block_ids: Vec<AsyncDependenciesBlockIdentifier>,
  dependency_ids: Vec<DependencyId>,
  dependencies: Vec<DependencyRef>,
  loc: Option<DependencyLocation>,
  parent: ModuleIdentifier,
  request: Option<String>,
}

impl CachedAsyncDependenciesBlock {
  pub(crate) fn from_block(block: &mut AsyncDependenciesBlock) -> Self {
    Self {
      id: block.id,
      group_options: block.group_options.clone(),
      blocks: block
        .blocks
        .iter_mut()
        .map(|block| Self::from_block(block))
        .collect(),
      block_ids: block.block_ids.clone(),
      dependency_ids: block.dependency_ids.clone(),
      dependencies: block.dependencies.freeze().to_vec(),
      loc: block.loc.clone(),
      parent: block.parent,
      request: block.request.clone(),
    }
  }

  pub(crate) fn materialize(&self) -> Box<AsyncDependenciesBlock> {
    Box::new(AsyncDependenciesBlock {
      id: self.id,
      group_options: self.group_options.clone(),
      blocks: self
        .blocks
        .iter()
        .map(|block| block.materialize())
        .collect(),
      block_ids: self.block_ids.clone(),
      dependency_ids: self.dependency_ids.clone(),
      dependencies: AsyncDependenciesBlockDependencies::Shared(self.dependencies.clone()),
      loc: self.loc.clone(),
      parent: self.parent,
      request: self.request.clone(),
    })
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
