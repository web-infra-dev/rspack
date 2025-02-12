use std::borrow::Cow;

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
  rspack_sources::{encode_mappings, Mapping, OriginalLocation, SourceMap},
  Loader, LoaderContext, RunnerContext,
};
use rspack_error::Result;
use rspack_loader_runner::{Identifiable, Identifier};
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
      matches!(&self.config.draft, Some(draft) if draft.custom_media),
    );
    parser_flags.set(
      ParserFlags::DEEP_SELECTOR_COMBINATOR,
      matches!(&self.config.non_standard, Some(non_standard) if non_standard.deep_selector_combinator),
    );

    let option = ParserOptions {
      filename: filename.clone(),
      css_modules: None,
      source_index: 0,
      error_recovery: self.config.error_recovery.unwrap_or(true),
      warnings: None,
      flags: parser_flags,
    };
    let stylesheet = StyleSheet::parse(&content_str, option.clone())
      .map_err(|e| rspack_error::error!(e.to_string()))?;

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
        .map(|include| Features::from_bits_truncate(*include))
        .unwrap_or(Features::empty()),
      exclude: self
        .config
        .exclude
        .as_ref()
        .map(|exclude| Features::from_bits_truncate(*exclude))
        .unwrap_or(Features::empty()),
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
      .map_err(|e| rspack_error::error!(e))?;

    let enable_sourcemap = loader_context.context.module_source_map_kind.enabled();

    let mut source_map = loader_context
      .source_map()
      .map(|input_source_map| -> Result<_> {
        let mut sm = parcel_sourcemap::SourceMap::new(
          input_source_map
            .source_root()
            .unwrap_or(&loader_context.context.options.context),
        );
        sm.add_source(&filename);
        sm.set_source_content(0, &content_str)
          .map_err(|e| rspack_error::error!(e))?;
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
      });

    let content = stylesheet
      .to_css(PrinterOptions {
        minify: self.config.minify.unwrap_or(false),
        source_map: if enable_sourcemap {
          Some(&mut source_map)
        } else {
          None
        },
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
      .map_err(|_| rspack_error::error!("failed to generate css"))?;

    if enable_sourcemap {
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
          .map(ToString::to_string)
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

impl Identifiable for LightningCssLoader {
  fn identifier(&self) -> rspack_loader_runner::Identifier {
    self.id
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for LightningCssLoader {
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
