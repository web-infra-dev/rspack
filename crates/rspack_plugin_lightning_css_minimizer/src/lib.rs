use std::{
  borrow::Cow,
  collections::HashSet,
  hash::Hash,
  sync::{Arc, LazyLock, RwLock},
};

pub use lightningcss::targets::Browsers;
use lightningcss::{
  printer::{
    OriginalLocation as LightningOriginalLocation, PrinterOptions, SourceMap as LightningSourceMap,
  },
  stylesheet::{MinifyOptions, ParserFlags, ParserOptions, StyleSheet},
  targets::{Features, Targets},
};
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  ChunkUkey, Compilation, CompilationChunkHash, CompilationProcessAssets, Plugin,
  diagnostics::MinifyError,
  rspack_sources::{
    MapOptions, Mapping, ObjectPool, OriginalLocation as RspackOriginalLocation, RawStringSource,
    Source, SourceExt, SourceMap, SourceMapSource, SourceMapSourceOptions, encode_mappings,
  },
};
use rspack_error::{Diagnostic, Result, ToStringResultToRspackResultExt};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::asset_condition::{AssetConditions, AssetConditionsObject, match_object};
use thread_local::ThreadLocal;

static CSS_ASSET_REGEXP: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\.css(\?.*)?$").expect("Invalid RegExp"));

#[derive(Default)]
struct RspackSourceMap {
  sources: Vec<Cow<'static, str>>,
  sources_content: Vec<Cow<'static, str>>,
  names: Vec<Cow<'static, str>>,
  mappings: Vec<Mapping>,
  source_root: Option<Cow<'static, str>>,
}

impl RspackSourceMap {
  fn with_source_root(source_root: Option<&str>) -> Self {
    Self {
      source_root: source_root.map(|source_root| Cow::Owned(source_root.to_string())),
      ..Default::default()
    }
  }

  fn finish(self) -> SourceMap<'static> {
    let mut source_map = SourceMap::new(
      encode_mappings(self.mappings.into_iter()),
      self.sources,
      self.sources_content,
      self.names,
    );
    source_map.set_source_root(self.source_root);
    source_map
  }
}

impl LightningSourceMap for RspackSourceMap {
  fn add_source(&mut self, source: &str) -> u32 {
    if let Some(index) = self.sources.iter().position(|s| s.as_ref() == source) {
      index as u32
    } else {
      self.sources.push(Cow::Owned(source.to_string()));
      (self.sources.len() - 1) as u32
    }
  }

  fn add_name(&mut self, name: &str) -> u32 {
    if let Some(index) = self.names.iter().position(|n| n.as_ref() == name) {
      index as u32
    } else {
      self.names.push(Cow::Owned(name.to_string()));
      (self.names.len() - 1) as u32
    }
  }

  fn set_source_content(&mut self, source_index: u32, source_content: &str) {
    let source_index = source_index as usize;
    if self.sources_content.len() <= source_index {
      self
        .sources_content
        .resize_with(source_index + 1, || Cow::Borrowed(""));
    }
    self.sources_content[source_index] = Cow::Owned(source_content.to_string());
  }

  fn add_mapping(
    &mut self,
    generated_line: u32,
    generated_column: u32,
    original: Option<LightningOriginalLocation>,
  ) {
    self.mappings.push(Mapping {
      generated_line: generated_line + 1,
      generated_column,
      original: original.map(|original| RspackOriginalLocation {
        source_index: original.source,
        original_line: original.original_line + 1,
        original_column: original.original_column,
        name_index: original.name,
      }),
    });
  }

  fn from_data_url(_source_root: &str, _data_url: &str) -> Option<Self> {
    None
  }

  fn find_closest_mapping(
    &mut self,
    _line: u32,
    _column: u32,
  ) -> Option<LightningOriginalLocation> {
    None
  }

  fn get_source(&self, source_index: u32) -> Option<&str> {
    self
      .sources
      .get(source_index as usize)
      .map(|source| source.as_ref())
  }

  fn get_name(&self, name_index: u32) -> Option<&str> {
    self
      .names
      .get(name_index as usize)
      .map(|name| name.as_ref())
  }

