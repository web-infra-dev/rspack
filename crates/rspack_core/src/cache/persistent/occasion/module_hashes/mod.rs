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
      .filter_map(|(module, runtime_map)| {
        let key = match self.codec.encode(module) {
          Ok(bytes) => bytes,
          Err(err) => {
            tracing::warn!(
              "module hashes persistent cache key encode failed: {:?}",
              err
            );
            return None;
          }
        };
        match self.codec.encode(runtime_map) {
          Ok(bytes) => Some((key, bytes)),
          Err(err) => {
            tracing::warn!("module hashes persistent cache encode failed: {:?}", err);
            None
          }
        }
      })
      .consume(|(key, bytes)| {
        storage.set(SCOPE, key, bytes);
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
      .map(|(key, value)| {
        let module = self.codec.decode::<ModuleIdentifier>(&key).map_err(|err| {
          rspack_error::error!("module hashes persistent cache key decode failed: {err}")
        })?;
        let runtime_map = self
          .codec
          .decode::<RuntimeSpecMap<RspackHashDigest>>(&value)
          .map_err(|err| {
            rspack_error::error!("module hashes persistent cache decode failed: {err}")
          })?;
        Ok((module, runtime_map))
      })
      .collect::<Result<IdentifierMap<RuntimeSpecMap<RspackHashDigest>>>>()?;

    tracing::debug!(
      "recovered {} module hashes persistent cache entries",
      entries.len()
    );
    Ok(entries.into_iter().collect())
  }
}
