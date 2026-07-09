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
