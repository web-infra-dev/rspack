use std::sync::Arc;

use rayon::prelude::*;
use rspack_collections::IdentifierMap;
use rspack_error::Result;
use rspack_hash::RspackHashDigest;

use super::{
  super::{codec::CacheCodec, storage::Storage},
  Occasion,
};
use crate::{CgmHashArtifact, ModuleIdentifier, RayonConsumer, RuntimeSpecMap};

pub const SCOPE: &str = "occasion_module_hashes";

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
      .filter_map(
        |(module, runtime_map)| match self.codec.encode(runtime_map) {
          Ok(bytes) => Some((module.as_bytes().to_vec(), bytes)),
          Err(err) => {
            tracing::warn!("module hashes persistent cache encode failed: {:?}", err);
            None
          }
        },
      )
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
        let runtime_map = match self
          .codec
          .decode::<RuntimeSpecMap<RspackHashDigest>>(&value)
        {
          Ok(runtime_map) => runtime_map,
          Err(err) => {
            tracing::warn!("module hashes persistent cache decode failed: {:?}", err);
            return None;
          }
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
