use std::{
  fmt::{Display, Formatter},
  hash::Hasher,
  path::Path,
  sync::{LazyLock, Mutex, mpsc},
};

use cow_utils::CowUtils;
use once_cell::sync::OnceCell;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  AssetInfo, CacheOptions, CacheValue, ChunkUkey, Compilation, CompilationAsset, CompilationParams,
  CompilationProcessAssets, CompilerCompilation, Etag, Logger, Plugin,
  cache::{CachedExtractedComments, CachedMinimizeEntry},
  diagnostics::MinifyError,
  legacy_cache::persistent::occasion::minimize::MinimizeCacheKey,
  rspack_sources::{
    BoxSource, ConcatSource, MapOptions, ObjectPool, RawStringSource, Source, SourceExt,
    SourceMapSource, SourceMapSourceOptions,
  },
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::RspackHasher;
use rspack_hook::{plugin, plugin_hook};
use rspack_javascript_compiler::JavaScriptCompiler;
use rspack_plugin_javascript::{ExtractedCommentsInfo, JavascriptModulesChunkHash, JsPlugin};
use rspack_regex::RspackRegex;
use rspack_util::{
  asset_condition::AssetConditions,
  fx_hash::{FxHashMap, FxHasher},
};
use swc_config::types::BoolOrDataConfig;
use swc_core::{
  base::config::JsMinifyFormatOptions,
  common::comments::{CommentKind, SingleThreadedComments},
};
pub use swc_ecma_minifier::option::{
  MangleOptions,
  terser::{TerserCompressorOptions, TerserEcmaVersion},
};
use thread_local::ThreadLocal;

const PLUGIN_NAME: &str = "rspack.SwcJsMinimizerRspackPlugin";

static JAVASCRIPT_ASSET_REGEXP: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\.[cm]?js(\?.*)?$").expect("Invalid RegExp"));

#[derive(Debug, Hash, rspack_hash::RspackHash)]
pub struct PluginOptions {
  pub test: Option<AssetConditions>,
  pub include: Option<AssetConditions>,
  pub exclude: Option<AssetConditions>,
  pub extract_comments: Option<ExtractComments>,
  pub minimizer_options: MinimizerOptions,
}

#[derive(Debug, Default)]
pub struct MinimizerOptions {
  pub ecma: TerserEcmaVersion,
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

impl rspack_hash::RspackHash for MinimizerOptions {
  fn hash(&self, state: &mut RspackHasher) {
    rspack_hash::RspackHash::hash(
      self
        .__format_cache
        .get_or_init(|| simd_json::to_string(&self.format).expect("Should be able to serialize")),
      state,
    );
    rspack_hash::RspackHash::hash(
      self.__compress_cache.get_or_init(|| {
        self
          .compress
          .as_ref()
          .map(|v| simd_json::to_string(v).expect("Should be able to serialize"))
      }),
      state,
    );
    rspack_hash::RspackHash::hash(
      self.__mangle_cache.get_or_init(|| {
        self
          .mangle
          .as_ref()
          .map(|v| simd_json::to_string(v).expect("Should be able to serialize"))
      }),
      state,
    );
  }
}

impl std::hash::Hash for MinimizerOptions {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self
      .__format_cache
      .get_or_init(|| simd_json::to_string(&self.format).expect("Should be able to serialize"))
      .hash(state);
    self
      .__compress_cache
      .get_or_init(|| {
        self
          .compress
          .as_ref()
          .map(|v| simd_json::to_string(v).expect("Should be able to serialize"))
      })
      .hash(state);
    self
      .__mangle_cache
      .get_or_init(|| {
        self
          .mangle
          .as_ref()
          .map(|v| simd_json::to_string(v).expect("Should be able to serialize"))
      })
      .hash(state);
  }
}

#[derive(Debug, Hash)]
pub enum OptionWrapper<T: std::fmt::Debug + std::hash::Hash> {
  Default,
  Disabled,
  Custom(T),
}

impl<T: std::fmt::Debug + std::hash::Hash> Display for OptionWrapper<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      OptionWrapper::Default => "default",
      OptionWrapper::Disabled => "disabled",
      OptionWrapper::Custom(_) => "custom",
    })
  }
}

impl<T: std::fmt::Debug + std::hash::Hash + rspack_hash::RspackHash> rspack_hash::RspackHash
  for OptionWrapper<T>
{
  fn hash(&self, state: &mut RspackHasher) {
    rspack_hash::RspackHash::hash(&self.to_string(), state);
    if let OptionWrapper::Custom(value) = self {
      rspack_hash::RspackHash::hash(value, state);
    }
  }
}

