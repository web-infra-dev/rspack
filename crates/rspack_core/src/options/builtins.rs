use std::collections::HashMap;

use swc_core::ecma::transforms::react::Runtime;

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
  pub refresh: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct DecoratorOptions {
  // https://swc.rs/docs/configuration/compilation#jsctransformlegacydecorator
  pub legacy: bool,
  // https://swc.rs/docs/configuration/compilation#jsctransformdecoratormetadata
  pub emit_metadata: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Builtins {
  pub minify: Minification,
  pub polyfill: bool,
  pub preset_env: Vec<String>,
  pub define: Define,
  pub tree_shaking: bool,
  pub react: ReactOptions,
  pub decorator: Option<DecoratorOptions>,
  pub no_emit_assets: bool,
  pub emotion: Option<swc_emotion::EmotionOptions>,
  pub dev_friendly_split_chunks: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Minification {
  pub enable: bool,
  pub passes: usize,
  pub drop_console: bool,
  pub pure_funcs: Vec<String>,
}
