use std::{borrow::Cow, fmt::Display, hash::Hash, sync::Arc};

use derivative::Derivative;
use rspack_collections::Identifier;
use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
};
use rspack_util::ext::DynHash;
use swc_core::common::{BytePos, SourceMap};

use crate::{
  BoxDependency, Compilation, DependencyId, GroupOptions, ModuleIdentifier, RuntimeSpec,
};

pub trait DependenciesBlock {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier);

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier];

  fn add_dependency_id(&mut self, dependency: DependencyId);

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
    let dep = mg.dependency_by_id(dep_id).expect("should have dependency");
    if let Some(dep) = dep.as_dependency_template() {
      dep.update_hash(hasher, compilation, runtime);
    }
  }
  for block_id in blocks {
    let block = mg.block_by_id_expect(block_id);
    block.update_hash(hasher, compilation, runtime);
  }
}

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct DependencyLocation {
  start: u32,
  end: u32,
  #[derivative(Debug = "ignore")]
  source: Option<Arc<SourceMap>>,
}

impl DependencyLocation {
  pub fn new(start: u32, end: u32, source: Option<Arc<SourceMap>>) -> Self {
    Self { start, end, source }
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

impl From<(u32, u32)> for DependencyLocation {
  fn from(value: (u32, u32)) -> Self {
    Self {
      start: value.0,
      end: value.1,
      source: None,
    }
  }
}

impl Display for DependencyLocation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if let Some(source) = &self.source {
      let pos = source.lookup_char_pos(BytePos(self.start + 1));
      let pos = format!("{}:{}", pos.line, pos.col.0);
      f.write_str(format!("{}-{}", pos, self.end - self.start).as_str())
    } else {
      Ok(())
    }
  }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct AsyncDependenciesBlockIdentifier(Identifier);

impl From<String> for AsyncDependenciesBlockIdentifier {
  fn from(value: String) -> Self {
    Self(value.into())
  }
}

#[derive(Debug, Clone)]
pub struct AsyncDependenciesBlock {
  id: AsyncDependenciesBlockIdentifier,
  group_options: Option<GroupOptions>,
  // Vec<Box<T: Sized>> makes sense if T is a large type (see #3530, 1st comment).
  // #3530: https://github.com/rust-lang/rust-clippy/issues/3530
  #[allow(clippy::vec_box)]
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
    let loc_str: Cow<str> = loc.clone().map_or_else(
      || "".into(),
      |loc| format!("|loc={}:{}", loc.start(), loc.end()).into(),
    );

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

  pub fn add_block(&mut self, _block: AsyncDependenciesBlock) {
    unimplemented!("Nested block are not implemented");
    // self.block_ids.push(block.id);
    // self.blocks.push(block);
  }

  pub fn take_blocks(&mut self) -> Vec<Box<AsyncDependenciesBlock>> {
    std::mem::take(&mut self.blocks)
  }

  pub fn loc(&self) -> Option<&DependencyLocation> {
    self.loc.as_ref()
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
      .chunk_graph
      .get_block_chunk_group(&self.id, &compilation.chunk_group_by_ukey)
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

  fn get_dependencies(&self) -> &[DependencyId] {
    &self.dependency_ids
  }
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic(code(AsyncDependencyToInitialChunkError))]
#[error("It's not allowed to load an initial chunk on demand. The chunk name \"{0}\" is already used by an entrypoint.")]
pub struct AsyncDependenciesToInitialChunkError(pub String, pub Option<DependencyLocation>);
