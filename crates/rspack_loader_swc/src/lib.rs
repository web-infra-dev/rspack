#![feature(let_chains)]

use std::default::Default;

use rspack_ast::RspackAst;
use rspack_core::{rspack_sources::SourceMap, LoaderRunnerContext, Mode};
use rspack_error::{internal_error, Diagnostic, Result};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_plugin_javascript::{
  ast::{self, SourceMapConfig},
  TransformOutput,
};
use swc_config::{config_types::MergingOption, merge::Merge};
use swc_core::base::config::{InputSourceMap, OutputCharset, TransformConfig};

mod compiler;
mod options;
mod transformer;

use compiler::{IntoJsAst, SwcCompiler};
use options::SwcCompilerOptionsWithAdditional;
pub use options::SwcLoaderJsOptions;

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
    let resource_path = loader_context.resource_path.to_path_buf();
    let Some(content) = std::mem::take(&mut loader_context.content) else {
      return Err(internal_error!("Content should be available"))
    };

    let swc_options = {
      let mut swc_options = self.options_with_additional.swc_options.clone();
      if swc_options.config.jsc.transform.as_ref().is_some() {
        let mut transform = TransformConfig::default();
        let default_development = matches!(loader_context.context.options.mode, Mode::Development);
        transform.react.development = Some(default_development);
        swc_options
          .config
          .jsc
          .transform
          .merge(MergingOption::from(Some(transform)));
      }
      if let Some(pre_source_map) = std::mem::take(&mut loader_context.source_map) {
        if let Ok(source_map) = pre_source_map.to_json() {
          swc_options.config.input_source_map = Some(InputSourceMap::Str(source_map))
        }
      }

      if swc_options.config.jsc.experimental.plugins.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          SWC_LOADER_IDENTIFIER.to_string(),
          "Experimental plugins are not currently supported.".to_string(),
          0,
          0,
        ));
      }

      if swc_options.config.jsc.target.is_some() && swc_options.config.env.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          SWC_LOADER_IDENTIFIER.to_string(),
          "`env` and `jsc.target` cannot be used together".to_string(),
          0,
          0,
        ));
      }
      swc_options
    };

    let devtool = &loader_context.context.options.devtool;
    let source = content.try_into_string()?;
    let c = SwcCompiler::new(resource_path.clone(), source.clone(), swc_options)?;

    let rspack_options = &*loader_context.context.options;
    let swc_options = c.options();
    let top_level_mark = swc_options
      .top_level_mark
      .expect("`top_level_mark` should be initialized");
    let unresolved_mark = swc_options
      .unresolved_mark
      .expect("`unresolved_mark` should be initialized");

    let built = c.parse(None, |_| {
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
    })?;

    let codegen_options = ast::CodegenOptions {
      target: Some(built.target),
      minify: Some(built.minify),
      ascii_only: built
        .output
        .charset
        .as_ref()
        .map(|v| matches!(v, OutputCharset::Ascii)),
      source_map_config: SourceMapConfig {
        enable: devtool.source_map(),
        inline_sources_content: true,
        emit_columns: !devtool.cheap(),
        names: Default::default(),
      },
      keep_comments: Some(true),
    };
    let program = c.transform(built)?;
    let ast = c.into_js_ast(program);

    // If swc-loader is the latest loader available,
    // then loader produces AST, which could be used as an optimization.
    if loader_context.loader_index() == 0
      && (loader_context
        .current_loader()
        .composed_index_by_identifier(&self.identifier)
        .map(|idx| idx == 0)
        .unwrap_or(true))
    {
      loader_context
        .additional_data
        .insert(RspackAst::JavaScript(ast));
      loader_context.additional_data.insert(codegen_options);
      loader_context.content = Some("".to_owned().into())
    } else {
      let TransformOutput { code, map } = ast::stringify(&ast, codegen_options)?;
      loader_context.content = Some(code.into());
      loader_context.source_map = map.map(|m| SourceMap::from_json(&m)).transpose()?;
    }

    Ok(())
  }
}

impl Identifiable for SwcLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
