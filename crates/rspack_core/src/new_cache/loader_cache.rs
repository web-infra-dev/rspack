use std::{
  ops::Deref,
  sync::{Arc, LazyLock, OnceLock, Weak},
  time::Duration,
};

use arc_swap::ArcSwapWeak;
use dashmap::mapref::entry::Entry;
use parking_lot::Mutex;
use rspack_paths::{InternedPath, InternedPathDashMap};

use super::{
  Cache, CompilerCache, FileCacheStrategy, IdleFileCache, MemoryCache,
  idle_file_cache::DEFAULT_IDLE_TIMEOUT, memory_cache,
};
use crate::{CacheOptions, CompilerOptions, StorageOptions, cache::CacheCodec};

static MEMORY_LOADER_CACHE: LazyLock<ArcSwapWeak<LoaderCache>> = LazyLock::new(Default::default);
static PERSISTENT_LOADER_CACHES: LazyLock<InternedPathDashMap<PersistentLoaderCacheEntry>> =
  LazyLock::new(Default::default);

struct PersistentLoaderCacheEntry {
  cache: Weak<LoaderCache>,
  closed: Option<Arc<OnceLock<()>>>,
}

/// Loader cache shared by compilers using the same storage.
#[derive(Debug)]
pub struct LoaderCache {
  cache: Cache,
  directory: Option<InternedPath>,
  active_builds: Mutex<usize>,
}

impl Deref for LoaderCache {
  type Target = Cache;

  fn deref(&self) -> &Self::Target {
    &self.cache
  }
}

impl LoaderCache {
  fn new(cache: Cache, directory: Option<InternedPath>) -> Arc<Self> {
    Arc::new(Self {
      cache,
      directory,
      active_builds: Mutex::default(),
    })
  }

  pub(crate) fn begin_build(self: Arc<Self>) -> LoaderCacheGuard {
    let mut active_builds = self.active_builds.lock();
    if *active_builds == 0 {
      self.end_idle();
    }
    *active_builds += 1;
    drop(active_builds);
    LoaderCacheGuard(self)
  }
}

impl Drop for LoaderCache {
  fn drop(&mut self) {
    self.cache.shutdown_blocking();
    if let Some(directory) = &self.directory
      && let Entry::Occupied(entry) = PERSISTENT_LOADER_CACHES.entry(directory.clone())
    {
      let closed = entry.remove().closed;
      if let Some(closed) = closed {
        let _ = closed.set(());
      }
    }
  }
}

/// Keeps the shared loader cache active until this module-building scope finishes.
#[derive(Debug)]
#[must_use]
pub(crate) struct LoaderCacheGuard(Arc<LoaderCache>);

impl Drop for LoaderCacheGuard {
  fn drop(&mut self) {
    let mut active_builds = self.0.active_builds.lock();
    *active_builds -= 1;
    if *active_builds == 0 {
      // Keep the count and notifications ordered across parallel module builds.
      self.0.begin_idle(Duration::ZERO);
    }
  }
}

pub(crate) fn create_loader_cache(
  options: &CompilerOptions,
  compiler_cache: &CompilerCache,
) -> Option<Arc<LoaderCache>> {
  if !options.experiments.new_cache.loader {
    return None;
  }
  let logger = || Arc::new(compiler_cache.logger().get_child("loader"));
  let options = match &options.cache {
    CacheOptions::Disabled => return None,
    CacheOptions::Memory { .. } => loop {
      let current = MEMORY_LOADER_CACHE.load();
      if let Some(cache) = current.upgrade() {
        return Some(cache);
      }

      let cache = LoaderCache::new(Cache::new(Some(MemoryCache::new(5)), None, logger()), None);
      let previous = MEMORY_LOADER_CACHE.compare_and_swap(&current, Arc::downgrade(&cache));
      if Weak::ptr_eq(&current, &previous) {
        return Some(cache);
      }
    },
    CacheOptions::Persistent(options) => options,
  };
  let StorageOptions::FileSystem { directory, .. } = &options.storage;
  let location = directory.join("loader");
  let key = InternedPath::from(location.as_path());

  let entry = loop {
    match PERSISTENT_LOADER_CACHES.entry(key.clone()) {
      Entry::Vacant(entry) => break entry,
      Entry::Occupied(mut entry) => {
        if let Some(cache) = entry.get().cache.upgrade() {
          drop(entry);
          if cache.matches_file_cache_version(&options.version) {
            return Some(cache);
          }
          return memory_cache(options.max_memory_generations)
            .map(|memory| LoaderCache::new(Cache::new(Some(memory), None, logger()), None));
        }

        // The final owner is still closing the database. Release the entry while waiting.
        let closed = Arc::clone(entry.get_mut().closed.get_or_insert_with(Default::default));
        drop(entry);
        closed.wait();
      }
    }
  };

  let logger = logger();
  // Read-only policy is enforced by each compiler before storing loader results.
  let strategy = FileCacheStrategy::new(
    false,
    rspack_workspace::rspack_pkg_version!().to_string(),
    options.version.clone(),
    Arc::new(CacheCodec::new(None)),
    // Omit FileSystemInfo to skip database-wide build dependency tracking.
    None,
    Arc::clone(&logger),
  );
  let file_cache = IdleFileCache::new(
    (directory.clone(), location),
    strategy,
    Arc::clone(&logger),
    Some(DEFAULT_IDLE_TIMEOUT),
    // Keep the loader cache timeout fixed for initial stores and large changes.
    Some(DEFAULT_IDLE_TIMEOUT),
    Some(DEFAULT_IDLE_TIMEOUT),
  );
  let cache = LoaderCache::new(
    Cache::new(
      memory_cache(options.max_memory_generations),
      Some(file_cache),
      logger,
    ),
    Some(key),
  );
  entry.insert(PersistentLoaderCacheEntry {
    cache: Arc::downgrade(&cache),
    closed: None,
  });
  Some(cache)
}
