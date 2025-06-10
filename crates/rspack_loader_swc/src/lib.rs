#![feature(let_chains)]

mod options;
mod plugin;
mod transformer;

use std::default::Default;

use options::SwcCompilerOptionsWithAdditional;
pub use options::SwcLoaderJsOptions;
pub use plugin::SwcLoaderPlugin;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{Mode, RunnerContext};
use rspack_error::{miette, Diagnostic, Result};
use rspack_javascript_compiler::{JavaScriptCompiler, TransformOutput};
use rspack_loader_runner::{Identifiable, Identifier, Loader, LoaderContext};
use swc_config::{merge::Merge, types::MergingOption};
use swc_core::{
  base::config::{InputSourceMap, TransformConfig},
  common::FileName,
};

#[cacheable]
#[derive(Debug)]
pub struct SwcLoader {
  identifier: Identifier,
  options_with_additional: SwcCompilerOptionsWithAdditional,
}

impl SwcLoader {
  pub fn new(raw_options: &str) -> Result<Self, serde_json::Error> {
    Ok(Self {
      identifier: SWC_LOADER_IDENTIFIER.into(),
      options_with_additional: raw_options.try_into()?,
    })
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
      swc_options.filename = resource_path.as_str().to_string();
      swc_options.source_file_name = Some(resource_path.as_str().to_string());

      if swc_options.config.jsc.target.is_some() && swc_options.config.env.is_some() {
        loader_context.emit_diagnostic(Diagnostic::warn(
          SWC_LOADER_IDENTIFIER.to_string(),
          "`env` and `jsc.target` cannot be used together".to_string(),
        ));
      }
      swc_options
    };

    let javascript_compiler = JavaScriptCompiler::new();
    let filename = FileName::Real(resource_path.into_std_path_buf());

    let source = content.into_string_lossy();

    let TransformOutput {
      code,
      map,
      diagnostics,
    } = javascript_compiler.transform(
      source,
      Some(filename),
      swc_options,
      Some(loader_context.context.module_source_map_kind),
      |_| transformer::transform(&self.options_with_additional.rspack_experiments),
    )?;

    for diagnostic in diagnostics {
      loader_context.emit_diagnostic(
        miette::miette! { severity = miette::Severity::Warning, "{}", diagnostic }.into(),
      );
    }

    loader_context.finish_with((code, map));

    Ok(())
  }
}

pub const SWC_LOADER_IDENTIFIER: &str = "builtin:swc-loader";

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for SwcLoader {
  #[tracing::instrument("loader:builtin-swc", skip_all, fields(
    perfetto.track_name = "loader:builtin-swc",
    perfetto.process_name = "Loader Analysis",
    resource =loader_context.resource(),
  ))]
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    #[allow(unused_mut)]
    let mut inner = || self.loader_impl(loader_context);
    #[cfg(all(debug_assertions, not(target_family = "wasm")))]
    {
      // Adjust stack to avoid stack overflow.
      stacker::maybe_grow(
        2 * 1024 * 1024, /* 2mb */
        4 * 1024 * 1024, /* 4mb */
        inner,
      )
    }
    #[cfg(any(not(debug_assertions), target_family = "wasm"))]
    inner()
  }
}

impl Identifiable for SwcLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }
}
