use std::{
  collections::HashMap,
  hash::Hash,
  path::Path,
  sync::{LazyLock, Mutex, mpsc},
};

use cow_utils::CowUtils;
use once_cell::sync::OnceCell;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  AssetInfo, ChunkUkey, Compilation, CompilationAsset, CompilationParams, CompilationProcessAssets,
  CompilerCompilation, Plugin,
  diagnostics::MinifyError,
  rspack_sources::{
    ConcatSource, MapOptions, ObjectPool, RawStringSource, Source, SourceExt, SourceMapSource,
    SourceMapSourceOptions,
  },
};
use rspack_error::{Diagnostic, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_javascript_compiler::JavaScriptCompiler;
use rspack_plugin_javascript::{ExtractedCommentsInfo, JavascriptModulesChunkHash, JsPlugin};
use rspack_util::asset_condition::AssetConditions;
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
    .try_for_each_with(tx,|tx, (filename, original)| -> Result<()>  {
      let _guard = enter_span.enter();
      let filename = filename.split('?').next().expect("Should have filename");
      if let Some(original_source) = original.get_source() {
        let input = original_source.source().into_string_lossy().into_owned();
        let object_pool = tls.get_or(ObjectPool::default);
        let input_source_map = original_source.map(object_pool, &MapOptions::default());

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
            condition: extract_comments_condition.as_ref().expect("must exists"),
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
                if extract_comments.condition.is_match(&c.text) {
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
                if extract_comments.condition.is_match(&c.text) {
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
