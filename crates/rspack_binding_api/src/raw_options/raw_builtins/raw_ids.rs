use napi_derive::napi;
use rspack_hash::{HashDigest, HashFunction};
use rspack_ids::{HashedModuleIdsPluginOptions, OccurrenceChunkIdsPluginOptions};

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawOccurrenceChunkIdsPluginOptions {
  pub prioritise_initial: Option<bool>,
}

impl From<RawOccurrenceChunkIdsPluginOptions> for OccurrenceChunkIdsPluginOptions {
  fn from(value: RawOccurrenceChunkIdsPluginOptions) -> Self {
    Self {
      prioritise_initial: value.prioritise_initial.unwrap_or_default(),
    }
  }
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawHashedModuleIdsPluginOptions {
  pub context: Option<String>,
  pub hash_function: Option<String>,
  pub hash_digest: Option<String>,
  pub hash_digest_length: Option<u32>,
}

impl From<RawHashedModuleIdsPluginOptions> for HashedModuleIdsPluginOptions {
  fn from(value: RawHashedModuleIdsPluginOptions) -> Self {
    let defaults = HashedModuleIdsPluginOptions::default();
    Self {
      context: value.context,
      hash_function: value
        .hash_function
        .map(|s| HashFunction::from(s.as_str()))
        .unwrap_or(defaults.hash_function),
      hash_digest: value
        .hash_digest
        .map(|s| HashDigest::from(s.as_str()))
        .unwrap_or(defaults.hash_digest),
      hash_digest_length: value
        .hash_digest_length
        .map(|n| n as usize)
        .unwrap_or(defaults.hash_digest_length),
    }
  }
}
