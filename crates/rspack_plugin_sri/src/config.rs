use std::sync::Arc;

use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_core::CrossOriginLoading;
use rspack_error::Result;
use rspack_fs::WritableFileSystem;
use rspack_paths::Utf8PathBuf;
#[cfg(allocative)]
use rspack_util::rspack_allocative;
use rustc_hash::FxHashMap as HashMap;

use crate::integrity::SubresourceIntegrityHashFunction;

pub type IntegrityCallbackFn =
  Arc<dyn Fn(IntegrityCallbackData) -> BoxFuture<'static, Result<()>> + Send + Sync>;

#[derive(Debug, Default)]
pub enum IntegrityHtmlPlugin {
  NativePlugin,
  JavaScriptPlugin,
  #[default]
  Disabled,
}

impl TryFrom<String> for IntegrityHtmlPlugin {
  type Error = rspack_error::Error;

  fn try_from(value: String) -> Result<Self, rspack_error::Error> {
    match value.as_str() {
      "JavaScript" => Ok(Self::JavaScriptPlugin),
      "Native" => Ok(Self::NativePlugin),
      "Disabled" => Ok(Self::Disabled),
      _ => Err(rspack_error::Error::error(format!(
        "Invalid integrity html plugin: {value}"
      ))),
    }
  }
}

/// Where the integrity hashes of async chunks are emitted.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityHashLoading {
  /// The runtime chunk carries the hashes of every async chunk it can load.
  #[default]
  Eager,
  /// Each chunk carries the hashes of its direct async children only.
  Lazy,
}

impl TryFrom<String> for IntegrityHashLoading {
  type Error = rspack_error::Error;

  fn try_from(value: String) -> Result<Self, rspack_error::Error> {
    match value.as_str() {
      "eager" => Ok(Self::Eager),
      "lazy" => Ok(Self::Lazy),
      _ => Err(rspack_error::Error::error(format!(
        "Invalid integrity hash loading: {value}, expected \"eager\" or \"lazy\""
      ))),
    }
  }
}

#[derive(Debug, Default)]
pub struct SubresourceIntegrityPluginOptions {
  pub hash_func_names: Vec<SubresourceIntegrityHashFunction>,
  pub html_plugin: IntegrityHtmlPlugin,
  pub hash_loading: IntegrityHashLoading,
  #[debug(skip)]
  pub integrity_callback: Option<IntegrityCallbackFn>,
}

pub type ArcFs = Arc<dyn WritableFileSystem + Send + Sync>;

#[cfg_attr(allocative, derive(rspack_allocative::Allocative))]
pub struct SRICompilationContext {
  #[cfg_attr(allocative, allocative(skip))]
  pub fs: ArcFs,
  pub output_path: Utf8PathBuf,
  pub cross_origin_loading: CrossOriginLoading,
  pub runtime_require_name: String,
}

pub struct IntegrityCallbackData {
  pub integerities: HashMap<String, String>,
}
