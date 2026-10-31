use std::{
  borrow::Cow,
  sync::{Arc, RwLock},
};

use config::Config;
use cow_utils::CowUtils;
use derive_more::Debug;
pub use lightningcss;
use lightningcss::{
  error::MinifyErrorKind,
  media_query::{MediaCondition, MediaFeatureName, QueryFeature},
  printer::{PrinterOptions, PseudoClasses},
  rules::CssRule,
  stylesheet::{MinifyOptions, ParserFlags, ParserOptions, StyleSheet},
  targets::{Features, Targets},
  traits::IntoOwned,
};
use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  DependencyCategory, Loader, LoaderContext, Module, ResolveOptionsWithDependencyType,
  ResolveResult, RunnerContext,
  rspack_sources::{
    MapOptions, Mapping, OriginalLocation, SourceExt, SourceMap, SourceMapSource,
    SourceMapSourceOptions, encode_mappings,
  },
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_loader_runner::Identifier;
use tokio::sync::Mutex;

pub mod config;
mod plugin;

pub use plugin::LightningcssLoaderPlugin;

pub const LIGHTNINGCSS_LOADER_IDENTIFIER: &str = "builtin:lightningcss-loader";

pub type LightningcssLoaderVisitor = Box<dyn Send + Fn(&mut StyleSheet<'static>)>;

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

  async fn parse<'i>(
    &self,
    content: &'i str,
    options: ParserOptions<'i>,
  ) -> Result<StyleSheet<'static>> {
    let stylesheet = StyleSheet::parse(content, options.clone()).to_rspack_result()?;

    let mut flags = options.flags;
    // Custom media in `@import` conditions isn't expanded, so its definitions must be kept.
    if has_custom_media_import(&stylesheet) {
      flags.remove(ParserFlags::CUSTOM_MEDIA);
    }

    let mut stylesheet = to_static(
      stylesheet,
      ParserOptions {
        filename: options.filename,
        css_modules: None,
        source_index: 0,
        error_recovery: true,
        warnings: None,
        flags,
      },
    );

    if let Some(visitors) = &self.visitors {
      let visitors = visitors.lock().await;
      for v in visitors.iter() {
        v(&mut stylesheet);
      }
    }

    Ok(stylesheet)
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
    let mut stylesheet = self.parse(&content_str, option.clone()).await?;
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

    let minify_options = || MinifyOptions {
      targets,
      unused_symbols: self
        .config
        .unused_symbols
        .iter()
        .flatten()
        .cloned()
        .collect(),
    };

    match stylesheet.minify(minify_options()) {
      // The custom media may be defined in a CSS file that this one imports.
      Err(error) if matches!(error.kind, MinifyErrorKind::CustomMediaNotDefined { .. }) => {
        stylesheet = self.parse(&content_str, option).await?;
        let definitions = imported_custom_media(loader_context, &stylesheet).await;
        stylesheet.rules.0.splice(0..0, definitions);
        stylesheet.minify(minify_options()).to_rspack_result()?;
      }
      result => result.to_rspack_result()?,
    }

    let mut parcel_source_map = if loader_context.context.source_map_kind.enabled() {
      let mut sm = parcel_sourcemap::SourceMap::new(&loader_context.context.options.context);
      sm.add_source(&filename);
      sm.set_source_content(0, &content_str).to_rspack_result()?;
      Some(sm)
    } else {
      None
    };

    let content = stylesheet
      .to_css(PrinterOptions {
        minify: self.config.minify.unwrap_or(false),
        source_map: parcel_source_map.as_mut(),
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

    if let Some(parcel_source_map) = parcel_source_map {
      let mappings = encode_mappings(parcel_source_map.get_mappings().iter().map(|mapping| {
        // Parcel source map uses 0-based line numbers, while Rspack source map uses 1-based
        Mapping {
          generated_line: mapping.generated_line + 1,
          generated_column: mapping.generated_column,
          original: mapping.original.map(|original| OriginalLocation {
            source_index: original.source,
            original_line: original.original_line + 1,
            original_column: original.original_column,
            name_index: original.name,
          }),
        }
      }));

      let mut posix_context = loader_context
        .context
        .options
        .context
        .cow_replace("\\", "/");
      if !posix_context.ends_with('/') {
        posix_context.to_mut().push('/');
      }
      let posix_context = posix_context.into_owned();

      let rspack_source_map = SourceMap::new(
        mappings,
        // Parcel stores sources relative to project_root, while Rspack source maps
        // use absolute module paths for downstream loader/plugin handling.
        parcel_source_map
          .get_sources()
          .iter()
          .map(|source| {
            Cow::Owned(if source.starts_with('/') || source.contains(':') {
              source.clone()
            } else {
              let mut absolute_source = String::with_capacity(posix_context.len() + source.len());
              absolute_source.push_str(&posix_context);
              absolute_source.push_str(source);
              absolute_source
            })
          })
          .collect::<Vec<_>>(),
        parcel_source_map
          .get_sources_content()
          .iter()
          .map(|source_content| Cow::Owned(source_content.clone()))
          .collect::<Vec<_>>(),
        parcel_source_map
          .get_names()
          .iter()
          .map(|name| Cow::Owned(name.clone()))
          .collect::<Vec<_>>(),
      );

      let posix_name = filename.cow_replace("\\", "/");
      let source_map_source = SourceMapSource::new(SourceMapSourceOptions {
        value: content.code.clone(),
        name: posix_name,
        source_map: rspack_source_map,
        original_source: None,
        inner_source_map: loader_context.take_source_map(),
        remove_original_source: false,
      })
      .boxed();
      let source_map = source_map_source.map_static(&MapOptions::default());
      loader_context.finish_with((content.code, source_map));
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

pub fn to_static(stylesheet: StyleSheet, options: ParserOptions<'static>) -> StyleSheet<'static> {
  let sources = stylesheet.sources;
  let rules = stylesheet.rules.into_owned();
  let license_comments = stylesheet.license_comments.into_owned();

  let mut stylesheet = StyleSheet::new(sources, rules, options);
  stylesheet.license_comments = license_comments;
  stylesheet
}

/// Collects `@custom-media` rules from the CSS files that a stylesheet imports without conditions,
/// in the order they would be defined if the imports were inlined.
async fn imported_custom_media(
  loader_context: &mut LoaderContext<RunnerContext>,
  stylesheet: &StyleSheet<'_>,
) -> Vec<CssRule<'static>> {
  let Some(resource_path) = loader_context.resource_path().map(ToOwned::to_owned) else {
    return Vec::new();
  };
  let context = &loader_context.context;
  let resolver = context
    .resolver_factory
    .get(ResolveOptionsWithDependencyType {
      resolve_options: context
        .module
        .get_resolve_options()
        .map(|options| Box::new(Arc::unwrap_or_clone(options))),
      resolve_to_context: false,
      dependency_category: DependencyCategory::CssImport,
    });

  let mut files = Vec::new();
  let mut definitions = Vec::new();
  // Each entry is a file with its pending import requests and its own definitions.
  let mut stack = vec![(
    resource_path.clone(),
    import_requests(stylesheet),
    Vec::new(),
  )];
  while let Some((importer, requests, own_definitions)) = stack.last_mut() {
    let Some(request) = requests.pop() else {
      definitions.append(own_definitions);
      stack.pop();
      continue;
    };
    let Some(dir) = importer.parent() else {
      continue;
    };
    let Ok(ResolveResult::Resource(resource)) = resolver.resolve(dir.as_std_path(), &request).await
    else {
      continue;
    };
    if resource.path == resource_path || files.contains(&resource.path) {
      continue;
    }
    files.push(resource.path.clone());
    let Ok(content) = context.fs.read(&resource.path).await else {
      continue;
    };
    let content = String::from_utf8_lossy(&content);
    let Ok(imported) = StyleSheet::parse(
      &content,
      ParserOptions {
        filename: resource.path.to_string(),
        error_recovery: true,
        flags: ParserFlags::CUSTOM_MEDIA,
        ..ParserOptions::default()
      },
    ) else {
      continue;
    };
    let requests = import_requests(&imported);
    let own_definitions = imported
      .rules
      .0
      .into_iter()
      .filter(|rule| matches!(rule, CssRule::CustomMedia(_)))
      .map(IntoOwned::into_owned)
      .collect();
    stack.push((resource.path, requests, own_definitions));
  }

  for file in files {
    loader_context.add_file_dependency(file);
  }
  definitions
}

/// Returns the requests of unconditional `@import` rules, last first.
fn import_requests(stylesheet: &StyleSheet) -> Vec<String> {
  stylesheet
    .rules
    .0
    .iter()
    .rev()
    .filter_map(|rule| match rule {
      CssRule::Import(import)
        if import.layer.is_none()
          && import.supports.is_none()
          && import.media.media_queries.is_empty() =>
      {
        Some(
          import
            .url
            .strip_prefix('~')
            .unwrap_or(&import.url)
            .to_string(),
        )
      }
      _ => None,
    })
    .collect()
}

fn has_custom_media_import(stylesheet: &StyleSheet) -> bool {
  stylesheet.rules.0.iter().any(|rule| {
    matches!(rule, CssRule::Import(import) if import
      .media
      .media_queries
      .iter()
      .any(|query| query.condition.as_ref().is_some_and(has_custom_media_reference)))
  })
}

fn has_custom_media_reference(condition: &MediaCondition) -> bool {
  match condition {
    MediaCondition::Feature(QueryFeature::Boolean {
      name: MediaFeatureName::Custom(_),
    }) => true,
    MediaCondition::Not(condition) => has_custom_media_reference(condition),
    MediaCondition::Operation { conditions, .. } => {
      conditions.iter().any(has_custom_media_reference)
    }
    MediaCondition::Feature(_) | MediaCondition::Unknown(_) => false,
  }
}
