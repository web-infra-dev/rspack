use std::{fmt::Write as _, hash::BuildHasherDefault, sync::Arc};

use rspack_cacheable::cacheable;
use rspack_collections::{Identifier, IdentifierHasher};
use rspack_hash::{RspackHash, RspackHasher};

use crate::{
  BoxDependency, Compilation, Dependency, DependencyId, DependencyLocation, DependencyRef,
  GroupOptions, ModuleIdentifier, RuntimeSpec,
};

pub trait DependenciesBlock {
  fn dependencies_block(&self) -> &DependenciesBlockData;

  fn dependencies_block_mut(&mut self) -> &mut DependenciesBlockData;

  fn add_block(&mut self, block: AsyncDependenciesBlockRef) {
    self.dependencies_block_mut().add_block(block);
  }

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier] {
    &self.dependencies_block().block_ids
  }

  fn get_block_refs(&self) -> &[AsyncDependenciesBlockRef] {
    &self.dependencies_block().blocks
  }

  fn add_dependency(&mut self, dependency: DependencyRef) {
    self.dependencies_block_mut().add_dependency(dependency);
  }

  fn remove_dependency_id(&mut self, dependency: DependencyId) {
    self.dependencies_block_mut().remove_dependency(dependency);
  }

  fn get_dependencies(&self) -> DependencyIds<'_> {
    DependencyIds(self.get_dependency_refs().iter())
  }

  fn get_dependency_refs(&self) -> &[DependencyRef] {
    &self.dependencies_block().dependencies
  }
}

/// Iterates dependency IDs directly from their owning references without allocating an ID list.
/// Cloning only copies the cursor so code generation can traverse the same dependencies multiple times.
#[derive(Clone)]
pub struct DependencyIds<'a>(std::slice::Iter<'a, DependencyRef>);

impl<'a> Iterator for DependencyIds<'a> {
  type Item = &'a DependencyId;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.0.next().map(|dependency| dependency.id())
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.0.size_hint()
  }
}

impl ExactSizeIterator for DependencyIds<'_> {}

/// Build-owned dependency objects and blocks. The graph indexes the same shared objects.
/// Dependency IDs are read from the objects; block IDs remain a contiguous read index.
/// Cloning copies these containers for normal-module state cache entries; the dependency
/// and block objects themselves remain shared.
#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct DependenciesBlockData {
  dependencies: Vec<DependencyRef>,
  #[cacheable(omit_bounds)]
  blocks: Vec<AsyncDependenciesBlockRef>,
  block_ids: Vec<AsyncDependenciesBlockIdentifier>,
}

impl DependenciesBlockData {
  pub fn new(dependencies: Vec<DependencyRef>, blocks: Vec<AsyncDependenciesBlockRef>) -> Self {
    Self {
      block_ids: blocks.iter().map(|block| block.identifier()).collect(),
      dependencies,
      blocks,
    }
  }

  fn add_dependency(&mut self, dependency: DependencyRef) {
    self.dependencies.push(dependency);
  }

  fn remove_dependency(&mut self, dependency: DependencyId) {
    self.dependencies.retain(|value| *value.id() != dependency);
  }

  pub(crate) fn add_block(&mut self, block: AsyncDependenciesBlockRef) {
    self.block_ids.push(block.identifier());
    self.blocks.push(block);
  }

  pub(crate) fn replace_block(&mut self, block: AsyncDependenciesBlockRef) {
    let existing = self
      .blocks
      .iter_mut()
      .find(|existing| existing.identifier() == block.identifier())
      .expect("the parent module should own the block being replaced");
    *existing = block;
  }
}

pub type AsyncDependenciesBlockIdentifierMap<V> = std::collections::HashMap<
  AsyncDependenciesBlockIdentifier,
  V,
  BuildHasherDefault<IdentifierHasher>,
>;
pub type AsyncDependenciesBlockIdentifierSet =
  std::collections::HashSet<AsyncDependenciesBlockIdentifier, BuildHasherDefault<IdentifierHasher>>;

pub fn dependencies_block_update_hash(
  deps: DependencyIds<'_>,
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
  dependencies_block: DependenciesBlockData,
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

    for dep in &dependencies {
      if let Some(resource_identifier) = dep.resource_identifier() {
        id.push_str(resource_identifier);
      }
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
      dependencies_block: DependenciesBlockData::new(
        dependencies.into_iter().map(Into::into).collect(),
        Vec::new(),
      ),
      loc,
      parent,
      request,
    }
  }

  pub fn get_dependency_mut(&mut self, idx: usize) -> Option<&mut (dyn Dependency + 'static)> {
    self
      .dependencies_block
      .dependencies
      .get_mut(idx)
      .and_then(DependencyRef::get_mut)
  }

  pub fn dependencies_mut(&mut self) -> impl Iterator<Item = &mut (dyn Dependency + 'static)> {
    self
      .dependencies_block
      .dependencies
      .iter_mut()
      .map(|dependency| {
        dependency
          .get_mut()
          .expect("parser dependencies must not be published")
      })
  }
}

/// A block and its dependency objects, shared by its owning module and graph indexes.
pub type AsyncDependenciesBlockRef = Arc<AsyncDependenciesBlock>;

impl AsyncDependenciesBlock {
  pub(crate) fn without_dependency(&self, dependency: DependencyId) -> Self {
    Self {
      id: self.id,
      group_options: self.group_options.clone(),
      dependencies_block: DependenciesBlockData::new(
        self
          .get_dependency_refs()
          .iter()
          .filter(|value| *value.id() != dependency)
          .cloned()
          .collect(),
        self.get_block_refs().to_vec(),
      ),
      loc: self.loc.clone(),
      parent: self.parent,
      request: self.request.clone(),
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

impl DependenciesBlock for AsyncDependenciesBlock {
  fn dependencies_block(&self) -> &DependenciesBlockData {
    &self.dependencies_block
  }

  fn dependencies_block_mut(&mut self) -> &mut DependenciesBlockData {
    &mut self.dependencies_block
  }

  fn add_block(&mut self, _block: AsyncDependenciesBlockRef) {
    unimplemented!("Nested block are not implemented");
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
