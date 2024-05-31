#![feature(let_chains)]

mod compiler;
mod options;
mod transformer;

use std::default::Default;

use compiler::{IntoJsAst, SwcCompiler};
use options::SwcCompilerOptionsWithAdditional;
pub use options::SwcLoaderJsOptions;
use rspack_core::{rspack_sources::SourceMap, LoaderRunnerContext, Mode};
use rspack_error::{error, AnyhowError, Diagnostic, Result};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_plugin_javascript::ast::{self, SourceMapConfig};
use rspack_plugin_javascript::TransformOutput;
use rspack_util::source_map::SourceMapKind;
use swc_config::{config_types::MergingOption, merge::Merge};
use swc_core::base::config::SourceMapsConfig;
use swc_core::base::config::{InputSourceMap, OutputCharset, TransformConfig};
use swc_core::ecma::visit::VisitWith;
use transformer::IdentCollector;

#[derive(Debug)]
pub struct SwcLoader {
  identifier: Identifier,
  options_with_additional: SwcCompilerOptionsWithAdditional,
}

impl SwcLoader {
  pub fn new(options: SwcLoaderJsOptions) -> Self {
    Self {
      identifier: SWC_LOADER_IDENTIFIER.into(),
      options_with_additional: options.into(),
    }
  }

  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:swc-loader`.
  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    assert!(identifier.starts_with(SWC_LOADER_IDENTIFIER));
    self.identifier = identifier;
    self
  }
}

pub const SWC_LOADER_IDENTIFIER: &str = "builtin:swc-loader";

#[async_trait::async_trait]
impl Loader<LoaderRunnerContext> for SwcLoader {
  async fn run(&self, loader_context: &mut LoaderContext<'_, LoaderRunnerContext>) -> Result<()> {
    let resource_path = loader_context.resource_path().to_path_buf();
    let content = std::mem::take(&mut loader_context.content).expect("content should be available");

    let swc_options = {
      let mut swc_options = self.options_with_additional.swc_options.clone();
      if swc_options.config.jsc.transform.as_ref().is_some() {
        let mut transform = TransformConfig::default();
        transform.react.development =
          Some(Mode::is_development(&loader_context.context.options.mode));
        swc_options
          .config
          .jsc
          .transform
          .merge(MergingOption::from(Some(transform)));
      }
      if let Some(pre_source_map) = loader_context.source_map.clone() {
        if let Ok(source_map) = pre_source_map.to_json() {
          swc_options.config.input_source_map = Some(InputSourceMap::Str(source_map))
        }
      }
      swc_options.filename = resource_path.to_string_lossy().to_string();
      swc_options.source_file_name = Some(resource_path.to_string_lossy().to_string());

      if swc_options.config.jsc.target.is_some() && swc_options.config.env.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          SWC_LOADER_IDENTIFIER.to_string(),
          "`env` and `jsc.target` cannot be used together".to_string(),
        ));
      }
      swc_options
    };

    let source_map_kind: SourceMapKind = match swc_options.config.source_maps {
      Some(SourceMapsConfig::Bool(false)) => SourceMapKind::empty(),
      _ => loader_context.context.module_source_map_kind,
    };

    let source = content.try_into_string()?;
    let c = SwcCompiler::new(resource_path.clone(), source.clone(), swc_options)
      .map_err(AnyhowError::from)?;

    let rspack_options = &*loader_context.context.options;
    let swc_options = c.options();
    let top_level_mark = swc_options
      .top_level_mark
      .expect("`top_level_mark` should be initialized");
    let unresolved_mark = swc_options
      .unresolved_mark
      .expect("`unresolved_mark` should be initialized");

    let built = c
      .parse(None, |_| {
        transformer::transform(
          &resource_path,
          rspack_options,
          Some(c.comments()),
          top_level_mark,
          unresolved_mark,
          c.cm().clone(),
          &source,
          &self.options_with_additional.rspack_experiments,
        )
      })
      .map_err(AnyhowError::from)?;

    let input_source_map = c
      .input_source_map(&built.input_source_map)
      .map_err(|e| error!(e.to_string()))?;
    let mut codegen_options = ast::CodegenOptions {
      target: Some(built.target),
      minify: Some(built.minify),
      input_source_map: input_source_map.as_ref(),
      ascii_only: built
        .output
        .charset
        .as_ref()
        .map(|v| matches!(v, OutputCharset::Ascii)),
      source_map_config: SourceMapConfig {
        enable: source_map_kind.source_map(),
        inline_sources_content: source_map_kind.source_map(),
        emit_columns: !source_map_kind.cheap(),
        names: Default::default(),
      },
      inline_script: Some(false),
      keep_comments: Some(true),
    };

    let program = tokio::task::block_in_place(|| c.transform(built).map_err(AnyhowError::from))?;
    if source_map_kind.enabled() {
      let mut v = IdentCollector {
        names: Default::default(),
      };
      program.visit_with(&mut v);
      codegen_options.source_map_config.names = v.names;
    }
    let ast = c.into_js_ast(program);
    let TransformOutput { code, map } = ast::stringify(&ast, codegen_options)?;

    loader_context.content = Some(code.into());
    let map = map
      .map(|m| SourceMap::from_json(&m))
      .transpose()
      .map_err(|e| error!(e.to_string()))?;
    loader_context.source_map = map;

    Ok(())
  }
}

impl Identifiable for SwcLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
