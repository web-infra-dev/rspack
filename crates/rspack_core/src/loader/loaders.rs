use std::sync::OnceLock;

use derive_more::Debug;
use rspack_cacheable::{cacheable, with::Skip};

use crate::{
  BoxLoader, LoaderChain, LoaderItem, LoaderRunnerPlan, RunnerContext, create_loader_items,
  plan_loader_chains,
};

/// Static loader metadata and its preassembled execution plan for one module.
#[cacheable]
#[derive(Debug)]
pub struct Loaders {
  #[debug(skip)]
  loaders: Vec<BoxLoader>,
  #[cacheable(with=Skip)]
  loader_items: OnceLock<Vec<LoaderItem<RunnerContext>>>,
  #[cacheable(with=Skip)]
  loader_chains: OnceLock<Vec<LoaderChain>>,
}

impl Loaders {
  pub(crate) fn new(loaders: Vec<BoxLoader>) -> Self {
    let loader_items = create_loader_items(loaders.clone());
    let loader_chains = plan_loader_chains(&loader_items);

    Self {
      loaders,
      loader_items: OnceLock::from(loader_items),
      loader_chains: OnceLock::from(loader_chains),
    }
  }

  pub fn loaders(&self) -> &[BoxLoader] {
    &self.loaders
  }

  fn loader_items(&self) -> &[LoaderItem<RunnerContext>] {
    self
      .loader_items
      .get_or_init(|| create_loader_items(self.loaders.clone()))
  }

  fn loader_chains(&self) -> &[LoaderChain] {
    self
      .loader_chains
      .get_or_init(|| plan_loader_chains(self.loader_items()))
  }
}

impl LoaderRunnerPlan<RunnerContext> for Loaders {
  fn loader_items(&self) -> &[LoaderItem<RunnerContext>] {
    self.loader_items()
  }

  fn loader_chains(&self) -> &[LoaderChain] {
    self.loader_chains()
  }
}
