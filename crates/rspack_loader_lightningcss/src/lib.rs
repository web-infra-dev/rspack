use std::{
  borrow::Cow,
  sync::{Arc, RwLock},
};

use config::Config;
use derive_more::Debug;
pub use lightningcss;
use lightningcss::{
  printer::{PrinterOptions, PseudoClasses},
  stylesheet::{MinifyOptions, ParserFlags, ParserOptions, StyleSheet},
  targets::{Features, Targets},
  traits::IntoOwned,
};
use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  Loader, LoaderContext, RunnerContext,
  rspack_sources::{Mapping, OriginalLocation, SourceMap, encode_mappings},
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_loader_runner::Identifier;
use tokio::sync::Mutex;

pub mod config;
mod plugin;

pub use plugin::LightningcssLoaderPlugin;

pub const LIGHTNINGCSS_LOADER_IDENTIFIER: &str = "builtin:lightningcss-loader";

pub type LightningcssLoaderVisitor = Box<dyn Send + Fn(&mut StyleSheet<'static, 'static>)>;

#[cacheable]
#[derive(Debug)]
pub struct LightningCssLoader {
  id: Identifier,
  #[debug(skip)]
  #[cacheable(with=Skip)]
  visitors: Option<Mutex<Vec<LightningcssLoaderVisitor>>>,
  config: Config,
}

impl LightningCssLoader {
  pub fn new(
    visitors: Option<Vec<LightningcssLoaderVisitor>>,
    config: Config,
    ident: &str,
  ) -> Self {
    Self {
      id: ident.into(),
      visitors: visitors.map(|v| Mutex::new(v)),
      config,
    }
  }

