use std::sync::Arc;

use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_core::CrossOriginLoading;
use rspack_error::Result;
use rspack_fs::WritableFileSystem;
use rspack_paths::Utf8PathBuf;
#[cfg(allocative)]
use rspack_util::allocative;
use rustc_hash::FxHashMap as HashMap;

use crate::integrity::SubresourceIntegrityHashFunction;

pub type IntegrityCallbackFn =
  Arc<dyn Fn(IntegrityCallbackData) -> BoxFuture<'static, Result<()>> + Send + Sync>;

#[derive(Debug)]
pub enum IntegrityHtmlPlugin {
  NativePlugin,
  JavaScriptPlugin,
  Disabled,
}

impl From<String> for IntegrityHtmlPlugin {
  fn from(value: String) -> Self {
    match value.as_str() {
      "JavaScript" => Self::JavaScriptPlugin,
      "Native" => Self::NativePlugin,
      "Disabled" => Self::Disabled,
      _ => panic!("Invalid integrity html plugin: {value}"),
    }
  }
}

#[derive(Debug)]
pub struct SubresourceIntegrityPluginOptions {
  pub hash_func_names: Vec<SubresourceIntegrityHashFunction>,
  pub html_plugin: IntegrityHtmlPlugin,
  #[debug(skip)]
  pub integrity_callback: Option<IntegrityCallbackFn>,
}

pub type ArcFs = Arc<dyn WritableFileSystem + Send + Sync>;

#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct SRICompilationContext {
  #[cfg_attr(allocative, allocative(skip))]
  pub fs: ArcFs,
  pub output_path: Utf8PathBuf,
  pub cross_origin_loading: CrossOriginLoading,
}

pub struct IntegrityCallbackData {
  pub integerities: HashMap<String, String>,
}
