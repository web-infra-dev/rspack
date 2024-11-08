#![feature(let_chains)]

mod compiler;
mod options;
mod transformer;

use std::default::Default;
use std::sync::Arc;

use compiler::{IntoJsAst, SwcCompiler};
use options::SwcCompilerOptionsWithAdditional;
pub use options::SwcLoaderJsOptions;
use rspack_core::{rspack_sources::SourceMap, Mode, RunnerContext};
use rspack_error::{error, AnyhowError, Diagnostic, Result};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use rspack_paths::Utf8PathBuf;
use rspack_plugin_emit_dts::SwcDtsEmitOptions;
use rspack_plugin_javascript::ast::{self, SourceMapConfig};
use rspack_plugin_javascript::TransformOutput;
use rspack_util::source_map::SourceMapKind;
use swc_config::{config_types::MergingOption, merge::Merge};
use swc_core::base::config::SourceMapsConfig;
use swc_core::base::config::{InputSourceMap, OutputCharset, TransformConfig};
use swc_core::ecma::codegen::to_code_with_comments;
use swc_core::ecma::visit::VisitWith;
use swc_typescript::fast_dts::FastDts;
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

  fn loader_impl(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let resource_path = loader_context
      .resource_path()
      .map(|p| p.to_path_buf())
      .unwrap_or_default();

    let filename = resource_path.as_str().to_string();
    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };

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
      if let Some(pre_source_map) = loader_context.source_map().cloned() {
        if let Ok(source_map) = pre_source_map.to_json() {
          swc_options.config.input_source_map = Some(InputSourceMap::Str(source_map))
        }
      }
      swc_options.filename = filename.clone();
      swc_options.source_file_name = Some(filename.clone());

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

    let source = content.into_string_lossy();
    let c = SwcCompiler::new(
      resource_path.into_std_path_buf(),
      source.clone(),
      swc_options,
    )
    .map_err(AnyhowError::from)?;

    let built = c
      .parse(None, |_| {
        transformer::transform(&self.options_with_additional.rspack_experiments)
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

    // let emit_dts = built.syntax.typescript() && built.emit_isolated_dts;
    let emit_dts = built.syntax.typescript()
      && self
        .options_with_additional
        .rspack_experiments
        .emit_dts
        .is_some();

    let program = &built.program;

    if emit_dts && program.is_module() {
      let mut checker = FastDts::new(Arc::new(swc_core::common::FileName::Custom(
        filename.clone(),
      )));
      let SwcDtsEmitOptions {
        abort_on_error,
        include: _,
        out_dir,
        root_dir,
        emit,
      } = self
        .options_with_additional
        .rspack_experiments
        .emit_dts
        .as_ref()
        .expect("never reach");

      let root_dir = Utf8PathBuf::from(root_dir);
      let dts_filename: Option<Utf8PathBuf> = {
        let filename = Utf8PathBuf::from(&filename);
        if let Ok(output_relative_path) = filename.strip_prefix(root_dir) {
          let output_filename = Utf8PathBuf::from(out_dir.clone()).join(output_relative_path);
          Some(output_filename)
        } else {
          None
        }
      };

      if let Some(dts_filename) = dts_filename {
        let mut program = program.clone();
        let issues = checker.transform(&mut program);
        let should_abort = *abort_on_error && !issues.is_empty();

        if should_abort {
          let error: Vec<String> = issues.iter().map(|e| e.to_string()).collect();
          let error = error.concat();
          return Err(error!(
            "Failed to generate dts code in {}, {}",
            SWC_LOADER_IDENTIFIER.to_string(),
            error
          ));
        } else {
          issues.into_iter().for_each(|issue| {
            loader_context.emit_diagnostic(Diagnostic::error(
              SWC_LOADER_IDENTIFIER.to_string(),
              issue.to_string(),
            ))
          });
        }

        if *emit {
          let module = program.expect_module();
          let dts_code = to_code_with_comments(Some(&built.comments), &module);
          loader_context
            .parse_meta
            .entry("swc-dts-emit-plugin-filename".to_string())
            .and_modify(|v| *v = filename.clone())
            .or_insert(filename.clone());
          loader_context
            .parse_meta
            .entry("swc-dts-emit-plugin-dts-filename".to_string())
            .and_modify(|v| *v = dts_filename.to_string())
            .or_insert(dts_filename.to_string());
          loader_context
            .parse_meta
            .entry("swc-dts-emit-plugin-dts-code".to_string())
            .and_modify(|v| v.push_str(&dts_code))
            .or_insert(dts_code);
        }
      }
    }

    let program = c.transform(built).map_err(AnyhowError::from)?;
    if source_map_kind.enabled() {
      let mut v = IdentCollector {
        names: Default::default(),
      };
      program.visit_with(&mut v);
      codegen_options.source_map_config.names = v.names;
    }

    let ast = c.into_js_ast(program);
    let TransformOutput { code, map } = ast::stringify(&ast, codegen_options)?;

    let map = map
      .map(|m| SourceMap::from_json(&m))
      .transpose()
      .map_err(|e| error!(e.to_string()))?;
    loader_context.finish_with((code, map));

    Ok(())
  }
}

pub const SWC_LOADER_IDENTIFIER: &str = "builtin:swc-loader";

#[async_trait::async_trait]
impl Loader<RunnerContext> for SwcLoader {
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    #[allow(unused_mut)]
    let mut inner = || self.loader_impl(loader_context);
    #[cfg(debug_assertions)]
    {
      // Adjust stack to avoid stack overflow.
      stacker::maybe_grow(
        2 * 1024 * 1024, /* 2mb */
        4 * 1024 * 1024, /* 4mb */
        inner,
      )
    }
    #[cfg(not(debug_assertions))]
    inner()
  }
}

impl Identifiable for SwcLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
