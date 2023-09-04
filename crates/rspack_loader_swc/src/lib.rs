#![feature(let_chains)]

use std::default::Default;
use std::sync::Arc;

use rspack_core::{rspack_sources::SourceMap, LoaderRunnerContext, Mode};
use rspack_error::{internal_error, Diagnostic, Result};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use serde::Deserialize;
use swc_config::config_types::{BoolConfig, MergingOption};
use swc_config::merge::Merge;
use swc_core::base::config::{
  Config, ErrorConfig, FileMatcher, InputSourceMap, IsModule, JscConfig, ModuleConfig, Options,
  SourceMapsConfig, TransformConfig,
};
use swc_core::base::{try_with_handler, Compiler};
use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::{FileName, FilePathMapping, GLOBALS};
use swc_core::ecma::transforms::base::pass::noop;

pub const SOURCE_MAP_INLINE: &str = "inline";

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
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct SwcLoaderOptions {
  pub config: Option<Config>,
  pub source_maps: Option<SourceMapsConfig>,
}

impl From<SwcLoaderJsOptions> for Options {
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
    Options {
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
    }
  }
}

#[derive(Debug)]
pub struct SwcLoader {
  options: Options,
  identifier: Identifier,
}

impl SwcLoader {
  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:swc-loader`.
  pub fn new(options: SwcLoaderJsOptions, identifier: Option<Identifier>) -> Self {
    // TODO: should stringify loader options to identifier
    Self::validate_identifier(&identifier);
    Self {
      options: Options::from(options),
      identifier: identifier.unwrap_or(SWC_LOADER_IDENTIFIER.into()),
    }
  }

  fn validate_identifier(identifier: &Option<Identifier>) {
    if let Some(i) = identifier {
      assert!(i.starts_with(SWC_LOADER_IDENTIFIER));
    }
  }
}

pub const SWC_LOADER_IDENTIFIER: &str = "builtin:swc-loader";

#[async_trait::async_trait]
impl Loader<LoaderRunnerContext> for SwcLoader {
  async fn run(&self, loader_context: &mut LoaderContext<'_, LoaderRunnerContext>) -> Result<()> {
    let resource_path = loader_context.resource_path.to_path_buf();
    let Some(content) = std::mem::take(&mut loader_context.content) else {
      return Err(internal_error!("Content should be available"))
    };

    let c: Compiler = Compiler::new(Arc::from(swc_core::common::SourceMap::new(
      FilePathMapping::empty(),
    )));
    let default_development = matches!(loader_context.context.options.mode, Mode::Development);
    let mut options = self.options.clone();
    if options.config.jsc.transform.as_ref().is_some() {
      let mut transform = TransformConfig::default();
      transform.react.development = Some(default_development);
      options
        .config
        .jsc
        .transform
        .merge(MergingOption::from(Some(transform)));
    }
    if let Some(pre_source_map) = std::mem::take(&mut loader_context.source_map) {
      if let Ok(source_map) = pre_source_map.to_json() {
        options.config.input_source_map = Some(InputSourceMap::Str(source_map))
      }
    }

    if options.config.jsc.experimental.plugins.is_some() {
      loader_context.emit_diagnostic(Diagnostic::warn(
        SWC_LOADER_IDENTIFIER.to_string(),
        "Experimental plugins are not currently supported.".to_string(),
        0,
        0,
      ));
    }

    GLOBALS.set(&Default::default(), || {
      try_with_handler(c.cm.clone(), Default::default(), |handler| {
        c.run(|| {
          let fm = c
            .cm
            .new_source_file(FileName::Real(resource_path), content.try_into_string()?);
          let comments = SingleThreadedComments::default();

          let out = c.process_js_with_custom_pass(
            fm,
            None,
            handler,
            &options,
            comments,
            |_a| noop(),
            |_a| noop(),
          )?;
          loader_context.content = Some(out.code.into());
          loader_context.source_map = out.map.map(|m| SourceMap::from_json(&m)).transpose()?;

          Ok(())
        })
      })
    })?;

    Ok(())
  }
}

impl Identifiable for SwcLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