#[derive(Debug, rspack_hash::RspackHash)]
pub struct ExtractComments {
  pub condition: String,
  pub condition_flags: String,
  pub banner: OptionWrapper<String>,
}

impl std::hash::Hash for ExtractComments {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.condition.as_str().hash(state);
    self.condition_flags.as_str().hash(state);
    self.banner.hash(state);
  }
}

#[derive(Debug)]
struct NormalizedExtractComments {
  filename: String,
  condition: RspackRegex,
  banner: Option<String>,
}

#[plugin]
#[derive(Debug)]
pub struct SwcJsMinimizerRspackPlugin {
  options: PluginOptions,
  options_hash: u64,
}

impl SwcJsMinimizerRspackPlugin {
  pub fn new(options: PluginOptions) -> Self {
    use std::hash::Hash;

    let mut hasher = FxHasher::default();
    PLUGIN_NAME.hash(&mut hasher);
    options.hash(&mut hasher);
    let options_hash = hasher.finish();
    Self::new_inner(options, options_hash)
  }
}

#[plugin_hook(CompilerCompilation for SwcJsMinimizerRspackPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for SwcJsMinimizerRspackPlugin)]
async fn js_chunk_hash(
  &self,
  _compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHasher,
) -> Result<()> {
  rspack_hash::RspackHash::hash(&self.options, hasher);
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for SwcJsMinimizerRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let options = &self.options;
  let minimizer_options = &self.options.minimizer_options;

  let new_cache = (compilation.options.experiments.new_cache.minimize
    && !matches!(&compilation.options.cache, CacheOptions::Disabled))
  .then(|| compilation.get_cache(PLUGIN_NAME));
  let minimize_persistent_cache = compilation.minimize_persistent_cache.take();
  let legacy_cache_entries: Mutex<Vec<(MinimizeCacheKey, CachedMinimizeEntry)>> =
    Mutex::new(Vec::new());
  let logger = compilation.get_logger(PLUGIN_NAME);
  let minimize_cache_counter = (new_cache.is_some() || minimize_persistent_cache.is_some())
    .then(|| logger.cache("minimize persistent cache"));

  let (tx, rx) = mpsc::channel::<Vec<Diagnostic>>();
  // collect all extracted comments info
  let all_extracted_comments = Mutex::new(FxHashMap::default());
  let extract_comments_condition = options.extract_comments.as_ref().map(|extract_comment| {
    RspackRegex::with_flags(
      extract_comment.condition.as_ref(),
      extract_comment.condition_flags.as_ref(),
    )
    .unwrap_or_else(|_| {
      panic!(
        "`/{}/{}` is invalid extractComments condition",
        extract_comment.condition, extract_comment.condition_flags
      )
    })
  });
  let enter_span = tracing::Span::current();

  let tls: ThreadLocal<ObjectPool> = ThreadLocal::new();
  compilation
    .assets_mut()
    .par_iter_mut()
    .filter(|(filename, original)| {
      // propagate span in rayon to keep parent relation
      let is_matched = match_object(options, filename);

      if !is_matched || original.get_info().minimized.unwrap_or(false) {
        return false
      }

      true
    })
    .try_for_each_with(tx,|tx, (asset_filename, original)| -> Result<()>  {
      let _guard = enter_span.enter();
      let filename = asset_filename.split('?').next().expect("Should have filename");
      if let Some(original_source) = original.get_source() {
        let is_module = get_is_module(minimizer_options, original, filename);

        let new_cache_entry = if let Some(cache) = &new_cache {
          let etag = Etag::from(format!(
            "{:016x}",
            minimize_cache_hash(original_source, self.options_hash, filename, is_module)
          ));
          let value = cache.get::<CachedMinimizeEntry>(asset_filename, Some(etag.clone()))?;
          Some((cache, etag, value))
        } else {
          None
        };

        let mut cache_key = None;
        let cached = if let Some((_, _, cached)) = &new_cache_entry {
          cached.as_deref()
        } else if let Some(cache) = &minimize_persistent_cache {
          let key = MinimizeCacheKey::new(minimize_cache_hash(
            original_source,
            self.options_hash,
            filename,
            is_module,
          ));
          cache_key = Some(key);
          cache.get(key)
        } else {
          None
        };
        if let Some(cached) = cached {
          if let Some(counter) = &minimize_cache_counter {
            counter.hit();
          }
          original.set_source(Some(cached.source.clone()));
          original.get_info_mut().minimized.replace(true);
          if let Some(ec) = &cached.extracted_comments {
            all_extracted_comments
              .lock()
              .expect("all_extract_comments lock failed")
              .insert(
                filename.to_string(),
                ExtractedCommentsInfo {
                  source: ec.source.clone(),
                  comments_file_name: ec.comments_file_name.clone(),
                },
              );
          }
          return Ok(());
        }
        if (new_cache_entry.is_some() || cache_key.is_some()) && let Some(counter) = &minimize_cache_counter {
          counter.miss();
        }
        let input = original_source.source().into_string_lossy().into_owned();
        let object_pool = tls.get_or(ObjectPool::default);
        let input_source_map =
          Source::map_static(original_source.clone(), object_pool, &MapOptions::default());

        let js_minify_options = rspack_javascript_compiler::minify::JsMinifyOptions {
          minify: minimizer_options.minify.unwrap_or(true),
          compress: minimizer_options.compress.clone(),
          mangle: minimizer_options.mangle.clone(),
          format: minimizer_options.format.clone(),
          ecma: minimizer_options.ecma.clone(),
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
              Some(format!("/*! LICENSE: {relative} */"))
            },
            OptionWrapper::Disabled => None,
            OptionWrapper::Custom(value) => Some(format!("/*! {value} */"))
          };
          NormalizedExtractComments {
            filename: comments_filename,
            condition: extract_comments_condition.as_ref().expect("must exist").clone(),
            banner
          }
        });

        let javascript_compiler = JavaScriptCompiler::new();
        let comments_op = |comments: &SingleThreadedComments| {
          if let Some(ref extract_comments) = extract_comments_option {
            let mut extracted_comments = vec![];
            // add all matched comments to source

            let (leading_trivial, trailing_trivial) = comments.borrow_all();

            leading_trivial.iter().for_each(|(_, comments)| {
              comments.iter().for_each(|c| {
                if extract_comments.condition.test(&c.text) {
                  let comment = match c.kind {
                    CommentKind::Line => {
                      format!("//{}", c.text)
                    }
                    CommentKind::Block => {
                      format!("/*{}*/", c.text)
                    }
                  };
                  if !extracted_comments.contains(&comment) {
                    extracted_comments.push(comment);
                  }
                }
              });
            });
            trailing_trivial.iter().for_each(|(_, comments)| {
              comments.iter().for_each(|c| {
                if extract_comments.condition.test(&c.text) {
                  let comment = match c.kind {
                    CommentKind::Line => {
                      format!("//{}", c.text)
                    }
                    CommentKind::Block => {
                      format!("/*{}*/", c.text)
                    }
                  };
                  if !extracted_comments.contains(&comment) {
                    extracted_comments.push(comment);
                  }
                }
              });
            });

            // if not matched comments, we don't need to emit .License.txt file
            if !extracted_comments.is_empty() {
              extracted_comments.sort();
              all_extracted_comments
                .lock()
                .expect("all_extract_comments lock failed")
                .insert(
                  filename.to_string(),
                  ExtractedCommentsInfo {
                    source: RawStringSource::from(extracted_comments.join("\n\n")).boxed(),
                    comments_file_name: extract_comments.filename.clone(),
                  },
                );
            }
          }
        };

        let mut output = match javascript_compiler.minify(
          swc_core::common::FileName::Custom(filename.to_string()),
          input,
          js_minify_options,
          Some(comments_op),
        ) {
            Ok(r) => r,
            Err(e) => {
              let errors = e.into_inner().into_iter().map(|err| {
                let mut d = Diagnostic::from(MinifyError(err));
                d.file = Some(filename.into());
                d
              }).collect::<Vec<_>>();
              tx.send(errors)?;
              return Ok(())
            },
        };

        let banner = if all_extracted_comments
          .lock()
          .expect("all_extract_comments lock failed")
          .contains_key(filename) {
            extract_comments_option.and_then(|option| option.banner)
          } else {
            None
          };

        let source = match banner {
            Some(banner) => {
              // There are two cases with banner:
              // 1. There's no shebang, we just prepend the banner to the code.
              // 2. There's a shebang, we prepend the shebang, then the banner, then the code.

              let mut shebang = None;
              if output.code.starts_with("#!") {
                if let Some(line_pos) = output.code.find('\n') {
                  shebang = Some(output.code[0..line_pos + 1].to_string());
                  output.code = output.code[line_pos + 1..].to_string();
                } else {
                  // Handle shebang without newline - treat entire content as shebang
                  shebang = Some(output.code.clone());
                  output.code = String::new();
                }
              }

              let source = if let Some(source_map) = output.map {
                SourceMapSource::new(SourceMapSourceOptions {
                  value: output.code,
                  name: filename,
                  source_map,
                  original_source: None,
                  inner_source_map: input_source_map,
                  remove_original_source: true,
                })
                .boxed()
              } else {
                RawStringSource::from(output.code).boxed()
              };

              if let Some(shebang) = shebang {
                ConcatSource::new([
                  RawStringSource::from(shebang).boxed(),
                  RawStringSource::from(banner).boxed(),
                  RawStringSource::from_static("\n").boxed(),
                  source
                ]).boxed()
              } else {
                ConcatSource::new([
                  RawStringSource::from(banner).boxed(),
                  RawStringSource::from_static("\n").boxed(),
                  source
                ]).boxed()
              }
            },
            None => {
              // If there's no banner, we don't need to handle `output.code` at all.
              if let Some(source_map) = output.map {
                SourceMapSource::new(SourceMapSourceOptions {
                  value: output.code,
                  name: filename,
                  source_map,
                  original_source: None,
                  inner_source_map: input_source_map,
                  remove_original_source: true,
                })
                .boxed()
              } else {
                RawStringSource::from(output.code).boxed()
              }
            },
        };

        if new_cache_entry.is_some() || cache_key.is_some() {
          let extracted_comments_for_cache = all_extracted_comments
            .lock()
            .expect("all_extract_comments lock failed")
            .get(filename)
            .map(|ec| CachedExtractedComments {
              source: ec.source.clone(),
              comments_file_name: ec.comments_file_name.clone(),
            });
          let entry = CachedMinimizeEntry {
            source: source.clone(),
            extracted_comments: extracted_comments_for_cache,
          };
          if let Some((cache, etag, _)) = &new_cache_entry {
            cache.store(asset_filename, Some(etag.clone()), CacheValue::new(entry))?;
          } else if let Some(cache_key) = cache_key {
            legacy_cache_entries
              .lock()
              .expect("legacy_cache_entries lock failed")
              .push((cache_key, entry));
          }
        }

        original.set_source(Some(source));
        original.get_info_mut().minimized.replace(true);
      }

      Ok(())
  })?;

  if let Some(mut cache) = minimize_persistent_cache {
    for (key, entry) in legacy_cache_entries
      .into_inner()
      .expect("legacy_cache_entries lock failed")
    {
      cache.insert(key, entry);
    }
    compilation.minimize_persistent_cache = Some(cache);
  }

  if let Some(counter) = minimize_cache_counter {
    logger.cache_end(counter);
  }

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
        CompilationAsset::new(
          Some(comments.source),
          AssetInfo {
            minimized: Some(true),
            ..Default::default()
          },
        ),
      )
    });

  Ok(())
}