  fn get_source_content(&self, source_index: u32) -> Option<&str> {
    self
      .sources_content
      .get(source_index as usize)
      .map(|source_content| source_content.as_ref())
  }
}

#[derive(Debug, Hash)]
pub struct PluginOptions {
  pub test: Option<AssetConditions>,
  pub include: Option<AssetConditions>,
  pub exclude: Option<AssetConditions>,
  pub remove_unused_local_idents: bool,
  pub minimizer_options: MinimizerOptions,
}

#[derive(Debug, Hash)]
pub struct Draft {
  pub custom_media: bool,
}

#[derive(Debug, Hash)]
pub struct NonStandard {
  pub deep_selector_combinator: bool,
}

#[derive(Debug, Hash)]
pub struct PseudoClasses {
  pub hover: Option<String>,
  pub active: Option<String>,
  pub focus: Option<String>,
  pub focus_visible: Option<String>,
  pub focus_within: Option<String>,
}

#[derive(Debug)]
pub struct MinimizerOptions {
  pub error_recovery: bool,
  pub targets: Option<Browsers>,
  pub include: Option<u32>,
  pub exclude: Option<u32>,
  pub drafts: Option<Draft>,
  pub non_standard: Option<NonStandard>,
  pub pseudo_classes: Option<PseudoClasses>,
  pub unused_symbols: Vec<String>,
}

impl Hash for MinimizerOptions {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.error_recovery.hash(state);
    self.include.hash(state);
    self.exclude.hash(state);
    self.drafts.hash(state);
    self.non_standard.hash(state);
    self.unused_symbols.hash(state);
    if let Some(pseudo_classes) = &self.pseudo_classes {
      pseudo_classes.hover.hash(state);
      pseudo_classes.active.hash(state);
      pseudo_classes.focus.hash(state);
      pseudo_classes.focus_visible.hash(state);
      pseudo_classes.focus_within.hash(state);
    }
    if let Some(targets) = &self.targets {
      targets.android.hash(state);
      targets.chrome.hash(state);
      targets.edge.hash(state);
      targets.firefox.hash(state);
      targets.ie.hash(state);
      targets.ios_saf.hash(state);
      targets.opera.hash(state);
      targets.safari.hash(state);
      targets.samsung.hash(state);
    }
  }
}

#[plugin]
#[derive(Debug)]
pub struct LightningCssMinimizerRspackPlugin {
  options: PluginOptions,
}

impl LightningCssMinimizerRspackPlugin {
  pub fn new(options: PluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilationChunkHash for LightningCssMinimizerRspackPlugin)]
