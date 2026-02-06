use std::sync::Arc;

use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_core::{CrossOriginLoading, ModuleCodegenRuntimeTemplate};
use rspack_error::Result;
use rspack_fs::WritableFileSystem;
use rspack_paths::Utf8PathBuf;
#[cfg(allocative)]
use rspack_util::allocative;
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

#[derive(Debug, Default)]
pub struct SubresourceIntegrityPluginOptions {
  pub hash_func_names: Vec<SubresourceIntegrityHashFunction>,
  pub html_plugin: IntegrityHtmlPlugin,
  #[debug(skip)]
  pub integrity_callback: Option<IntegrityCallbackFn>,
}

pub(crate) type ArcFs = Arc<dyn WritableFileSystem + Send + Sync>;

#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct SRICompilationContext {
  #[cfg_attr(allocative, allocative(skip))]
  pub fs: ArcFs,
  pub output_path: Utf8PathBuf,
  pub cross_origin_loading: CrossOriginLoading,
  pub runtime_template: ModuleCodegenRuntimeTemplate,
}

pub struct IntegrityCallbackData {
  pub integerities: HashMap<String, String>,
}
