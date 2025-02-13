use std::sync::Arc;

use derive_more::Debug;
use rspack_core::CrossOriginLoading;
use rspack_error::Result;
use rspack_fs::WritableFileSystem;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashMap as HashMap;

use crate::integrity::SubresourceIntegrityHashFunction;

pub type IntegrityCallbackFn = Arc<dyn Fn(IntegrityCallbackData) -> Result<()> + Send + Sync>;

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
      _ => panic!("Invalid integrity html plugin: {}", value),
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
pub struct SRICompilationContext {
  pub fs: ArcFs,
  pub output_path: Utf8PathBuf,
  pub cross_origin_loading: CrossOriginLoading,
}

pub struct IntegrityCallbackData {
  pub integerities: HashMap<String, String>,
}
