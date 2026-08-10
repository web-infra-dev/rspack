use std::sync::{Arc, OnceLock, RwLock};

use derive_more::Debug;
use rspack_cacheable::{cacheable, with::Skip};

use crate::{
  BoxLoader, LoaderChain, LoaderChainLocation, LoaderChainStrategy, LoaderItem,
  LoaderRunnerOptions, RunnerContext, SharedLoaderCacheService, create_loader_items,
  plan_loader_chains,
};

type LoaderChains = (Arc<[LoaderChain]>, Arc<[LoaderChainLocation]>);

/// Static loader metadata and its preassembled execution plan for one module.
///
/// The skipped fields are initialized by the module factory. When a module is
/// restored from persistent cache, they are rebuilt once on first use because
/// runtime loader metadata and the compiler-scoped cache service are not
/// serializable.
#[cacheable]
#[derive(Debug)]
pub struct Loaders {
  #[debug(skip)]
  loaders: Vec<BoxLoader>,
  options: Vec<LoaderRunnerOptions>,
  #[cacheable(with=Skip)]
  loader_items: OnceLock<Arc<[LoaderItem<RunnerContext>]>>,
  #[cacheable(with=Skip)]
  loader_chains: OnceLock<LoaderChains>,
  #[debug(skip)]
  #[cacheable(with=Skip)]
  cache_service: RwLock<Option<SharedLoaderCacheService>>,
}

impl Loaders {
  pub(crate) fn new(
    loaders: Vec<BoxLoader>,
    options: Vec<LoaderRunnerOptions>,
    cache_service: SharedLoaderCacheService,
  ) -> Self {
    let loader_items = Arc::from(create_loader_items(loaders.clone(), options.clone()));
    let (loader_chains, loader_chain_locations) =
      plan_loader_chains(&loader_items, LoaderChainStrategy::default());

    Self {
      loaders,
      options,
      loader_items: OnceLock::from(loader_items),
      loader_chains: OnceLock::from((Arc::from(loader_chains), Arc::from(loader_chain_locations))),
      cache_service: RwLock::new(Some(cache_service)),
    }
  }

  pub fn loaders(&self) -> &[BoxLoader] {
    &self.loaders
  }

  pub(crate) fn loader_items(&self) -> &Arc<[LoaderItem<RunnerContext>]> {
    self.loader_items.get_or_init(|| {
      Arc::from(create_loader_items(
        self.loaders.clone(),
        self.options.clone(),
      ))
    })
  }

  pub(crate) fn loader_chains(&self) -> &LoaderChains {
    self.loader_chains.get_or_init(|| {
      let (chains, locations) =
        plan_loader_chains(self.loader_items(), LoaderChainStrategy::default());
      (Arc::from(chains), Arc::from(locations))
    })
  }

  pub(crate) fn bind_cache_service(&self, cache_service: SharedLoaderCacheService) {
    let mut current = self
      .cache_service
      .write()
      .expect("loader cache service lock should not be poisoned");
    if current.is_none() {
      *current = Some(cache_service);
    }
  }

  pub(crate) fn cache_service(&self) -> SharedLoaderCacheService {
    self
      .cache_service
      .read()
      .expect("loader cache service lock should not be poisoned")
      .clone()
      .expect("loader cache service should be bound before running loaders")
  }
}
