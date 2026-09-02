use super::{Cache, memory::MemoryCache, persistent::PersistentCache};
use crate::Compilation;

/// Combines process-local and persistent build caches.
#[derive(Debug)]
pub struct MixedCache {
  persistent: PersistentCache,
  memory: Option<MemoryCache>,
}

impl MixedCache {
  pub fn new(persistent: PersistentCache, use_memory_cache: bool) -> Self {
    Self {
      persistent,
      memory: use_memory_cache.then(MemoryCache::default),
    }
  }
}

#[async_trait::async_trait]
impl Cache for MixedCache {
  async fn before_compile(&mut self, compilation: &mut Compilation) -> bool {
    if compilation.is_rebuild {
      if let Some(memory) = &mut self.memory {
        memory.before_compile(compilation).await
      } else {
        false
      }
    } else {
      self.persistent.before_compile(compilation).await
    }
  }

  async fn after_compile(&mut self, compilation: &Compilation) {
    self.persistent.after_compile(compilation).await;
  }

  async fn before_build_module_graph(&mut self, compilation: &mut Compilation) {
    self.persistent.before_build_module_graph(compilation).await;
  }

  async fn after_build_module_graph(&mut self, compilation: &Compilation) {
    self.persistent.after_build_module_graph(compilation).await;
  }

  async fn before_process_assets(&mut self, compilation: &mut Compilation) {
    if let Some(memory) = &mut self.memory {
      memory.before_process_assets(compilation).await;
    }
    self.persistent.before_process_assets(compilation).await;
  }

  async fn after_process_assets(&mut self, compilation: &Compilation) {
    self.persistent.after_process_assets(compilation).await;
  }

  fn store_hot_cache(&mut self, compilation: &mut Compilation) {
    if let Some(memory) = &mut self.memory {
      memory.store_hot_cache(compilation);
    }
  }

  async fn close(&self) {
    self.persistent.close().await;
  }
}
