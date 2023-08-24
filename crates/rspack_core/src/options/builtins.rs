use std::fmt::Debug;
use std::{collections::HashMap, fmt::Display, path::PathBuf};

use glob::Pattern as GlobPattern;
use rspack_error::Result;
use swc_core::ecma::transforms::react::Runtime;
use swc_plugin_import::PluginImportConfig;

use crate::{ApplyContext, AssetInfo, CompilerOptions, Plugin, PluginContext};

pub type Define = HashMap<String, String>;

#[derive(Debug)]
pub struct DefinePlugin {
  options: Define,
}

impl DefinePlugin {
  pub fn new(options: Define) -> Self {
    Self { options }
  }
}

impl Plugin for DefinePlugin {
  fn name(&self) -> &'static str {
    "rspack.DefinePlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.define.extend(self.options.clone());
    Ok(())
  }
}

pub type Provide = HashMap<String, Vec<String>>;

#[derive(Debug)]
pub struct ProvidePlugin {
  options: Provide,
}

impl ProvidePlugin {
  pub fn new(options: Provide) -> Self {
    Self { options }
  }
}

impl Plugin for ProvidePlugin {
  fn name(&self) -> &'static str {
    "rspack.ProvidePlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.provide.extend(self.options.clone());
    Ok(())
  }
}

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

#[derive(Debug)]
pub struct ReactOptionsPlugin {
  options: ReactOptions,
}

impl ReactOptionsPlugin {
  pub fn new(options: ReactOptions) -> Self {
    Self { options }
  }
}

impl Plugin for ReactOptionsPlugin {
  fn name(&self) -> &'static str {
    "rspack.ReactOptionsPlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.react = self.options.clone();
    Ok(())
  }
}

#[derive(Debug, Clone, Default)]
pub struct DecoratorOptions {
  // https://swc.rs/docs/configuration/compilation#jsctransformlegacydecorator
  pub legacy: bool,
  // https://swc.rs/docs/configuration/compilation#jsctransformdecoratormetadata
  pub emit_metadata: bool,
}

#[derive(Debug)]
pub struct DecoratorOptionsPlugin {
  options: DecoratorOptions,
}

impl DecoratorOptionsPlugin {
  pub fn new(options: DecoratorOptions) -> Self {
    Self { options }
  }
}

impl Plugin for DecoratorOptionsPlugin {
  fn name(&self) -> &'static str {
    "rspack.DecoratorOptionsPlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.decorator = Some(self.options.clone());
    Ok(())
  }
}

#[derive(Debug, Clone, Default, Copy)]
pub enum TreeShaking {
  True,
  #[default]
  False,
  Module,
}

impl TreeShaking {
  pub fn enable(&self) -> bool {
    matches!(self, TreeShaking::Module | TreeShaking::True)
  }

  /// Returns `true` if the tree shaking is [`True`].
  ///
  /// [`True`]: TreeShaking::True
  #[must_use]
  pub fn is_true(&self) -> bool {
    matches!(self, Self::True)
  }

  /// Returns `true` if the tree shaking is [`False`].
  ///
  /// [`False`]: TreeShaking::False
  #[must_use]
  pub fn is_false(&self) -> bool {
    matches!(self, Self::False)
  }

  /// Returns `true` if the tree shaking is [`Module`].
  ///
  /// [`Module`]: TreeShaking::Module
  #[must_use]
  pub fn is_module(&self) -> bool {
    matches!(self, Self::Module)
  }
}

impl From<String> for TreeShaking {
  fn from(value: String) -> Self {
    match value.as_ref() {
      "true" => TreeShaking::True,
      "false" => TreeShaking::False,
      "module" => TreeShaking::Module,
      _ => panic!("Unknown tree shaking option, please use one of `true`, `false` and `module`"),
    }
  }
}

#[derive(Debug)]
pub struct TreeShakingPlugin {
  options: TreeShaking,
}

impl TreeShakingPlugin {
  pub fn new(options: TreeShaking) -> Self {
    Self { options }
  }
}

impl Plugin for TreeShakingPlugin {
  fn name(&self) -> &'static str {
    "rspack.TreeShakingPlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.tree_shaking = self.options.clone();
    Ok(())
  }
}

#[derive(Debug)]
pub struct NoEmitAssetsPlugin;

impl NoEmitAssetsPlugin {
  pub fn new() -> Self {
    Self
  }
}

impl Plugin for NoEmitAssetsPlugin {
  fn name(&self) -> &'static str {
    "rspack.NoEmitAssetsPlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.no_emit_assets = true;
    Ok(())
  }
}

#[derive(Debug, Clone, Default)]
pub struct Builtins {
  // TODO: migrate to builtin:swc-loader
  pub preset_env: Option<PresetEnv>,
  // TODO: refactor to string-replacement based
  pub define: Define,
  // TODO: refactor to string-replacement based
  pub provide: Provide,
  // TODO: refactoring
  pub tree_shaking: TreeShaking,
  // TODO: migrate to builtin:swc-loader
  pub react: ReactOptions,
  // TODO: migrate to builtin:swc-loader
  pub decorator: Option<DecoratorOptions>,
  // TODO: remove this when drop support for builtin options (0.6.0)
  pub no_emit_assets: bool,
  // TODO: migrate to builtin:swc-loader
  pub emotion: Option<swc_emotion::EmotionOptions>,
  // TODO: migrate to builtin:swc-loader
  pub plugin_import: Option<Vec<PluginImportConfig>>,
  // TODO: migrate to builtin:swc-loader
  pub relay: Option<RelayConfig>,
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

#[derive(Debug)]
pub struct PresetEnvPlugin {
  options: PresetEnv,
}

impl PresetEnvPlugin {
  pub fn new(options: PresetEnv) -> Self {
    Self { options }
  }
}

impl Plugin for PresetEnvPlugin {
  fn name(&self) -> &'static str {
    "rspack.PresetEnvPlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.preset_env = Some(self.options.clone());
    Ok(())
  }
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

#[derive(Debug)]
pub struct RelayPlugin {
  options: RelayConfig,
}

impl RelayPlugin {
  pub fn new(options: RelayConfig) -> Self {
    Self { options }
  }
}

impl Plugin for RelayPlugin {
  fn name(&self) -> &'static str {
    "rspack.RelayPlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.relay = Some(self.options.clone());
    Ok(())
  }
}

#[derive(Debug)]
pub struct EmotionPlugin {
  options: swc_emotion::EmotionOptions,
}

impl EmotionPlugin {
  pub fn new(options: swc_emotion::EmotionOptions) -> Self {
    Self { options }
  }
}

impl Plugin for EmotionPlugin {
  fn name(&self) -> &'static str {
    "rspack.EmotionPlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.emotion = Some(self.options.clone());
    Ok(())
  }
}

#[derive(Debug)]
pub struct PluginImportPlugin {
  options: Vec<PluginImportConfig>,
}

impl PluginImportPlugin {
  pub fn new(options: Vec<PluginImportConfig>) -> Self {
    Self { options }
  }
}

impl Plugin for PluginImportPlugin {
  fn name(&self) -> &'static str {
    "rspack.PluginImportPlugin"
  }

  fn apply(
    &self,
    _ctx: PluginContext<&mut ApplyContext>,
    options: &mut CompilerOptions,
  ) -> Result<()> {
    options.builtins.plugin_import = Some(self.options.clone());
    Ok(())
  }
}
