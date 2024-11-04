#![feature(let_chains)]

mod minify;

use std::collections::HashMap;
use std::hash::Hash;
use std::path::Path;
use std::sync::{mpsc, LazyLock, Mutex};

use cow_utils::CowUtils;
use once_cell::sync::OnceCell;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::rspack_sources::{ConcatSource, MapOptions, RawSource, SourceExt, SourceMap};
use rspack_core::rspack_sources::{Source, SourceMapSource, SourceMapSourceOptions};
use rspack_core::{
  AssetInfo, ChunkUkey, Compilation, CompilationAsset, CompilationParams, CompilationProcessAssets,
  CompilerCompilation, Plugin, PluginContext,
};
use rspack_error::miette::IntoDiagnostic;
use rspack_error::{Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{JavascriptModulesChunkHash, JsPlugin};
use rspack_util::asset_condition::AssetConditions;
use swc_config::config_types::BoolOrDataConfig;
use swc_core::base::config::JsMinifyFormatOptions;
pub use swc_ecma_minifier::option::terser::{TerserCompressorOptions, TerserEcmaVersion};
pub use swc_ecma_minifier::option::MangleOptions;

use self::minify::{match_object, minify};

const PLUGIN_NAME: &str = "rspack.SwcJsMinimizerRspackPlugin";

static JAVASCRIPT_ASSET_REGEXP: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\.[cm]?js(\?.*)?$").expect("Invalid RegExp"));

#[derive(Debug, Hash)]
pub struct PluginOptions {
  pub test: Option<AssetConditions>,
  pub include: Option<AssetConditions>,
  pub exclude: Option<AssetConditions>,
  pub extract_comments: Option<ExtractComments>,
  pub minimizer_options: MinimizerOptions,
}

#[derive(Debug, Default)]
pub struct MinimizerOptions {
  pub minify: Option<bool>,
  pub compress: BoolOrDataConfig<TerserCompressorOptions>,
  pub mangle: BoolOrDataConfig<MangleOptions>,
  pub format: JsMinifyFormatOptions,
  pub module: Option<bool>,

  /// Internal fields for hashing only.
  /// This guaranteed these field should only be readonly.
  /// Otherwise, hash would be generated with inconsistencies.
  pub __compress_cache: OnceCell<BoolOrDataConfig<String>>,
  pub __mangle_cache: OnceCell<BoolOrDataConfig<String>>,
  pub __format_cache: OnceCell<String>,
}

impl std::hash::Hash for MinimizerOptions {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self
      .__format_cache
      .get_or_init(|| serde_json::to_string(&self.format).expect("Should be able to serialize"))
      .hash(state);
    self
      .__compress_cache
      .get_or_init(|| {
        self
          .compress
          .as_ref()
          .map(|v| serde_json::to_string(v).expect("Should be able to serialize"))
      })
      .hash(state);
    self
      .__mangle_cache
      .get_or_init(|| {
        self
          .mangle
          .as_ref()
          .map(|v| serde_json::to_string(v).expect("Should be able to serialize"))
      })
      .hash(state);
  }
}

#[derive(Debug, Hash)]
pub enum OptionWrapper<T: std::fmt::Debug + Hash> {
  Default,
  Disabled,
  Custom(T),
}

#[derive(Debug)]
pub struct ExtractComments {
  pub condition: String,
  pub banner: OptionWrapper<String>,
}

impl Hash for ExtractComments {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.condition.as_str().hash(state);
    self.banner.hash(state);
  }
}

#[derive(Debug)]
struct NormalizedExtractComments<'a> {
  filename: String,
  condition: &'a Regex,
  banner: Option<String>,
}

#[plugin]
#[derive(Debug)]
pub struct SwcJsMinimizerRspackPlugin {
  options: PluginOptions,
}

impl SwcJsMinimizerRspackPlugin {
  pub fn new(options: PluginOptions) -> Self {
    Self::new_inner(options)
  }
}

