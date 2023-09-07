#![feature(let_chains)]

use std::default::Default;
use std::sync::Arc;

use options::{SwcCompilerOptionsWithAdditional, SwcLoaderJsOptions};
use rspack_core::{rspack_sources::SourceMap, LoaderRunnerContext, Mode};
use rspack_error::{internal_error, Diagnostic, Result};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use swc_config::config_types::MergingOption;
use swc_config::merge::Merge;
use swc_core::base::config::{InputSourceMap, TransformConfig};
use swc_core::base::{try_with_handler, Compiler};
use swc_core::common::comments::SingleThreadedComments;
use swc_core::common::{FileName, FilePathMapping, Mark, GLOBALS};
use swc_core::ecma::transforms::base::pass::noop;
use transformer::transform;
use xxhash_rust::xxh32::xxh32;

mod options;
mod transformer;

#[derive(Debug)]
pub struct SwcLoader {
  identifier: Identifier,
  options_with_additional: SwcCompilerOptionsWithAdditional,
}

impl SwcLoader {
  /// Panics:
  /// Panics if `identifier` passed in is not starting with `builtin:swc-loader`.
  pub fn new(options: SwcLoaderJsOptions, identifier: Option<Identifier>) -> Self {
    // TODO: should stringify loader options to identifier
    if let Some(i) = &identifier {
      assert!(i.starts_with(SWC_LOADER_IDENTIFIER));
    }
    Self {
      identifier: identifier.unwrap_or(SWC_LOADER_IDENTIFIER.into()),
      options_with_additional: options.into(),
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

    let c = Compiler::new(Arc::from(swc_core::common::SourceMap::new(
      FilePathMapping::empty(),
    )));
    let default_development = matches!(loader_context.context.options.mode, Mode::Development);
    let mut swc_options = self.options_with_additional.swc_options.clone();
    if swc_options.config.jsc.transform.as_ref().is_some() {
      let mut transform = TransformConfig::default();
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

    GLOBALS.set(&Default::default(), || {
      try_with_handler(c.cm.clone(), Default::default(), |handler| {
        c.run(|| {
          let top_level_mark = Mark::new();
          let unresolved_mark = Mark::new();
          swc_options.top_level_mark = Some(top_level_mark);
          swc_options.unresolved_mark = Some(unresolved_mark);
          let source = content.try_into_string()?;
          let rspack_options = &*loader_context.context.options;
          let source_content_hash = rspack_options
            .builtins
            .emotion
            .as_ref()
            .map(|_| xxh32(source.as_bytes(), 0));

          let fm = c
            .cm
            .new_source_file(FileName::Real(resource_path.clone()), source);
          let comments = SingleThreadedComments::default();

          let out = c.process_js_with_custom_pass(
            fm,
            None,
            handler,
            &swc_options,
            comments,
            |_| noop(),
            |_| {
              transform(
                &resource_path,
                rspack_options,
                Some(c.comments()),
                top_level_mark,
                unresolved_mark,
                c.cm.clone(),
                source_content_hash,
                &self.options_with_additional.rspack_experiments,
              )
            },
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
