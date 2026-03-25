use std::path::Path;

use rspack_cacheable::{
  cacheable,
  with::{AsRefStr, AsRefStrConverter},
};
use rspack_swc_plugin_import::{ImportOptions, RawImportOptions};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use swc_config::{file_pattern::FilePattern, types::BoolConfig};
use swc_core::base::config::{
  Config, ErrorConfig, FileMatcher, InputSourceMap, IsModule, JscConfig, ModuleConfig, Options,
  SourceMapsConfig,
};

#[derive(Default, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct RawReactServerComponentsOptions {
  /// Whether to disable the compile-time check that reports errors when React
  /// client-only APIs (e.g. `useState`, `useEffect`) are imported in server
  /// components. Defaults to `false`.
  #[serde(default)]
  pub disable_client_api_checks: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RawReactServerComponents {
  Bool(bool),
  WithOptions(RawReactServerComponentsOptions),
}

impl Default for RawReactServerComponents {
  fn default() -> Self {
    RawReactServerComponents::Bool(false)
  }
}

#[derive(Default, Deserialize, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct RawRspackExperiments {
  pub import: Option<Vec<RawImportOptions>>,
  #[serde(default)]
  pub react_server_components: RawReactServerComponents,
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
  pub(crate) react_server_components: ReactServerComponentsOptions,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ReactServerComponentsOptions {
  pub(crate) enabled: bool,
  pub(crate) disable_client_api_checks: bool,
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
      react_server_components: match value.react_server_components {
        RawReactServerComponents::Bool(enabled) => ReactServerComponentsOptions {
          enabled,
          disable_client_api_checks: false,
        },
        RawReactServerComponents::WithOptions(opts) => ReactServerComponentsOptions {
          enabled: true,
          disable_client_api_checks: opts.disable_client_api_checks,
        },
      },
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum DetectSyntax {
  #[default]
  Disabled,
  Auto,
}

impl DetectSyntax {
  fn is_auto(self) -> bool {
    matches!(self, Self::Auto)
  }
}

impl<'de> Deserialize<'de> for DetectSyntax {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawDetectSyntax {
      Bool(bool),
      String(String),
    }

    match RawDetectSyntax::deserialize(deserializer)? {
      RawDetectSyntax::Bool(false) => Ok(DetectSyntax::Disabled),
      RawDetectSyntax::String(value) if value == "auto" => Ok(DetectSyntax::Auto),
      RawDetectSyntax::Bool(true) => Err(serde::de::Error::custom(
        "`detectSyntax` only supports `false` or \"auto\"",
      )),
      RawDetectSyntax::String(_) => Err(serde::de::Error::custom(
        "`detectSyntax` only supports `false` or \"auto\"",
      )),
    }
  }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct RawJscConfig {
  #[serde(default)]
  #[serde(rename = "parser")]
  raw_parser: Option<Value>,
  #[serde(flatten)]
  other_fields: Map<String, Value>,
}

impl RawJscConfig {
  fn has_explicit_syntax(&self) -> bool {
    matches!(
      &self.raw_parser,
      Some(Value::Object(parser)) if parser.contains_key("syntax")
    )
  }

  fn parse(&self) -> Result<JscConfig, serde_json::Error> {
    self.parse_with_resolved_parser(self.raw_parser.clone())
  }

  fn parse_for_resource(&self, resource_path: &Path) -> Result<JscConfig, serde_json::Error> {
    self.parse_with_resolved_parser(self.parser_for_resource(resource_path))
  }

  fn parse_with_resolved_parser(
    &self,
    parser: Option<Value>,
  ) -> Result<JscConfig, serde_json::Error> {
    let mut object: Map<String, Value> = self.other_fields.clone();
    if let Some(parser) = parser {
      object.insert("parser".into(), parser);
    }
    serde_json::from_value(Value::Object(object))
  }

  fn parser_for_resource(&self, resource_path: &Path) -> Option<Value> {
    let inferred_parser = inferred_parser_from_extension(resource_path);
    let Some(inferred_parser) = inferred_parser else {
      return self.raw_parser.clone();
    };

    match self.raw_parser.clone() {
      Some(Value::Object(parser)) if parser.contains_key("syntax") => Some(Value::Object(parser)),
      Some(Value::Object(parser)) => {
        let mut merged = inferred_parser;
        merged.extend(parser);
        Some(Value::Object(merged))
      }
      Some(parser) => Some(parser),
      None => Some(Value::Object(inferred_parser)),
    }
  }
}

fn inferred_parser_from_extension(resource_path: &Path) -> Option<Map<String, Value>> {
  let extension = resource_path.extension().and_then(|ext| ext.to_str())?;
  let mut parser = Map::default();

  match extension {
    "js" | "jsx" | "mjs" | "cjs" => {
      parser.insert("syntax".into(), Value::String("ecmascript".into()));
      parser.insert("jsx".into(), Value::Bool(true));
      Some(parser)
    }
    "ts" | "mts" | "cts" => {
      parser.insert("syntax".into(), Value::String("typescript".into()));
      parser.insert("tsx".into(), Value::Bool(false));
      Some(parser)
    }
    "tsx" => {
      parser.insert("syntax".into(), Value::String("typescript".into()));
      parser.insert("tsx".into(), Value::Bool(true));
      Some(parser)
    }
    _ => None,
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
  jsc: RawJscConfig,

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
  pub collect_type_script_info: Option<RawCollectTypeScriptInfoOptions>,

  #[serde(default)]
  pub transform_import: Option<Vec<RawImportOptions>>,

  #[serde(default)]
  pub rspack_experiments: Option<RawRspackExperiments>,

  #[serde(default)]
  detect_syntax: DetectSyntax,
}

#[cacheable(with=AsRefStr)]
#[derive(Debug)]
pub(crate) struct SwcCompilerOptionsWithAdditional {
  raw_options: String,
  pub(crate) swc_options: Options,
  pub(crate) resource_specific_jsc: Option<RawJscConfig>,
  pub(crate) transform_import: Option<Vec<ImportOptions>>,
  pub(crate) rspack_experiments: RspackExperiments,
  pub(crate) collect_typescript_info: Option<CollectTypeScriptInfoOptions>,
}

impl SwcCompilerOptionsWithAdditional {
  pub(crate) fn raw_options(&self) -> &str {
    &self.raw_options
  }

  pub(crate) fn parse_resource_specific_jsc(
    &self,
    resource_path: &Path,
  ) -> Result<Option<JscConfig>, serde_json::Error> {
    self
      .resource_specific_jsc
      .as_ref()
      .map(|jsc| jsc.parse_for_resource(resource_path))
      .transpose()
  }
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
      collect_type_script_info,
      transform_import,
      rspack_experiments,
      source_map_ignore_list,
      detect_syntax,
    } = option;

    let needs_resource_specific_jsc = detect_syntax.is_auto() && !jsc.has_explicit_syntax();
    let parsed_jsc = if needs_resource_specific_jsc {
      JscConfig::default()
    } else {
      jsc.parse()?
    };

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
          jsc: parsed_jsc,
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
        swcrc: false,
        ..serde_json::from_value(serde_json::Value::Object(Default::default()))?
      },
      resource_specific_jsc: needs_resource_specific_jsc.then_some(jsc),
      transform_import: transform_import.map(|i| i.into_iter().map(|v| v.into()).collect()),
      rspack_experiments: rspack_experiments.unwrap_or_default().into(),
      collect_typescript_info: collect_type_script_info.map(|v| v.into()),
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

    // We dont't want swc-loader in rspack to respect swcrc
    assert!(!swc_options_from_rspack.swcrc);
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

  #[test]
  fn test_detect_syntax_auto_merges_inferred_parser() {
    let raw_options = r#"{
      "detectSyntax": "auto",
      "jsc": {
        "parser": {
          "decorators": true
        }
      }
    }"#;
    let swc_options_with_additional: SwcCompilerOptionsWithAdditional = raw_options
      .try_into()
      .expect("Parse SwcCompilerOptionsWithAdditional from JSON string failed");

    let jsc = swc_options_with_additional
      .parse_resource_specific_jsc(Path::new("/project/index.tsx"))
      .expect("should resolve jsc config for tsx file")
      .expect("should require resource-specific jsc resolution");

    match jsc.syntax.expect("syntax should be inferred") {
      swc_core::ecma::parser::Syntax::Typescript(ts) => {
        assert!(ts.tsx);
        assert!(ts.decorators);
      }
      _ => panic!("expected typescript syntax"),
    }
  }

  #[test]
  fn test_detect_syntax_auto_preserves_explicit_parser_syntax() {
    let raw_options = r#"{
      "detectSyntax": "auto",
      "jsc": {
        "parser": {
          "syntax": "ecmascript",
          "jsx": false
        }
      }
    }"#;
    let swc_options_with_additional: SwcCompilerOptionsWithAdditional = raw_options
      .try_into()
      .expect("Parse SwcCompilerOptionsWithAdditional from JSON string failed");

    assert!(swc_options_with_additional.resource_specific_jsc.is_none());
    let jsc = &swc_options_with_additional.swc_options.config.jsc;

    match jsc.syntax.expect("syntax should remain explicit") {
      swc_core::ecma::parser::Syntax::Es(es) => {
        assert!(!es.jsx);
      }
      _ => panic!("expected ecmascript syntax"),
    }
  }

  #[test]
  fn test_preparse_jsc_when_resource_specific_resolution_is_not_needed() {
    let raw_options = r#"{
      "detectSyntax": false,
      "jsc": {
        "parser": {
          "syntax": "typescript",
          "tsx": true
        }
      }
    }"#;
    let swc_options_with_additional: SwcCompilerOptionsWithAdditional = raw_options
      .try_into()
      .expect("Parse SwcCompilerOptionsWithAdditional from JSON string failed");

    assert!(swc_options_with_additional.resource_specific_jsc.is_none());
    match swc_options_with_additional
      .swc_options
      .config
      .jsc
      .syntax
      .expect("syntax should be parsed at loader creation")
    {
      swc_core::ecma::parser::Syntax::Typescript(ts) => {
        assert!(ts.tsx);
      }
      _ => panic!("expected typescript syntax"),
    }
  }
}
