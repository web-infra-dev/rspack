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
const CODE_GENERATION_SCOPE: &str = "occasion_module_code_generation_hashes";

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
    storage.reset(CODE_GENERATION_SCOPE);
  }

  #[tracing::instrument(name = "Cache::Occasion::ModuleHashes::save", skip_all)]
  fn save(&self, storage: &mut dyn Storage, artifact: &CgmHashArtifact) {
    storage.reset(SCOPE);
    storage.reset(CODE_GENERATION_SCOPE);

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

    let saved_code_generation_hash_count = std::sync::atomic::AtomicUsize::new(0);
    artifact
      .code_generation_hashes_iter()
      .par_bridge()
      .filter_map(|(module, hash)| {
        let key = match self.codec.encode(module) {
          Ok(bytes) => bytes,
          Err(err) => {
            tracing::warn!(
              "module code generation hash persistent cache key encode failed: {:?}",
              err
            );
            return None;
          }
        };
        match self.codec.encode(hash) {
          Ok(bytes) => Some((key, bytes)),
          Err(err) => {
            tracing::warn!(
              "module code generation hash persistent cache encode failed: {:?}",
              err
            );
            None
          }
        }
      })
      .consume(|(key, bytes)| {
        storage.set(CODE_GENERATION_SCOPE, key, bytes);
        saved_code_generation_hash_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
      });

    tracing::debug!(
      "saved {} module code generation hashes persistent cache entries",
      saved_code_generation_hash_count.load(std::sync::atomic::Ordering::Relaxed)
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
    let mut artifact: CgmHashArtifact = entries.into_iter().collect();

    let code_generation_hash_items = storage.load(CODE_GENERATION_SCOPE).await?;
    let code_generation_hashes = code_generation_hash_items
      .into_par_iter()
      .map(|(key, value)| {
        let module = self.codec.decode::<ModuleIdentifier>(&key).map_err(|err| {
          rspack_error::error!(
            "module code generation hash persistent cache key decode failed: {err}"
          )
        })?;
        let hash = self
          .codec
          .decode::<RspackHashDigest>(&value)
          .map_err(|err| {
            rspack_error::error!(
              "module code generation hash persistent cache decode failed: {err}"
            )
          })?;
        Ok((module, hash))
      })
      .collect::<Result<IdentifierMap<RspackHashDigest>>>()?;
    let code_generation_hash_count = code_generation_hashes.len();
    for (module, hash) in code_generation_hashes {
      artifact.set_code_generation_hash(module, hash);
    }
    tracing::debug!(
      "recovered {} module code generation hashes persistent cache entries",
      code_generation_hash_count
    );

    Ok(artifact)
  }
}
