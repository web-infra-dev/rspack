use napi_derive::napi;
use serde::Deserialize;
use swc_plugin_import::{CustomTransform, PluginImportConfig, StyleConfig};

#[derive(Debug, Deserialize)]
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

impl From<RawPluginImportConfig> for PluginImportConfig {
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
