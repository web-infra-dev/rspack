use std::collections::HashMap;

use rspack_cacheable::cacheable;
use serde::Deserialize;

pub type ModularizeImports = HashMap<String, PackageConfig>;

#[cacheable]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageConfig {
  pub transform: Transform,
  #[serde(default)]
  pub prevent_full_import: bool,
  #[serde(default)]
  pub handle_default_import: bool,
  #[serde(default)]
  pub handle_namespace_import: bool,
  #[serde(default)]
  pub skip_default_conversion: bool,
}

#[cacheable]
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Transform {
  String(String),
  Vec(Vec<(String, String)>),
}

#[cacheable]
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NextSwcLoaderJsOptions {
  pub root_dir: String,

  pub is_server: bool,

  pub pages_dir: Option<String>,

  pub app_dir: Option<String>,

  pub has_react_refresh: bool,

  #[serde(default)]
  pub optimize_server_react: bool,

  // next_config
  // js_config
  #[serde(default)]
  pub supported_browsers: Vec<String>,

  pub swc_cache_dir: String,

  #[serde(default)]
  pub server_components: Option<bool>,

  pub server_reference_hash_salt: String,

  pub bundle_layer: Option<String>,

  #[serde(default)]
  pub esm: bool,

  #[serde(default)]
  pub transpile_packages: Vec<String>,

  // rspack specific options
  #[serde(default)]
  pub pnp: bool,

  #[serde(default)]
  pub modularize_imports: Option<ModularizeImports>,

  #[serde(default)]
  pub decorators: bool,

  #[serde(default)]
  pub emit_decorator_metadata: bool,

  #[serde(default)]
  pub regenerator_runtime_path: Option<String>,
}

impl TryFrom<&str> for NextSwcLoaderJsOptions {
  type Error = serde_json::Error;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    serde_json::from_str(value)
  }
}
