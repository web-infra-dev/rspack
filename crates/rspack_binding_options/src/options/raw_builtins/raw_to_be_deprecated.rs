use std::{path::PathBuf, str::FromStr};

use napi_derive::napi;
use rspack_core::{Builtins, DecoratorOptions, PluginExt, PresetEnv};
use rspack_error::internal_error;
use rspack_plugin_css::{
  plugin::{CssConfig, LocalIdentName, LocalsConvention, ModulesConfig},
  CssPlugin,
};
use rspack_plugin_dev_friendly_split_chunks::DevFriendlySplitChunksPlugin;
use rspack_swc_visitors::{
  CustomTransform, ImportOptions, ReactOptions, RelayLanguageConfig, RelayOptions, StyleConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawDecoratorOptions {
  pub legacy: bool,
  pub emit_metadata: bool,
}

impl From<RawDecoratorOptions> for DecoratorOptions {
  fn from(value: RawDecoratorOptions) -> Self {
    Self {
      legacy: value.legacy,
      emit_metadata: value.emit_metadata,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawStyleConfig {
  pub style_library_directory: Option<String>,
  pub custom: Option<String>,
  pub css: Option<String>,
  pub bool: Option<bool>,
}

impl From<RawStyleConfig> for StyleConfig {
  fn from(raw_style_config: RawStyleConfig) -> Self {
    if let Some(style_library_directory) = raw_style_config.style_library_directory {
      Self::StyleLibraryDirectory(style_library_directory)
    } else if let Some(custom) = raw_style_config.custom {
      Self::Custom(CustomTransform::Tpl(custom))
    } else if raw_style_config.css.is_some() {
      Self::Css
    } else if let Some(bool) = raw_style_config.bool {
      Self::Bool(bool)
    } else {
      Self::None
    }
  }
}

#[derive(Debug, Deserialize)]
#[napi(object)]
pub struct RawPluginImportConfig {
  pub library_name: String,
  pub library_directory: Option<String>, // default to `lib`
  pub custom_name: Option<String>,
  pub custom_style_name: Option<String>, // If this is set, `style` option will be ignored
  pub style: Option<RawStyleConfig>,
  pub camel_to_dash_component_name: Option<bool>, // default to true
  pub transform_to_default_import: Option<bool>,
  pub ignore_es_component: Option<Vec<String>>,
  pub ignore_style_component: Option<Vec<String>>,
}

impl From<RawPluginImportConfig> for ImportOptions {
  fn from(plugin_import: RawPluginImportConfig) -> Self {
    let RawPluginImportConfig {
      library_name,
      library_directory,
      custom_name,
      custom_style_name,
      style,
      camel_to_dash_component_name,
      transform_to_default_import,
      ignore_es_component,
      ignore_style_component,
    } = plugin_import;

    Self {
      library_name,
      library_directory,
      custom_name: custom_name.map(CustomTransform::Tpl),
      custom_style_name: custom_style_name.map(CustomTransform::Tpl),
      style: style.map(Into::into),
      camel_to_dash_component_name,
      transform_to_default_import,
      ignore_es_component,
      ignore_style_component,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawPresetEnv {
  pub targets: Vec<String>,
  #[napi(ts_type = "'usage' | 'entry'")]
  pub mode: Option<String>,
  pub core_js: Option<String>,
}

impl From<RawPresetEnv> for PresetEnv {
  fn from(raw_preset_env: RawPresetEnv) -> Self {
    Self {
      targets: raw_preset_env.targets,
      mode: raw_preset_env.mode.and_then(|mode| match mode.as_str() {
        "usage" => Some(swc_core::ecma::preset_env::Mode::Usage),
        "entry" => Some(swc_core::ecma::preset_env::Mode::Entry),
        _ => None,
      }),
      core_js: raw_preset_env.core_js,
    }
  }
}

use swc_core::ecma::transforms::react::Runtime;

use crate::RawOptionsApply;

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawReactOptions {
  #[napi(ts_type = "\"automatic\" | \"classic\"")]
  pub runtime: Option<String>,
  pub import_source: Option<String>,
  pub pragma: Option<String>,
  pub pragma_frag: Option<String>,
  pub throw_if_namespace: Option<bool>,
  pub development: Option<bool>,
  pub use_builtins: Option<bool>,
  pub use_spread: Option<bool>,
  pub refresh: Option<bool>,
}

impl From<RawReactOptions> for ReactOptions {
  fn from(value: RawReactOptions) -> Self {
    let runtime = if let Some(runtime) = &value.runtime {
      match runtime.as_str() {
        "automatic" => Some(Runtime::Automatic),
        "classic" => Some(Runtime::Classic),
        _ => None,
      }
    } else {
      Some(Runtime::Automatic)
    };

    Self {
      runtime,
      import_source: value.import_source,
      pragma: value.pragma,
      pragma_frag: value.pragma_frag,
      throw_if_namespace: value.throw_if_namespace,
      development: value.development,
      use_builtins: value.use_builtins,
      use_spread: value.use_spread,
      refresh: value.refresh,
    }
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawRelayConfig {
  pub artifact_directory: Option<String>,
  #[napi(ts_type = "'javascript' | 'typescript' | 'flow'")]
  pub language: String,
}

impl From<RawRelayConfig> for RelayOptions {
  fn from(raw_config: RawRelayConfig) -> Self {
    Self {
      artifact_directory: raw_config.artifact_directory.map(PathBuf::from),
      language: match raw_config.language.as_str() {
        "typescript" => RelayLanguageConfig::TypeScript,
        "flow" => RelayLanguageConfig::Flow,
        _ => RelayLanguageConfig::JavaScript,
      },
    }
  }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCssPluginConfig {
  pub modules: RawCssModulesConfig,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCssModulesConfig {
  #[napi(ts_type = "\"asIs\" | \"camelCase\" | \"camelCaseOnly\" | \"dashes\" | \"dashesOnly\"")]
  pub locals_convention: String,
  pub local_ident_name: String,
  pub exports_only: bool,
}

impl TryFrom<RawCssModulesConfig> for ModulesConfig {
  type Error = rspack_error::Error;

  fn try_from(value: RawCssModulesConfig) -> Result<Self, Self::Error> {
    Ok(Self {
      locals_convention: LocalsConvention::from_str(&value.locals_convention)?,
      local_ident_name: LocalIdentName::from(value.local_ident_name),
      exports_only: value.exports_only,
    })
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawBuiltins {
  pub css: Option<RawCssPluginConfig>,
  pub preset_env: Option<RawPresetEnv>,
  pub tree_shaking: String,
  pub react: RawReactOptions,
  pub decorator: Option<RawDecoratorOptions>,
  pub no_emit_assets: bool,
  pub emotion: Option<String>,
  pub dev_friendly_split_chunks: bool,
  pub plugin_import: Option<Vec<RawPluginImportConfig>>,
  pub relay: Option<RawRelayConfig>,
}

impl RawOptionsApply for RawBuiltins {
  type Options = Builtins;

  fn apply(
    self,
    plugins: &mut Vec<rspack_core::BoxPlugin>,
  ) -> Result<Self::Options, rspack_error::Error> {
    if let Some(css) = self.css {
      let options = CssConfig {
        modules: css.modules.try_into()?,
      };
      plugins.push(CssPlugin::new(options).boxed());
    }
    if self.dev_friendly_split_chunks {
      plugins.push(DevFriendlySplitChunksPlugin::new().boxed());
    }

    Ok(Builtins {
      define: Default::default(),
      provide: Default::default(),
      preset_env: self.preset_env.map(Into::into),
      tree_shaking: self.tree_shaking.into(),
      react: self.react.into(),
      decorator: self.decorator.map(|i| i.into()),
      no_emit_assets: self.no_emit_assets,
      emotion: self
        .emotion
        .map(|i| serde_json::from_str(&i))
        .transpose()
        .map_err(|e| internal_error!(e.to_string()))?,
      plugin_import: self
        .plugin_import
        .map(|plugin_imports| plugin_imports.into_iter().map(Into::into).collect()),
      relay: self.relay.map(Into::into),
    })
  }
}