fn minimize_cache_hash(
  source: &BoxSource,
  options_hash: u64,
  filename: &str,
  is_module: Option<bool>,
) -> u64 {
  use std::hash::Hash;

  let mut hasher = FxHasher::default();
  source.buffer().hash(&mut hasher);
  options_hash.hash(&mut hasher);
  filename.hash(&mut hasher);
  is_module.hash(&mut hasher);
  hasher.finish()
}

fn get_is_module(
  minimizer_options: &MinimizerOptions,
  asset: &CompilationAsset,
  filename: &str,
) -> Option<bool> {
  minimizer_options
    .module
    .or(asset.info.javascript_module)
    .or_else(|| {
      filename
        .ends_with(".mjs")
        .then_some(true)
        .or_else(|| filename.ends_with(".cjs").then_some(false))
    })
}

pub fn match_object(obj: &PluginOptions, str: &str) -> bool {
  if let Some(condition) = &obj.test {
    if !condition.try_match(str) {
      return false;
    }
  } else if !JAVASCRIPT_ASSET_REGEXP.is_match(str) {
    return false;
  }
  if let Some(condition) = &obj.include
    && !condition.try_match(str)
  {
    return false;
  }
  if let Some(condition) = &obj.exclude
    && condition.try_match(str)
  {
    return false;
  }

  true
}

impl Plugin for SwcJsMinimizerRspackPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
