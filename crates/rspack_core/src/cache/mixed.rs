use super::{Cache, memory::MemoryCache, persistent::PersistentCache};
use crate::{
  Compilation, compilation::build_module_graph::BuildModuleGraphArtifact, incremental::Incremental,
};

/// Mixed cache implementation
///
/// The mixed cache combines both persistent cache and memory cache:
/// - PersistentCache: Used for initial build to recover cache from disk and save cache after compilation
/// - MemoryCache: Used for rebuild to recover artifacts from previous compilation in memory
///
/// This provides the best of both worlds:
/// - Fast rebuilds using in-memory cache
/// - Cache persistence across process restarts using disk storage
#[derive(Debug)]
pub struct MixedCache {
  persistent: PersistentCache,
  memory: MemoryCache,
}

impl MixedCache {
  pub fn new(persistent: PersistentCache) -> Self {
    Self {
      persistent,
      memory: MemoryCache::default(),
    }
  }
}

#[async_trait::async_trait]
impl Cache for MixedCache {
  async fn before_compile(&mut self, compilation: &mut Compilation) -> bool {
    // For rebuild, use memory cache to check if it's a hot start
    // For initial build, use persistent cache
    if compilation.is_rebuild {
      self.memory.before_compile(compilation).await
    } else {
      self.persistent.before_compile(compilation).await
    }
  }

  async fn after_compile(&mut self, compilation: &Compilation) {
    // Always save to persistent cache
    self.persistent.after_compile(compilation).await;
  }

  fn store_old_compilation(&mut self, compilation: Box<Compilation>) {
    // Store in memory cache for fast rebuild
    self.memory.store_old_compilation(compilation);
  }

  // BUILD_MODULE_GRAPH hooks
  async fn before_build_module_graph(&mut self, compilation: &mut Compilation) {
    // First try memory cache (for rebuild)
    self.memory.before_build_module_graph(compilation).await;
    // Then try persistent cache (for initial build)
    self.persistent.before_build_module_graph(compilation).await;
  }

  async fn after_build_module_graph(&self, compilation: &Compilation) {
    // Save to persistent cache
    self.persistent.after_build_module_graph(compilation).await;
  }

  // FINISH_MODULES hooks
  async fn before_finish_modules(&mut self, compilation: &mut Compilation) {
    // Use memory cache for rebuild
    self.memory.before_finish_modules(compilation).await;
    // Persistent cache currently doesn't implement this
    self.persistent.before_finish_modules(compilation).await;
  }

  async fn after_finish_modules(&self, compilation: &Compilation) {
    self.persistent.after_finish_modules(compilation).await;
  }

  // OPTIMIZE_DEPENDENCIES hooks
  async fn before_optimize_dependencies(&mut self, compilation: &mut Compilation) {
    self.memory.before_optimize_dependencies(compilation).await;
    self
      .persistent
      .before_optimize_dependencies(compilation)
      .await;
  }

  async fn after_optimize_dependencies(&self, compilation: &Compilation) {
    self
      .persistent
      .after_optimize_dependencies(compilation)
      .await;
  }

  // BUILD_CHUNK_GRAPH hooks
  async fn before_build_chunk_graph(&mut self, compilation: &mut Compilation) {
    self.memory.before_build_chunk_graph(compilation).await;
    self.persistent.before_build_chunk_graph(compilation).await;
  }

  async fn after_build_chunk_graph(&mut self, compilation: &mut Compilation) {
    self.memory.after_build_chunk_graph(compilation).await;
    self.persistent.after_build_chunk_graph(compilation).await;
  }

  // MODULE_IDS hooks
  async fn before_module_ids(&mut self, compilation: &mut Compilation) {
    self.memory.before_module_ids(compilation).await;
    self.persistent.before_module_ids(compilation).await;
  }

  async fn after_module_ids(&self, compilation: &Compilation) {
    self.persistent.after_module_ids(compilation).await;
  }

  // CHUNK_IDS hooks
  async fn before_chunk_ids(&mut self, compilation: &mut Compilation) {
    self.memory.before_chunk_ids(compilation).await;
    self.persistent.before_chunk_ids(compilation).await;
  }

  async fn after_chunk_ids(&self, compilation: &Compilation) {
    self.persistent.after_chunk_ids(compilation).await;
  }

  // MODULES_HASHES hooks
  async fn before_modules_hashes(&mut self, compilation: &mut Compilation) {
    self.memory.before_modules_hashes(compilation).await;
    self.persistent.before_modules_hashes(compilation).await;
  }

  async fn after_modules_hashes(&self, compilation: &Compilation) {
    self.persistent.after_modules_hashes(compilation).await;
  }

  // MODULES_CODEGEN hooks
  async fn before_modules_codegen(&mut self, compilation: &mut Compilation) {
    self.memory.before_modules_codegen(compilation).await;
    self.persistent.before_modules_codegen(compilation).await;
  }

  async fn after_modules_codegen(&self, compilation: &Compilation) {
    self.persistent.after_modules_codegen(compilation).await;
  }

  // MODULES_RUNTIME_REQUIREMENTS hooks
  async fn before_modules_runtime_requirements(&mut self, compilation: &mut Compilation) {
    self
      .memory
      .before_modules_runtime_requirements(compilation)
      .await;
    self
      .persistent
      .before_modules_runtime_requirements(compilation)
      .await;
  }

  async fn after_modules_runtime_requirements(&self, compilation: &Compilation) {
    self
      .persistent
      .after_modules_runtime_requirements(compilation)
      .await;
  }

  // CHUNKS_RUNTIME_REQUIREMENTS hooks
  async fn before_chunks_runtime_requirements(&mut self, compilation: &mut Compilation) {
    self
      .memory
      .before_chunks_runtime_requirements(compilation)
      .await;
    self
      .persistent
      .before_chunks_runtime_requirements(compilation)
      .await;
  }

  async fn after_chunks_runtime_requirements(&self, compilation: &Compilation) {
    self
      .persistent
      .after_chunks_runtime_requirements(compilation)
      .await;
  }

  // CHUNKS_HASHES hooks
  async fn before_chunks_hashes(&mut self, compilation: &mut Compilation) {
    self.memory.before_chunks_hashes(compilation).await;
    self.persistent.before_chunks_hashes(compilation).await;
  }

  async fn after_chunks_hashes(&self, compilation: &Compilation) {
    self.persistent.after_chunks_hashes(compilation).await;
  }

  // CHUNK_ASSET hooks
  async fn before_chunk_asset(&mut self, compilation: &mut Compilation) {
    self.memory.before_chunk_asset(compilation).await;
    self.persistent.before_chunk_asset(compilation).await;
  }

  async fn after_chunk_asset(&self, compilation: &Compilation) {
    self.persistent.after_chunk_asset(compilation).await;
  }

  // EMIT_ASSETS hooks
  async fn before_emit_assets(&mut self, compilation: &mut Compilation) {
    self.memory.before_emit_assets(compilation).await;
    self.persistent.before_emit_assets(compilation).await;
  }

  async fn after_emit_assets(&self, compilation: &Compilation) {
    self.persistent.after_emit_assets(compilation).await;
  }
}
