use std::{ptr::NonNull, sync::Arc};

use futures::future::BoxFuture;
use napi_derive::napi;
use rspack_core::{CompilerId, Module};
use rspack_hash::{HashDigest, HashFunction};
use rspack_ids::{
  DeterministicModuleIdsPluginOptions, HashedModuleIdsPluginOptions, ModuleFilterFn,
  OccurrenceChunkIdsPluginOptions,
};

use crate::{
  compiler_scoped_tsfn::CompilerScopedTsFnHandle as ThreadsafeFunction, module::ModuleObject,
};

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
        .map_or(defaults.hash_function, |s| HashFunction::from(s.as_str())),
      hash_digest: value
        .hash_digest
        .map_or(defaults.hash_digest, |s| HashDigest::from(s.as_str())),
      hash_digest_length: value
        .hash_digest_length
        .map_or(defaults.hash_digest_length, |n| n as usize),
    }
  }
}

type RawModuleFilter = ThreadsafeFunction<ModuleObject, Option<bool>>;

fn into_module_filter(test: RawModuleFilter) -> ModuleFilterFn {
  Arc::new(
    move |compiler_id: CompilerId,
          module: &dyn Module|
          -> BoxFuture<'_, rspack_error::Result<bool>> {
      let test = test.clone();
      let module = ModuleObject::with_ptr(
        NonNull::new(module as *const dyn Module as *mut dyn Module)
          .expect("module pointer should not be null"),
        compiler_id,
      );
      Box::pin(async move { Ok(test.call_with_sync(module).await?.unwrap_or(false)) })
    },
  )
}

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawDeterministicModuleIdsPluginOptions {
  pub context: Option<String>,
  #[napi(ts_type = "(module: Module) => boolean")]
  pub test: Option<RawModuleFilter>,
  pub max_length: Option<u32>,
  pub salt: Option<u32>,
  pub fixed_length: Option<bool>,
  pub fail_on_conflict: Option<bool>,
}

impl From<RawDeterministicModuleIdsPluginOptions> for DeterministicModuleIdsPluginOptions {
  fn from(value: RawDeterministicModuleIdsPluginOptions) -> Self {
    Self {
      context: value.context,
      test: value.test.map(into_module_filter),
      max_length: value.max_length.map(|n| n as usize),
      salt: value.salt.map(|n| n as usize),
      fixed_length: value.fixed_length,
      fail_on_conflict: value.fail_on_conflict,
    }
  }
}