  async fn loader_impl(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let Some(resource_path) = loader_context.resource_path() else {
      return Ok(());
    };

    let filename = resource_path.as_str().to_string();

    let Some(content) = loader_context.take_content() else {
      return Ok(());
    };

    let content_str = match &content {
      rspack_core::Content::String(s) => Cow::Borrowed(s.as_str()),
      rspack_core::Content::Buffer(buf) => String::from_utf8_lossy(buf),
    };

    let mut parser_flags = ParserFlags::empty();
    parser_flags.set(
      ParserFlags::CUSTOM_MEDIA,
      matches!(&self.config.drafts, Some(drafts) if drafts.custom_media),
    );
    parser_flags.set(
      ParserFlags::DEEP_SELECTOR_COMBINATOR,
      matches!(&self.config.non_standard, Some(non_standard) if non_standard.deep_selector_combinator),
    );

    let error_recovery = self.config.error_recovery.unwrap_or(true);
    let warnings = if error_recovery {
      Some(Arc::new(RwLock::new(Vec::new())))
    } else {
      None
    };

    let option = ParserOptions {
      filename: filename.clone(),
      css_modules: None,
      source_index: 0,
      error_recovery,
      warnings: warnings.clone(),
      flags: parser_flags,
    };
    let stylesheet = StyleSheet::parse(&content_str, option.clone()).to_rspack_result()?;
    // FIXME: Disable the warnings for now, cause it cause too much positive-negative warnings,
    // enable when we have a better way to handle it.

    // if let Some(warnings) = warnings {
    //   #[allow(clippy::unwrap_used)]
    //   let warnings = warnings.read().unwrap();
    //   for warning in warnings.iter() {
    //     if matches!(
    //       warning.kind,
    //       lightningcss::error::ParserError::SelectorError(
    //         lightningcss::error::SelectorError::UnsupportedPseudoClass(_)
    //       ) | lightningcss::error::ParserError::SelectorError(
    //         lightningcss::error::SelectorError::UnsupportedPseudoElement(_)
    //       )
    //     ) {
    //       // ignore parsing errors on pseudo class from lightningcss-loader
    //       // to allow pseudo class in CSS modules and Vue.
    //       continue;
    //     }
    //     loader_context.emit_diagnostic(Diagnostic::warn(
    //       "builtin:lightningcss-loader".to_string(),
    //       format!("LightningCSS parse warning: {}", warning),
    //     ));
    //   }
    // }

    let mut stylesheet = to_static(
      stylesheet,
      ParserOptions {
        filename: filename.clone(),
        css_modules: None,
        source_index: 0,
        error_recovery: true,
        warnings: None,
        flags: ParserFlags::empty(),
      },
    );

    if let Some(visitors) = &self.visitors {
      let visitors = visitors.lock().await;
      for v in visitors.iter() {
        v(&mut stylesheet);
      }
    }

    let targets = Targets {
      browsers: self.config.targets,
      include: self
        .config
        .include
        .as_ref()
        .map_or(Features::empty(), |include| {
          Features::from_bits_truncate(*include)
        }),
      exclude: self
        .config
        .exclude
        .as_ref()
        .map_or(Features::empty(), |exclude| {
          Features::from_bits_truncate(*exclude)
        }),
    };

    let unused_symbols = self
      .config
      .unused_symbols
      .clone()
      .map(|unused_symbols| unused_symbols.into_iter().collect())
      .unwrap_or_default();

    stylesheet
      .minify(MinifyOptions {
        targets,
        unused_symbols,
      })
      .to_rspack_result()?;

    let mut source_map = if loader_context.context.source_map_kind.enabled() {
      Some(
        loader_context
          .source_map()
          .map(|input_source_map| -> Result<_> {
            let mut sm = parcel_sourcemap::SourceMap::new(
              input_source_map
                .source_root()
                .unwrap_or(&loader_context.context.options.context),
            );
            sm.add_source(&filename);
            sm.set_source_content(0, &content_str).to_rspack_result()?;
            Ok(sm)
          })
          .transpose()?
          .unwrap_or_else(|| {
            let mut source_map =
              parcel_sourcemap::SourceMap::new(&loader_context.context.options.context);
            let source_idx = source_map.add_source(&filename);
            source_map
              .set_source_content(source_idx as usize, &content_str)
              .expect("should set source content");
            source_map
          }),
      )
    } else {
      None
    };

    let content = stylesheet
      .to_css(PrinterOptions {
        minify: self.config.minify.unwrap_or(false),
        source_map: source_map.as_mut(),
        project_root: None,
        targets,
        analyze_dependencies: None,
        pseudo_classes: self
          .config
          .pseudo_classes
          .as_ref()
          .map(|pseudo_classes| PseudoClasses {
            hover: pseudo_classes.hover.as_deref(),
            active: pseudo_classes.active.as_deref(),
            focus: pseudo_classes.focus.as_deref(),
            focus_visible: pseudo_classes.focus_visible.as_deref(),
            focus_within: pseudo_classes.focus_within.as_deref(),
          }),
      })
      .to_rspack_result_with_message(|e| format!("failed to generate css: {e}"))?;

    if let Some(source_map) = source_map {
      let mappings = encode_mappings(source_map.get_mappings().iter().map(|mapping| Mapping {
        generated_line: mapping.generated_line,
        generated_column: mapping.generated_column,
        original: mapping.original.map(|original| OriginalLocation {
          source_index: original.source,
          original_line: original.original_line,
          original_column: original.original_column,
          name_index: original.name,
        }),
      }));
      let rspack_source_map = SourceMap::new(
        mappings,
        source_map
          .get_sources()
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<_>>(),
        source_map
          .get_sources_content()
          .iter()
          .map(|source_content| Arc::from(source_content.clone()))
          .collect::<Vec<_>>(),
        source_map
          .get_names()
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<_>>(),
      );
      loader_context.finish_with((content.code, rspack_source_map));
    } else {
      loader_context.finish_with(content.code);
    }

    Ok(())
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for LightningCssLoader {
  fn identifier(&self) -> rspack_loader_runner::Identifier {
    self.id
  }

  #[tracing::instrument("loader:lightningcss", skip_all, fields(
    perfetto.track_name = "loader:lightningcss",
    perfetto.process_name = "Loader Analysis",
    resource =loader_context.resource(),
  ))]
  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    // for better diagnostic, as async_trait macro don't show beautiful error message
    self.loader_impl(loader_context).await
  }
}

pub fn to_static(
  stylesheet: StyleSheet,
  options: ParserOptions<'static, 'static>,
) -> StyleSheet<'static, 'static> {
  let sources = stylesheet.sources.clone();
  let rules = stylesheet.rules.clone().into_owned();

  StyleSheet::new(sources, rules, options)
}
