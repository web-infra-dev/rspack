use std::{cell::RefCell, future::Future};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{ChunkUkey, SourceType};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContentHashDependency {
  Chunk(ChunkUkey, SourceType),
  Hash(String),
}

#[derive(Debug, Clone, Default)]
pub struct ContentHashDependencies(FxHashSet<ContentHashDependency>);

impl ContentHashDependencies {
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn insert_chunk(&mut self, chunk: ChunkUkey, source_type: SourceType) {
    self
      .0
      .insert(ContentHashDependency::Chunk(chunk, source_type));
  }

  pub fn insert_hash(&mut self, hash: impl Into<String>) {
    self.0.insert(ContentHashDependency::Hash(hash.into()));
  }

  pub fn extend(&mut self, other: &Self) {
    self.0.extend(other.0.iter().cloned());
  }

  pub fn iter(&self) -> impl Iterator<Item = &ContentHashDependency> {
    self.0.iter()
  }
}

tokio::task_local! {
  static CURRENT_CONTENT_HASH_DEPENDENCIES: RefCell<ContentHashDependencies>;
}

pub async fn collect_content_hash_dependencies<F>(
  enabled: bool,
  future: F,
) -> (F::Output, ContentHashDependencies)
where
  F: Future,
{
  if !enabled {
    return (future.await, ContentHashDependencies::default());
  }

  CURRENT_CONTENT_HASH_DEPENDENCIES
    .scope(
      RefCell::new(ContentHashDependencies::default()),
      async move {
        let output = future.await;
        let dependencies = CURRENT_CONTENT_HASH_DEPENDENCIES
          .with(|dependencies| std::mem::take(&mut *dependencies.borrow_mut()));
        (output, dependencies)
      },
    )
    .await
}

pub fn record_chunk_content_hash_dependency(chunk: ChunkUkey, source_type: SourceType) {
  _ = CURRENT_CONTENT_HASH_DEPENDENCIES
    .try_with(|dependencies| dependencies.borrow_mut().insert_chunk(chunk, source_type));
}

#[derive(Debug, Clone, Default)]
pub struct RealContentHashAssetRecord {
  own_hashes: FxHashSet<String>,
  dependencies: ContentHashDependencies,
  chunk_source: Option<(ChunkUkey, SourceType)>,
}

impl RealContentHashAssetRecord {
  pub fn own_hashes(&self) -> &FxHashSet<String> {
    &self.own_hashes
  }

  pub fn dependencies(&self) -> &ContentHashDependencies {
    &self.dependencies
  }
}

#[derive(Debug, Default)]
pub struct RealContentHashArtifact {
  asset_records: FxHashMap<String, RealContentHashAssetRecord>,
  chunk_hashes: FxHashMap<(ChunkUkey, SourceType), FxHashSet<String>>,
}

impl RealContentHashArtifact {
  pub fn clear(&mut self) {
    self.asset_records.clear();
    self.chunk_hashes.clear();
  }

  pub fn record_asset(
    &mut self,
    asset: String,
    chunk: ChunkUkey,
    source_type: Option<SourceType>,
    own_hashes: impl IntoIterator<Item = String>,
    dependencies: ContentHashDependencies,
  ) {
    let own_hashes = own_hashes.into_iter().collect::<FxHashSet<_>>();
    let record = self.asset_records.entry(asset).or_default();
    record.own_hashes.extend(own_hashes.iter().cloned());
    record.dependencies.extend(&dependencies);
    record.chunk_source = source_type.map(|source_type| (chunk, source_type));

    if let Some(source_type) = source_type {
      self
        .chunk_hashes
        .entry((chunk, source_type))
        .or_default()
        .extend(own_hashes);
    }
  }

  pub fn asset_record(&self, asset: &str) -> Option<&RealContentHashAssetRecord> {
    self.asset_records.get(asset)
  }

  pub fn add_asset_content_hash(&mut self, asset: &str, hash: String) {
    let Some(record) = self.asset_records.get_mut(asset) else {
      return;
    };
    record.own_hashes.insert(hash.clone());
    if let Some(chunk_source) = record.chunk_source {
      self
        .chunk_hashes
        .entry(chunk_source)
        .or_default()
        .insert(hash);
    }
  }

  pub fn add_asset_content_hash_dependency(&mut self, asset: &str, hash: String) {
    if let Some(record) = self.asset_records.get_mut(asset) {
      record.dependencies.insert_hash(hash);
    }
  }

  pub fn chunk_hashes(
    &self,
    chunk: ChunkUkey,
    source_type: SourceType,
  ) -> Option<&FxHashSet<String>> {
    self.chunk_hashes.get(&(chunk, source_type))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn collects_dependencies_only_inside_enabled_scope() {
    record_chunk_content_hash_dependency(ChunkUkey::new(), SourceType::JavaScript);

    let (_, disabled) = collect_content_hash_dependencies(false, async {
      record_chunk_content_hash_dependency(ChunkUkey::new(), SourceType::JavaScript);
    })
    .await;
    assert!(disabled.is_empty());

    let chunk = ChunkUkey::new();
    let (_, enabled) = collect_content_hash_dependencies(true, async {
      record_chunk_content_hash_dependency(chunk, SourceType::JavaScript);
      record_chunk_content_hash_dependency(chunk, SourceType::JavaScript);
    })
    .await;
    assert_eq!(enabled.iter().count(), 1);
  }

  #[test]
  fn extends_asset_records_after_chunk_rendering() {
    let chunk = ChunkUkey::new();
    let mut artifact = RealContentHashArtifact::default();
    artifact.record_asset(
      "main.js".to_string(),
      chunk,
      Some(SourceType::JavaScript),
      ["content-hash".to_string()],
      ContentHashDependencies::default(),
    );

    artifact.add_asset_content_hash("main.js", "integrity-hash".to_string());
    artifact.add_asset_content_hash_dependency("main.js", "referenced-integrity".to_string());

    let record = artifact.asset_record("main.js").expect("recorded asset");
    assert!(record.own_hashes().contains("integrity-hash"));
    assert!(record.dependencies().iter().any(
      |dependency| matches!(dependency, ContentHashDependency::Hash(hash) if hash == "referenced-integrity")
    ));
    assert!(
      artifact
        .chunk_hashes(chunk, SourceType::JavaScript)
        .expect("recorded chunk source")
        .contains("integrity-hash")
    );
  }
}