async fn chunk_hash(
  &self,
  _compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  self.options.hash(hasher);
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for LightningCssMinimizerRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let options = &self.options;
  let minimizer_options = &self.options.minimizer_options;
  let all_warnings: RwLock<Vec<Diagnostic>> = Default::default();
  let condition_object = AssetConditionsObject {
    test: options.test.as_ref(),
    include: options.include.as_ref(),
    exclude: options.exclude.as_ref(),
  };

  let tls: ThreadLocal<ObjectPool> = ThreadLocal::new();
  compilation
    .assets_mut()
    .par_iter_mut()
    .filter(|(filename, original)| {
      if !CSS_ASSET_REGEXP.is_match(filename) {
        return false;
      }

      let is_matched = match_object(&condition_object, filename);

      if !is_matched || original.get_info().minimized.unwrap_or(false) {
        return false;
      }

      true
    })
    .try_for_each(|(filename, original)| -> Result<()> {
      if original.get_info().minimized.unwrap_or(false) {
        return Ok(());
      }

      if let Some(original_source) = original.get_source() {
        let input = original_source.source().into_string_lossy().into_owned();
        let object_pool = tls.get_or(ObjectPool::default);
        let input_source_map =
          Source::map_static(original_source.clone(), object_pool, &MapOptions::default());

        let mut parser_flags = ParserFlags::empty();
        parser_flags.set(
          ParserFlags::CUSTOM_MEDIA,
          matches!(&minimizer_options.drafts, Some(drafts) if drafts.custom_media),
        );
        parser_flags.set(
          ParserFlags::DEEP_SELECTOR_COMBINATOR,
          matches!(&minimizer_options.non_standard, Some(non_standard) if non_standard.deep_selector_combinator),
        );

        let mut source_map = input_source_map.as_ref().map(|input_source_map| {
          let mut sm = RspackSourceMap::with_source_root(input_source_map.source_root());
          let source_index = sm.add_source(filename);
          sm.set_source_content(source_index, &input);
          sm
        });
        let result = {
          let warnings: Arc<RwLock<Vec<_>>> = Default::default();
          let mut stylesheet = StyleSheet::parse(
            &input,
            ParserOptions {
              filename: filename.clone(),
              css_modules: None,
              source_index: 0,
              error_recovery: minimizer_options.error_recovery,
              warnings: Some(warnings.clone()),
              flags: parser_flags,
            },
          )
          .to_rspack_result()?;

          let targets = Targets {
            browsers: minimizer_options.targets,
            include: minimizer_options
              .include
              .as_ref()
              .map_or(Features::empty(), |include| Features::from_bits_truncate(*include)),
            exclude: minimizer_options
              .exclude
              .as_ref()
              .map_or(Features::empty(), |exclude| Features::from_bits_truncate(*exclude)),
          };
          let mut unused_symbols = HashSet::from_iter(minimizer_options.unused_symbols.clone());
          if self.options.remove_unused_local_idents
            && let Some(css_unused_idents) = original.info.css_unused_idents.take()
          {
            unused_symbols.extend(css_unused_idents.into_iter().map(String::from));
          }
          stylesheet
            .minify(MinifyOptions {
              targets,
              unused_symbols,
            })
            .to_rspack_result()?;
          // FIXME: Disable the warnings for now, cause it cause too much positive-negative warnings,
          // enable when we have a better way to handle it. let warnings = warnings.read().expect("should lock");
          // all_warnings.write().expect("should lock").extend(
          //   warnings.iter().map(|e| {
          //     if let Some(loc) = &e.loc {
          //       let rope = ropey::Rope::from_str(&input);
          //       let start = rope.line_to_byte(loc.line as usize) + loc.column as usize - 1;
          //       let end = start;
          //       Diagnostic::from(Box::new(Error::from_file(
          //         input.clone(),
          //         start,
          //         end,
          //         "LightningCSS minimize warning".to_string(),
          //         e.to_string(),
          //       )
          //       .with_severity(Severity::Warning)))
          //     } else {
          //       Diagnostic::warn("LightningCSS minimize warning".to_string(), e.to_string())
          //     }
          //   }),
          // );
          stylesheet
            .to_css(PrinterOptions {
              minify: true,
              project_root: None,
              targets,
              analyze_dependencies: None,
              pseudo_classes: minimizer_options.pseudo_classes
              .as_ref()
              .map(|pseudo_classes| lightningcss::stylesheet::PseudoClasses {
                hover: pseudo_classes.hover.as_deref(),
                active: pseudo_classes.active.as_deref(),
                focus: pseudo_classes.focus.as_deref(),
                focus_visible: pseudo_classes.focus_visible.as_deref(),
                focus_within: pseudo_classes.focus_within.as_deref(),
              }),
            }, source_map.as_mut())
            .to_rspack_result()?
        };

        let minimized_source = if let Some(source_map) = source_map {
          SourceMapSource::new(SourceMapSourceOptions {
            value: result.code,
            name: filename,
            source_map: source_map.finish(),
            original_source: Some(Box::from(input)),
            inner_source_map: input_source_map,
            remove_original_source: true,
          })
          .boxed()
        } else {
          RawStringSource::from(result.code).boxed()
        };

        original.set_source(Some(minimized_source));
      }
      original.get_info_mut().minimized.replace(true);
      Ok(())
    }).map_err(MinifyError)?;

  compilation.extend_diagnostics(all_warnings.into_inner().expect("should lock"));

  Ok(())
}

impl Plugin for LightningCssMinimizerRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.LightningCssMinimizerRspackPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compilation_hooks.chunk_hash.tap(chunk_hash::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
