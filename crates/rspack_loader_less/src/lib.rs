use std::sync::Arc;

use cow_utils::CowUtils;
use parking_lot::Mutex;
use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  Loader, LoaderContext, RunnerContext,
  rspack_sources::{
    ConcatSource, MapOptions, OriginalSource, RawStringSource, SourceExt, SourceMap,
    SourceMapSource, SourceMapSourceOptions,
  },
};
use rspack_error::{Diagnostic, Result, ToStringResultToRspackResultExt};
use rspack_loader_runner::{Identifier, LoaderDependencies};

mod file_system;
pub mod options;
mod plugin;

pub use options::LessLoaderOptions;
pub use plugin::LessLoaderPlugin;

pub const LESS_LOADER_IDENTIFIER: &str = "builtin:less-loader";

#[cacheable]
#[derive(Debug)]
pub struct LessLoader {
  identifier: Identifier,
  options: LessLoaderOptions,
}

impl LessLoader {
  pub fn new(raw_options: &str) -> Result<Self, serde_json::Error> {
    Ok(Self {
      identifier: LESS_LOADER_IDENTIFIER.into(),
      options: serde_json::from_str(raw_options)?,
    })
  }

  pub fn with_identifier(mut self, identifier: Identifier) -> Self {
    self.identifier = identifier;
    self
  }

  async fn loader_impl(&self, context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(resource) = context.resource_path() else {
      return Ok(());
    };
    let filename = resource.as_str().cow_replace('\\', "/").into_owned();
    let Some(content) = context.take_content() else {
      return Ok(());
    };
    let mut content = content.try_into_string()?;
    let use_source_map = self
      .options
      .source_map
      .unwrap_or_else(|| context.context.source_map_kind.enabled());
    let mut options = self.options.compiler_options(&filename, use_source_map);
    let filename = options.filename.take().unwrap_or(filename);
    let source_map = options.source_map.is_some();
    let mut inner_source_map = context.take_source_map();
    if let Some(additional_data) = &self.options.additional_data {
      let prefix = format!("{additional_data}\n");
      if source_map {
        let original = if let Some(map) = inner_source_map.take() {
          SourceMapSource::new(SourceMapSourceOptions {
            value: content.clone(),
            name: filename.clone(),
            source_map: map,
            original_source: None,
            inner_source_map: None,
            remove_original_source: false,
          })
          .boxed()
        } else {
          OriginalSource::new(content.clone(), filename.clone()).boxed()
        };
        let source =
          ConcatSource::new([RawStringSource::from(prefix.clone()).boxed(), original]).boxed();
        inner_source_map = source.map_static(&MapOptions::default());
      }
      content.insert_str(0, &prefix);
    }
    for path in &mut options.include_paths {
      *path = context
        .context
        .options
        .context
        .as_path()
        .join(&*path)
        .as_str()
        .cow_replace('\\', "/")
        .into_owned();
    }
    let dependencies = Arc::new(Mutex::new(LoaderDependencies::default()));
    let compiler = less_rs::Compiler::builder()
      .options(options)
      .file_system(file_system::FileSystem {
        fs: Arc::clone(&context.context.fs),
        context: context.context.options.context.as_str().into(),
        dependencies: Arc::clone(&dependencies),
      })
      .build();
    // Less is synchronous and its input filesystem may call back into Node.js.
    // Run it outside Tokio so those synchronous reads can drive their futures.
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let span = tracing::Span::current();
    rayon::spawn(move || {
      span.in_scope(|| {
        let output = compiler.compile_str(&content, &filename);
        let _ = sender.send((output, content, filename));
      });
    });
    let (output, content, filename) = receiver.await.to_rspack_result_with_message(|_| {
      "builtin:less-loader compilation task was cancelled".to_string()
    })?;
    context.add_dependencies(&dependencies.lock());
    let output = output.to_rspack_result()?;
    for warning in output.warnings {
      context.emit_diagnostic(Diagnostic::warn(
        LESS_LOADER_IDENTIFIER.to_string(),
        warning.message,
      ));
    }
    let mut map = output
      .map
      .map(|map| SourceMap::from_json(map).to_rspack_result())
      .transpose()?;
    if use_source_map && let Some(map) = &mut map {
      // The final output filename is determined by the rest of the loader chain.
      map.set_file(None);
      map.set_source_root(Some(String::new().into()));
    }
    let map = match (map, inner_source_map) {
      (Some(map), Some(inner_source_map)) => SourceMapSource::new(SourceMapSourceOptions {
        value: output.css.clone(),
        name: filename,
        source_map: map,
        original_source: Some(content.into()),
        inner_source_map: Some(inner_source_map),
        remove_original_source: true,
      })
      .boxed()
      .map_static(&MapOptions::default()),
      (map, _) => map,
    };
    let additional_data = context.take_additional_data();
    context.finish_with((output.css, map, additional_data));
    Ok(())
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for LessLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  #[tracing::instrument("loader:less", skip_all, fields(
    perfetto.track_name = "loader:less",
    perfetto.process_name = "Loader Analysis",
    resource = context.resource(),
  ))]
  async fn run(&self, context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    self.loader_impl(context).await
  }
}
