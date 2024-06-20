use std::{
  fmt::Display,
  hash::{Hash, Hasher},
  sync::Arc,
};

use derivative::Derivative;
use rspack_error::{
  miette::{self, Diagnostic},
  thiserror::{self, Error},
};
use rspack_identifier::Identifier;
use swc_core::common::{source_map::Pos, BytePos, SourceMap};

use crate::{
  update_hash::{UpdateHashContext, UpdateRspackHash},
  BoxDependency, DependencyId, DependencyTemplate, GroupOptions, ModuleIdentifier,
};

pub trait DependenciesBlock {
  fn add_block_id(&mut self, block: AsyncDependenciesBlockIdentifier);

  fn get_blocks(&self) -> &[AsyncDependenciesBlockIdentifier];

  fn add_dependency_id(&mut self, dependency: DependencyId);

  fn get_dependencies(&self) -> &[DependencyId];

  fn get_presentational_dependencies_for_block(&self) -> Option<&[Box<dyn DependencyTemplate>]> {
    None
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
      let pos = source.lookup_char_pos(BytePos::from_u32(self.start + 1));
      let pos = format!("{}:{}", pos.line, pos.col.to_usize());
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
  blocks: Vec<AsyncDependenciesBlock>,
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
    let loc_str = loc.clone().map_or_else(
      || "".to_string(),
      |loc| format!("|loc={}:{}", loc.start(), loc.end()),
    );

    let modifier_str = modifier.map_or_else(
      || "".to_string(),
      |modifier| format!("|modifier={modifier}"),
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

  pub fn take_blocks(&mut self) -> Vec<AsyncDependenciesBlock> {
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
