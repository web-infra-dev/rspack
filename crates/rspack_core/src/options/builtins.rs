use std::fmt::Debug;

use rspack_error::Result;
pub use rspack_swc_visitors::{Define, Provide};
use rspack_swc_visitors::{EmotionOptions, ImportOptions, ReactOptions, RelayOptions};

use crate::{ApplyContext, CompilerOptions, Plugin, PluginContext};

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
pub struct DecoratorOptions {
  // https://swc.rs/docs/configuration/compilation#jsctransformlegacydecorator
  pub legacy: bool,
  // https://swc.rs/docs/configuration/compilation#jsctransformdecoratormetadata
  pub emit_metadata: bool,
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

#[derive(Debug, Clone, Default)]
pub struct Builtins {
  // TODO: refactor to string-replacement based
  pub define: Define,
  // TODO: refactor to string-replacement based
  pub provide: Provide,
  // TODO: migrate to builtin:swc-loader
  pub preset_env: Option<PresetEnv>,
  // TODO: refactoring
  pub tree_shaking: TreeShaking,
  // TODO: migrate to builtin:swc-loader
  pub react: ReactOptions,
  // TODO: migrate to builtin:swc-loader
  pub decorator: Option<DecoratorOptions>,
  // TODO: remove this when drop support for builtin options (0.6.0)
  pub no_emit_assets: bool,
  // TODO: migrate to builtin:swc-loader
  pub emotion: Option<EmotionOptions>,
  // TODO: migrate to builtin:swc-loader
  pub plugin_import: Option<Vec<ImportOptions>>,
  // TODO: migrate to builtin:swc-loader
  pub relay: Option<RelayOptions>,
}

#[derive(Debug, Clone, Default)]
pub struct PresetEnv {
  pub targets: Vec<String>,
  pub mode: Option<swc_core::ecma::preset_env::Mode>,
  pub core_js: Option<String>,
}
