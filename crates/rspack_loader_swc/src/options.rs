use rspack_cacheable::{
  cacheable,
  with::{AsRefStr, AsRefStrConverter},
};
use rspack_swc_plugin_import::{ImportOptions, RawImportOptions};
use serde::Deserialize;
use swc_config::{file_pattern::FilePattern, types::BoolConfig};
use swc_core::base::config::{
  Config, ErrorConfig, FileMatcher, InputSourceMap, IsModule, JscConfig, ModuleConfig, Options,
  SourceMapsConfig,
};

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct RawRspackExperiments {
  pub import: Option<Vec<RawImportOptions>>,
  pub collect_type_script_info: Option<RawCollectTypeScriptInfoOptions>,
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct RawCollectTypeScriptInfoOptions {
  pub type_exports: Option<bool>,
  pub exported_enum: Option<String>,
}

#[derive(Default, Debug)]
pub(crate) struct RspackExperiments {
  pub(crate) import: Option<Vec<ImportOptions>>,
  pub(crate) collect_typescript_info: Option<CollectTypeScriptInfoOptions>,
}

#[derive(Default, Debug)]
pub(crate) struct CollectTypeScriptInfoOptions {
  pub(crate) type_exports: Option<bool>,
  pub(crate) exported_enum: Option<CollectingEnumKind>,
}

#[derive(Default, Debug)]
pub(crate) enum CollectingEnumKind {
  All,
  #[default]
  ConstOnly,
}

impl From<RawRspackExperiments> for RspackExperiments {
  fn from(value: RawRspackExperiments) -> Self {
    Self {
      import: value
        .import
        .map(|i| i.into_iter().map(|v| v.into()).collect()),
      collect_typescript_info: value.collect_type_script_info.map(|v| v.into()),
    }
  }
}

impl From<RawCollectTypeScriptInfoOptions> for CollectTypeScriptInfoOptions {
  fn from(value: RawCollectTypeScriptInfoOptions) -> Self {
    Self {
      type_exports: value.type_exports,
      exported_enum: value.exported_enum.and_then(|v| match v.as_str() {
        "const-only" => Some(CollectingEnumKind::ConstOnly),
        "all" => Some(CollectingEnumKind::All),
        _ => None,
      }),
    }
  }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SwcLoaderJsOptions {
  #[serde(default)]
  pub source_maps: Option<SourceMapsConfig>,

  pub source_map: Option<SourceMapsConfig>,
  #[serde(default)]
  pub env: Option<swc_core::ecma::preset_env::Config>,

  #[serde(default)]
  pub test: Option<FileMatcher>,

  #[serde(default)]
  pub exclude: Option<FileMatcher>,

  #[serde(default)]
  pub jsc: JscConfig,

  #[serde(default)]
  pub module: Option<ModuleConfig>,

  #[serde(default)]
  pub minify: BoolConfig<false>,

  #[serde(default)]
  pub input_source_map: Option<InputSourceMap>,

  #[serde(default)]
  pub inline_sources_content: BoolConfig<true>,

  #[serde(default)]
  pub emit_source_map_columns: BoolConfig<true>,

  #[serde(default)]
  pub error: ErrorConfig,

  #[serde(default)]
  pub is_module: Option<IsModule>,

  #[serde(rename = "$schema")]
  pub schema: Option<String>,

  #[serde(default)]
  pub source_map_ignore_list: Option<FilePattern>,

  #[serde(default)]
  pub rspack_experiments: Option<RawRspackExperiments>,
}

#[cacheable(with=AsRefStr)]
#[derive(Debug)]
pub(crate) struct SwcCompilerOptionsWithAdditional {
  raw_options: String,
  pub(crate) swc_options: Options,
  pub(crate) rspack_experiments: RspackExperiments,
}

impl AsRefStrConverter for SwcCompilerOptionsWithAdditional {
  fn as_str(&self) -> &str {
    &self.raw_options
  }
  fn from_str(s: &str) -> Self {
    s.try_into()
      .expect("failed to generate SwcCompilerOptionsWithAdditional")
  }
}

const SOURCE_MAP_INLINE: &str = "inline";

impl TryFrom<&str> for SwcCompilerOptionsWithAdditional {
  type Error = serde_json::Error;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let option: SwcLoaderJsOptions = serde_json::from_str(value)?;
    let SwcLoaderJsOptions {
      source_maps,
      source_map,
      env,
      test,
      exclude,
      jsc,
      module,
      minify,
      input_source_map,
      inline_sources_content,
      emit_source_map_columns,
      error,
      is_module,
      schema,
      rspack_experiments,
      source_map_ignore_list,
    } = option;
    let mut source_maps: Option<SourceMapsConfig> = source_maps;
    if source_maps.is_none() && source_map.is_some() {
      source_maps = source_map
    }
    if let Some(SourceMapsConfig::Str(str)) = &source_maps
      && str == SOURCE_MAP_INLINE
    {
      source_maps = Some(SourceMapsConfig::Bool(true))
    }
    Ok(SwcCompilerOptionsWithAdditional {
      raw_options: value.into(),
      swc_options: Options {
        config: Config {
          env,
          test,
          exclude,
          jsc,
          module,
          minify,
          input_source_map,
          source_maps,
          inline_sources_content,
          emit_source_map_columns,
          error,
          is_module,
          schema,
          source_map_ignore_list,
        },
        ..serde_json::from_value(serde_json::Value::Object(Default::default()))?
      },
      rspack_experiments: rspack_experiments.unwrap_or_default().into(),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_specially_default_values_in_swc_options() {
    // Verifies that fields using `#[serde(default = "...")]`
    // receive the correct default value from the initializations of
    // both SwcCompilerOptionsWithAdditional and swc_core::base::config::Options.
    let raw_options = "{}";
    let swc_options_with_additional: SwcCompilerOptionsWithAdditional = raw_options
      .try_into()
      .expect("Parse SwcCompilerOptionsWithAdditional from empty JSON string failed");

    let swc_options_from_rspack: Options = swc_options_with_additional.swc_options;

    let swc_options_from_native_lib: Options =
      serde_json::from_value(serde_json::from_str(raw_options).unwrap()).unwrap();

    assert_eq!(
      swc_options_from_rspack.env_name,
      swc_options_from_native_lib.env_name
    );
    assert_eq!(swc_options_from_rspack.cwd, swc_options_from_native_lib.cwd);
    assert_eq!(
      swc_options_from_rspack.swcrc,
      swc_options_from_native_lib.swcrc
    );
  }

  #[test]
  fn test_swc_loader_js_options_ignore_unexpected_field() {
    let raw_options = "{ \"envName\": \"my-env\" }";
    let swc_options_with_additional: SwcCompilerOptionsWithAdditional = raw_options
      .try_into()
      .expect("Parse SwcCompilerOptionsWithAdditional from JSON string failed");

    let swc_options_from_rspack: Options = swc_options_with_additional.swc_options;

    let swc_options_from_native_lib: Options =
      serde_json::from_value(serde_json::from_str(raw_options).unwrap()).unwrap();

    assert_eq!(
      swc_options_from_native_lib.env_name, "my-env",
      "native options should parse envName from JSON"
    );
    assert_ne!(
      swc_options_from_rspack.env_name, "my-env",
      "SwcCompilerOptionsWithAdditional should ignore unexpected field envName"
    );
    assert!(
      !swc_options_from_rspack.env_name.is_empty(),
      "SwcCompilerOptionsWithAdditional should receive a default value of env_name"
    )
  }
}
