use rspack_swc_visitors::{
  EmotionOptions, ImportOptions, PreactOptions, RawEmotionOptions, RawImportOptions,
  RawPreactOptions, RawRelayOptions, RawStyledComponentsOptions, RelayOptions,
  StyledComponentsOptions,
};
use serde::Deserialize;
use swc_config::config_types::BoolConfig;
use swc_core::base::config::{
  Config, ErrorConfig, FileMatcher, InputSourceMap, IsModule, JscConfig, ModuleConfig, Options,
  SourceMapsConfig,
};

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct RawRspackExperiments {
  pub relay: Option<RawRelayOptions>,
  pub styled_components: Option<RawStyledComponentsOptions>,
  pub import: Option<Vec<RawImportOptions>>,
  pub emotion: Option<RawEmotionOptions>,
  pub preact: Option<RawPreactOptions>,
}

#[derive(Default, Debug)]
pub(crate) struct RspackExperiments {
  pub(crate) relay: Option<RelayOptions>,
  pub(crate) styled_components: Option<StyledComponentsOptions>,
  pub(crate) import: Option<Vec<ImportOptions>>,
  pub(crate) emotion: Option<EmotionOptions>,
  pub(crate) preact: Option<PreactOptions>,
}

impl From<RawRspackExperiments> for RspackExperiments {
  fn from(value: RawRspackExperiments) -> Self {
    Self {
      relay: value.relay.map(|v| v.into()),
      styled_components: value.styled_components.map(|v| v.into()),
      import: value
        .import
        .map(|i| i.into_iter().map(|v| v.into()).collect()),
      emotion: value.emotion,
      preact: value.preact,
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
  pub rspack_experiments: Option<RawRspackExperiments>,
}

#[derive(Debug)]
pub(crate) struct SwcCompilerOptionsWithAdditional {
  pub(crate) swc_options: Options,
  pub(crate) rspack_experiments: RspackExperiments,
}

const SOURCE_MAP_INLINE: &str = "inline";

impl From<SwcLoaderJsOptions> for SwcCompilerOptionsWithAdditional {
  fn from(value: SwcLoaderJsOptions) -> Self {
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
    } = value;
    let mut source_maps: Option<SourceMapsConfig> = source_maps;
    if source_maps.is_none() && source_map.is_some() {
      source_maps = source_map
    }
    if let Some(SourceMapsConfig::Str(str)) = &source_maps {
      if str == SOURCE_MAP_INLINE {
        source_maps = Some(SourceMapsConfig::Bool(true))
      }
    }
    SwcCompilerOptionsWithAdditional {
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
        },
        ..Default::default()
      },
      rspack_experiments: rspack_experiments.unwrap_or_default().into(),
    }
  }
}
