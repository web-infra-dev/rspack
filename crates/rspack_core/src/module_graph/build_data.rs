use std::collections::VecDeque;

use rspack_cacheable::cacheable;

use crate::{AsyncDependenciesBlock, BoxModule, DependencyRef, OptimizationBailoutItem};

/// Module-local build output retained after its indices are installed in the graph.
/// Dependency references keep their shared identity, including mutable dependency state.
#[cacheable]
#[derive(Debug, Default, Clone)]
pub(crate) struct ModuleBuildData {
  pub dependencies: Vec<DependencyRef>,
  #[allow(clippy::vec_box)]
  pub blocks: Vec<Box<AsyncDependenciesBlock>>,
  pub optimization_bailouts: Vec<OptimizationBailoutItem>,
}

impl ModuleBuildData {
  /// Both fresh builds and restored entries use a flat block table. Blocks keep
  /// their dependency objects; the graph only installs indices into this data.
  pub(crate) fn normalize_blocks(&mut self) {
    let mut queue = VecDeque::from(std::mem::take(&mut self.blocks));
    while let Some(mut block) = queue.pop_front() {
      queue.extend(block.take_blocks());
      self.blocks.push(block);
    }
  }
}

#[derive(Debug)]
pub(crate) struct ModuleRecord {
  pub module: BoxModule,
  pub build_data: ModuleBuildData,
}
