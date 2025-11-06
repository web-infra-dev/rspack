use std::sync::Arc;

use napi::bindgen_prelude::Either3;
use rayon::iter::Either;
use rspack_core::CompilerId;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_esm_library::{CacheGroup, EsmLibraryPlugin};
use rspack_regex::RspackRegex;

use crate::module::ModuleObject;

pub type RawNameGetter = Either<String, ThreadsafeFunction<ModuleObject, Option<String>>>;

pub type RawCacheGroupTest =
  Either3<String, RspackRegex, ThreadsafeFunction<ModuleObject, Option<bool>>>;

#[napi(object, object_to_js = false)]
pub struct RawCacheGroup {
  pub name: Option<RawNameGetter>,
  pub test: Option<RawCacheGroupTest>,
  pub key: String,
  pub r#type: Option<String>,
  pub filename: Option<String>,
  pub priority: Option<f64>,
  pub min_size: Option<f64>,
}

#[napi(object, object_to_js = false)]
pub struct RawEsmLibraryPlugin {
  pub preserve_modules: Option<String>,
  pub split_chunks: Option<Vec<RawCacheGroup>>,
}

impl From<RawCacheGroup> for EsmLibraryPlugin::CacheGroup {
  fn from(value: RawCacheGroup) -> Self {
    CacheGroup {
      name: match value.name {
        Some(Either::Right(name_getter)) => {
          Either::Right(Arc::new(async move |module, compilation| {
            Box::pin(async move {
              name_getter
                .call_with_sync(ModuleObject::with_ref(module, compilation.compiler_id()))
                .await
            })
          }))
        }
        Some(Either::Left(name)) => Either::Left(Some(name)),
        None => Either::Left(None),
      },
      key: value.key,
      test: match value.test {
        Some(Either3::A(string)) => {
          Arc::new(async move |module, _| module.identifier().contains(&string))
        }
        Some(Either3::B(regex)) => Arc::new(|module| regex.test(module.identifier().as_str())),
        Some(Either3::C(test_fn)) => Arc::new(async |module, compilation| {
          test_fn
            .call_with_sync(ModuleObject::with_ref(module, compilation.compiler_id()))
            .await
            .unwrap_or(false)
        }),
        None => Arc::new(|_| true),
      },
      r#type: match value.r#type {
        Some(raw) => crate::split_chunks::normalize_raw_cache_group_type(raw),
        None => raw_split_chunks::default_cache_group_type(),
      },
      filename: value.filename,
      priority: value.priority.unwrap_or(0.0),
      min_size: value
        .min_size
        .map(|s| crate::split_chunks::SplitChunkSizes {
          javascript: s,
          ..Default::default()
        }),
      index: 0, // will be set later
    }
  }
}
