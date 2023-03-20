use std::{collections::HashMap, fmt::Display, path::PathBuf};

use glob::Pattern as GlobPattern;
use swc_core::ecma::transforms::react::Runtime;
use swc_plugin_import::PluginImportConfig;

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
  pub minify_options: Option<Minification>,
  pub preset_env: Option<PresetEnv>,
  pub define: Define,
  pub tree_shaking: bool,
  pub react: ReactOptions,
  pub decorator: Option<DecoratorOptions>,
  pub no_emit_assets: bool,
  pub emotion: Option<swc_emotion::EmotionOptions>,
  pub dev_friendly_split_chunks: bool,
  pub plugin_import: Option<Vec<PluginImportConfig>>,
  pub relay: Option<RelayConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct Minification {
  pub passes: usize,
  pub drop_console: bool,
  pub pure_funcs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CopyPluginConfig {
  pub patterns: Vec<Pattern>,
}

#[derive(Debug, Clone, Copy)]
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
  pub glob_options: GlobOptions,
}

#[derive(Debug, Clone)]
pub struct GlobOptions {
  pub case_sensitive_match: Option<bool>,
  pub dot: Option<bool>,
  pub ignore: Option<Vec<GlobPattern>>,
}
#[derive(Debug, Clone, Default)]
pub struct PresetEnv {
  pub targets: Vec<String>,
  pub mode: Option<swc_core::ecma::preset_env::Mode>,
  pub core_js: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct RelayConfig {
  pub artifact_directory: Option<PathBuf>,
  pub language: RelayLanguageConfig,
}

#[derive(Copy, Clone, Debug)]
pub enum RelayLanguageConfig {
  JavaScript,
  TypeScript,
  Flow,
}

impl Default for RelayLanguageConfig {
  fn default() -> Self {
    Self::Flow
  }
}
