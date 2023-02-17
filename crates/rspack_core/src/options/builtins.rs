use std::{collections::HashMap, fmt::Display, path::PathBuf};

use swc_core::ecma::transforms::react::Runtime;

use crate::AssetInfo;

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
  pub browserslist: Vec<String>,
  pub define: Define,
  pub tree_shaking: bool,
  pub react: ReactOptions,
  pub decorator: Option<DecoratorOptions>,
  pub no_emit_assets: bool,
  pub emotion: Option<swc_emotion::EmotionOptions>,
  pub dev_friendly_split_chunks: bool,
  pub copy: Option<CopyPluginConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct Minification {
  pub enable: bool,
  pub passes: usize,
  pub drop_console: bool,
  pub pure_funcs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CopyPluginConfig {
  pub patterns: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub enum FromType {
  Dir,
  File,
  Glob,
}

#[derive(Debug, Clone)]
pub enum ToType {
  Dir,
  File,
  Template,
}

impl Display for ToType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      ToType::Dir => "dir",
      ToType::File => "file",
      ToType::Template => "template",
    })
  }
}

#[derive(Debug, Clone)]
pub struct Pattern {
  pub from: String,
  pub to: Option<String>,
  pub context: Option<PathBuf>,
  pub to_type: Option<ToType>,
  pub no_error_on_missing: bool,
  pub info: Option<AssetInfo>,
  pub force: bool,
  pub priority: i32,
  pub glob_options: Option<GlobOptions>,
}

#[derive(Debug, Clone)]
pub struct GlobOptions {
  pub case_sensitive_match: Option<bool>,
  pub dot: Option<bool>,
}
