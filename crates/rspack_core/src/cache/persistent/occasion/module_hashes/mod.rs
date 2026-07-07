use std::sync::Arc;

use rayon::prelude::*;
use rspack_cacheable::cacheable;
use rspack_collections::IdentifierMap;
use rspack_error::Result;
use rspack_hash::RspackHashDigest;

use super::{
  super::{codec::CacheCodec, storage::Storage},
  Occasion,
};
use crate::{
  CgmHashArtifact, ModuleIdentifier, RayonConsumer, RuntimeKey, RuntimeMode, RuntimeSpec,
  RuntimeSpecMap,
};

pub const SCOPE: &str = "occasion_module_hashes";

const MODE_EMPTY: u8 = 0;
const MODE_SINGLE_ENTRY: u8 = 1;
const MODE_MAP: u8 = 2;

#[cacheable]
struct Entry {
  mode: u8,
  single_runtime: Option<RuntimeSpec>,
  single_value: Option<RspackHashDigest>,
  map: Vec<(RuntimeKey, RspackHashDigest)>,
}

impl Entry {
  fn from_runtime_map(runtime_map: &RuntimeSpecMap<RspackHashDigest>) -> Self {
    match runtime_map.mode {
      RuntimeMode::Empty => Self {
        mode: MODE_EMPTY,
        single_runtime: None,
        single_value: None,
        map: Vec::new(),
      },
      RuntimeMode::SingleEntry => Self {
        mode: MODE_SINGLE_ENTRY,
        single_runtime: runtime_map.single_runtime.clone(),
        single_value: runtime_map.single_value.clone(),
        map: Vec::new(),
      },
      RuntimeMode::Map => Self {
        mode: MODE_MAP,
        single_runtime: None,
        single_value: None,
        map: runtime_map
          .map
          .iter()
          .map(|(runtime, hash)| (runtime.clone(), hash.clone()))
          .collect(),
      },
    }
  }

  fn into_runtime_map(self) -> Option<RuntimeSpecMap<RspackHashDigest>> {
    match self.mode {
      MODE_EMPTY => Some(RuntimeSpecMap::new()),
      MODE_SINGLE_ENTRY => Some(RuntimeSpecMap {
        mode: RuntimeMode::SingleEntry,
        map: Default::default(),
        single_runtime: Some(self.single_runtime?),
        single_value: Some(self.single_value?),
      }),
      MODE_MAP => Some(RuntimeSpecMap {
        mode: RuntimeMode::Map,
        map: self.map.into_iter().collect(),
        single_runtime: None,
        single_value: None,
      }),
      _ => None,
    }
  }
}

#[derive(Debug)]
pub struct ModuleHashesOccasion {
  codec: Arc<CacheCodec>,
}

impl ModuleHashesOccasion {
  pub fn new(codec: Arc<CacheCodec>) -> Self {
    Self { codec }
  }
}

impl Occasion for ModuleHashesOccasion {
  type Artifact = CgmHashArtifact;

