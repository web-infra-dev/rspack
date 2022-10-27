use std::collections::HashMap;

use swc_ecma_transforms::react::Runtime;

pub type Define = HashMap<String, String>;

#[derive(Debug, Clone, Default)]
pub struct ReactOptions {
  pub runtime: Option<Runtime>,
  pub import_source: Option<String>,
  pub pragma: Option<String>,
  pub pragma_frag: Option<String>,
  pub throw_if_namespace: Option<bool>,
  pub development: Option<bool>,
  pub use_builtins: Option<bool>,
  pub use_spread: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct Builtins {
  pub minify: bool,
  pub polyfill: bool,
  pub browserslist: Vec<String>,
  pub define: Define,
  pub tree_shaking: bool,
  pub react: ReactOptions,
}