#[plugin_hook(CompilerCompilation for SwcJsMinimizerRspackPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation);
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for SwcJsMinimizerRspackPlugin)]
async fn js_chunk_hash(
  &self,
  _compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  PLUGIN_NAME.hash(hasher);
  self.options.hash(hasher);
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for SwcJsMinimizerRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let options = &self.options;
  let minimizer_options = &self.options.minimizer_options;

  let (tx, rx) = mpsc::channel::<Vec<Diagnostic>>();
  // collect all extracted comments info
  let all_extracted_comments = Mutex::new(HashMap::new());
  let extract_comments_condition = options
    .extract_comments
    .as_ref()
    .map(|extract_comment| extract_comment.condition.as_ref())
    .map(|condition| {
      Regex::new(condition)
        .unwrap_or_else(|_| panic!("`{condition}` is invalid extractComments condition"))
    });

  compilation
    .assets_mut()
    .par_iter_mut()
    .filter(|(filename, original)| {
      if !JAVASCRIPT_ASSET_REGEXP.is_match(filename) {
        return false
      }

      let is_matched = match_object(options, filename);

      if !is_matched || original.get_info().minimized.unwrap_or(false) {
        return false
      }

      true
    })
    .try_for_each_with(tx,|tx, (filename, original)| -> Result<()>  {
      let filename = filename.split('?').next().expect("Should have filename");
      if let Some(original_source) = original.get_source() {
        let input = original_source.source().to_string();
        let input_source_map = original_source.map(&MapOptions::default());

        let is_module = if let Some(module) = minimizer_options.module {
          Some(module)
        } else if let Some(module) = original.info.javascript_module {
          Some(module)
        } else if filename.ends_with(".mjs") {
          Some(true)
        } else if filename.ends_with(".cjs") {
          Some(false)
        } else {
          None
        };

        let js_minify_options = JsMinifyOptions {
          minify: minimizer_options.minify.unwrap_or(true),
          compress: minimizer_options.compress.clone(),
          mangle: minimizer_options.mangle.clone(),
          format: minimizer_options.format.clone(),
          source_map: BoolOrDataConfig::from_bool(input_source_map.is_some()),
          inline_sources_content: true, /* Using true so original_source can be None in SourceMapSource */
          module: is_module,
          ..Default::default()
          };
        let extract_comments_option = options.extract_comments.as_ref().map(|extract_comments| {
          let comments_filename = format!("{filename}.LICENSE.txt");
          let banner = match &extract_comments.banner {
            OptionWrapper::Default => {
              let dir = Path::new(filename).parent().expect("should has parent");
              let raw = Path::new(&comments_filename).strip_prefix(dir).expect("should has common prefix").to_string_lossy();
              let relative = raw.cow_replace('\\', "/");
              Some(format!("/*! For license information please see {relative} */"))
            },
            OptionWrapper::Disabled => None,
            OptionWrapper::Custom(value) => Some(format!("/*! {value} */"))
          };
          NormalizedExtractComments {
            filename: comments_filename,
            condition: extract_comments_condition.as_ref().expect("must exists"),
            banner
          }
        });
        let output = match minify(
          &js_minify_options,
          input,
          filename,
          &all_extracted_comments,
          &extract_comments_option,
        ) {
          Ok(r) => r,
          Err(e) => {
            tx.send(e.into()).into_diagnostic()?;
            return Ok(())
          }
        };
        let source = if let Some(map) = &output.map {
          SourceMapSource::new(SourceMapSourceOptions {
            value: output.code,
            name: filename,
            source_map: SourceMap::from_json(map).expect("should be able to generate source-map"),
            original_source: None,
            inner_source_map: input_source_map,
            remove_original_source: true,
          })
          .boxed()
        } else {
          RawSource::from(output.code).boxed()
        };
        let source = if let Some(Some(banner)) = extract_comments_option.map(|option| option.banner)
          && all_extracted_comments
          .lock()
          .expect("all_extract_comments lock failed")
          .contains_key(filename)
        {
          ConcatSource::new([
            RawSource::from(banner).boxed(),
            RawSource::from("\n").boxed(),
            source
          ]).boxed()
        } else {
          source
        };
        original.set_source(Some(source));
        original.get_info_mut().minimized.replace(true);
      }

      Ok(())
  })?;

  compilation.extend_diagnostics(rx.into_iter().flatten().collect::<Vec<_>>());

  // write all extracted comments to assets
  all_extracted_comments
    .lock()
    .expect("all_extracted_comments lock failed")
    .clone()
    .into_iter()
    .for_each(|(_, comments)| {
      compilation.emit_asset(
        comments.comments_file_name,
        CompilationAsset {
          source: Some(comments.source),
          info: AssetInfo {
            minimized: Some(true),
            ..Default::default()
          },
        },
      )
    });

  Ok(())
}

impl Plugin for SwcJsMinimizerRspackPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}

#[derive(Debug, Clone, Default)]
pub struct JsMinifyOptions {
  pub minify: bool,
  pub compress: BoolOrDataConfig<TerserCompressorOptions>,
  pub mangle: BoolOrDataConfig<MangleOptions>,
  pub format: JsMinifyFormatOptions,
  pub ecma: TerserEcmaVersion,
  pub keep_class_names: bool,
  pub keep_fn_names: bool,
  pub module: Option<bool>,
  pub safari10: bool,
  pub toplevel: bool,
  pub source_map: BoolOrDataConfig<TerserSourceMapKind>,
  pub output_path: Option<String>,
  pub inline_sources_content: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TerserSourceMapKind {
  pub filename: Option<String>,
  pub url: Option<String>,
  pub root: Option<String>,
  pub content: Option<String>,
}