  fn name(&self) -> &'static str {
    "module hashes"
  }

  #[tracing::instrument(name = "Cache::Occasion::ModuleHashes::reset", skip_all)]
  fn reset(&self, storage: &mut dyn Storage) {
    storage.reset(SCOPE);
  }

  #[tracing::instrument(name = "Cache::Occasion::ModuleHashes::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, artifact: &CgmHashArtifact) {
    storage.reset(SCOPE);

    let saved_count = std::sync::atomic::AtomicUsize::new(0);
    artifact
      .iter()
      .par_bridge()
      .filter_map(|(module, runtime_map)| {
        let entry = Entry::from_runtime_map(runtime_map);
        match self.codec.encode(&entry) {
          Ok(bytes) => Some((module.as_bytes().to_vec(), bytes)),
          Err(err) => {
            tracing::warn!("module hashes persistent cache encode failed: {:?}", err);
            None
          }
        }
      })
      .consume(|(module, bytes)| {
        storage.set(SCOPE, module, bytes);
        saved_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
      });

    tracing::debug!(
      "saved {} module hashes persistent cache entries",
      saved_count.load(std::sync::atomic::Ordering::Relaxed)
    );
  }

  #[tracing::instrument(name = "Cache::Occasion::ModuleHashes::recovery", skip_all)]
  async fn recovery(&self, storage: &dyn Storage) -> Result<CgmHashArtifact> {
    let items = storage.load(SCOPE).await?;
    let entries = items
      .into_par_iter()
      .filter_map(|(key, value)| {
        let module = match std::str::from_utf8(&key) {
          Ok(module) => ModuleIdentifier::from(module),
          Err(err) => {
            tracing::warn!(
              "module hashes persistent cache key decode failed: {:?}",
              err
            );
            return None;
          }
        };
        let entry = match self.codec.decode::<Entry>(&value) {
          Ok(entry) => entry,
          Err(err) => {
            tracing::warn!("module hashes persistent cache decode failed: {:?}", err);
            return None;
          }
        };
        let Some(runtime_map) = entry.into_runtime_map() else {
          tracing::warn!("module hashes persistent cache entry has invalid runtime mode");
          return None;
        };
        Some((module, runtime_map))
      })
      .collect::<IdentifierMap<RuntimeSpecMap<RspackHashDigest>>>();

    tracing::debug!(
      "recovered {} module hashes persistent cache entries",
      entries.len()
    );
    Ok(entries.into_iter().collect())
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_hash::RspackHashDigest;
  use ustr::Ustr;

  use super::{ModuleHashesOccasion, Occasion};
  use crate::{
    CgmHashArtifact, ModuleIdentifier, RuntimeSpec, RuntimeSpecMap,
    cache::persistent::{codec::CacheCodec, storage::MemoryStorage},
  };

  fn runtime(name: &str) -> RuntimeSpec {
    RuntimeSpec::from_iter([Ustr::from(name)])
  }

  #[tokio::test]
  async fn should_save_and_recover_module_hashes() {
    let occasion = ModuleHashesOccasion::new(Arc::new(CacheCodec::new(None)));
    let mut storage = MemoryStorage::default();

    let module_a = ModuleIdentifier::from("module-a");
    let runtime_a = runtime("a");
    let mut module_a_hashes = RuntimeSpecMap::new();
    module_a_hashes.set(runtime_a.clone(), RspackHashDigest::from("hash-a"));

    let module_b = ModuleIdentifier::from("module-b");
    let runtime_b1 = runtime("b1");
    let runtime_b2 = runtime("b2");
    let mut module_b_hashes = RuntimeSpecMap::new();
    module_b_hashes.set(runtime_b1.clone(), RspackHashDigest::from("hash-b1"));
    module_b_hashes.set(runtime_b2.clone(), RspackHashDigest::from("hash-b2"));

    let mut artifact = CgmHashArtifact::default();
    artifact.set_hashes(module_a, module_a_hashes);
    artifact.set_hashes(module_b, module_b_hashes);
    occasion.save(&mut storage, &artifact);

    let recovered = occasion.recovery(&storage).await.unwrap();
    assert_eq!(
      recovered
        .get(&module_a, &runtime_a)
        .map(|hash| hash.encoded()),
      Some("hash-a")
    );
    assert_eq!(
      recovered
        .get(&module_b, &runtime_b1)
        .map(|hash| hash.encoded()),
      Some("hash-b1")
    );
    assert_eq!(
      recovered
        .get(&module_b, &runtime_b2)
        .map(|hash| hash.encoded()),
      Some("hash-b2")
    );

    let mut replacement = CgmHashArtifact::default();
    let mut module_a_hashes = RuntimeSpecMap::new();
    module_a_hashes.set(runtime_a.clone(), RspackHashDigest::from("hash-a-next"));
    replacement.set_hashes(module_a, module_a_hashes);
    occasion.save(&mut storage, &replacement);

    let recovered = occasion.recovery(&storage).await.unwrap();
    assert_eq!(
      recovered
        .get(&module_a, &runtime_a)
        .map(|hash| hash.encoded()),
      Some("hash-a-next")
    );
    assert!(recovered.get_runtime_map(&module_b).is_none());
  }
}
