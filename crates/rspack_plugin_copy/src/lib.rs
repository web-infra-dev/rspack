#![feature(let_chains)]
use std::{
  borrow::Cow,
  fmt::Display,
  fs,
  hash::Hash,
  ops::DerefMut,
  path::{Path, PathBuf, MAIN_SEPARATOR},
  sync::{Arc, LazyLock, Mutex},
};

use dashmap::DashSet;
use derive_more::Debug;
use futures::future::BoxFuture;
use glob::{MatchOptions, Pattern as GlobPattern};
use regex::Regex;
use rspack_core::{
  rspack_sources::{RawSource, Source},
  AssetInfo, AssetInfoRelated, Compilation, CompilationAsset, CompilationLogger,
  CompilationProcessAssets, FilenameTemplate, Logger, PathData, Plugin,
};
use rspack_error::{Diagnostic, DiagnosticError, Error, ErrorExt, Result};
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash, RspackHashDigest};
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
use rspack_util::infallible::ResultInfallibleExt as _;
use sugar_path::SugarPath;

#[derive(Debug)]
pub struct CopyRspackPluginOptions {
  pub patterns: Vec<CopyPattern>,
}

#[derive(Debug, Clone)]
pub struct Info {
  pub immutable: Option<bool>,
  pub minimized: Option<bool>,
  pub chunk_hash: Option<Vec<String>>,
  pub content_hash: Option<Vec<String>>,
  pub development: Option<bool>,
  pub hot_module_replacement: Option<bool>,
  pub related: Option<Related>,
  pub version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Related {
  pub source_map: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum FromType {
  Dir,
  File,
  Glob,
}

#[derive(Debug, Clone)]
pub enum ToType {
  Dir,
  File,
  Template,
}

impl Display for ToType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      ToType::Dir => "dir",
      ToType::File => "file",
      ToType::Template => "template",
    })
  }
}

pub type TransformerFn =
  Box<dyn for<'a> Fn(Vec<u8>, &'a str) -> BoxFuture<'a, Result<RawSource>> + Sync + Send>;

pub enum Transformer {
  Fn(TransformerFn),
}

pub struct ToFnCtx<'a> {
  pub context: &'a Utf8Path,
  pub absolute_filename: &'a Utf8Path,
}

pub type ToFn = Box<dyn for<'a> Fn(ToFnCtx<'a>) -> BoxFuture<'a, Result<String>> + Sync + Send>;

pub enum ToOption {
  String(String),
  Fn(ToFn),
}

#[derive(Debug)]
pub struct CopyPattern {
  pub from: String,
  #[debug(skip)]
  pub to: Option<ToOption>,
  pub context: Option<Utf8PathBuf>,
  pub to_type: Option<ToType>,
  pub no_error_on_missing: bool,
  pub info: Option<Info>,
  pub force: bool,
  pub priority: i32,
  pub glob_options: CopyGlobOptions,
  #[debug(skip)]
  pub transform: Option<Transformer>,
}

#[derive(Debug, Clone)]
pub struct CopyGlobOptions {
  pub case_sensitive_match: Option<bool>,
  pub dot: Option<bool>,
  pub ignore: Option<Vec<GlobPattern>>,
}

#[derive(Debug, Clone)]
pub struct RunPatternResult {
  pub source_filename: Utf8PathBuf,
  pub absolute_filename: Utf8PathBuf,
  pub filename: String,
  pub source: RawSource,
  pub info: Option<Info>,
  pub force: bool,
  pub priority: i32,
}

#[plugin]
#[derive(Debug)]
pub struct CopyRspackPlugin {
  pub patterns: Vec<CopyPattern>,
}

static TEMPLATE_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\[\\*([\w:]+)\\*\]").expect("This never fail"));

impl CopyRspackPlugin {
  pub fn new(patterns: Vec<CopyPattern>) -> Self {
    Self::new_inner(patterns)
  }

  fn get_content_hash(
    source: &RawSource,
    function: &HashFunction,
    digest: &HashDigest,
    salt: &HashSalt,
  ) -> RspackHashDigest {
    let mut hasher = RspackHash::with_salt(function, salt);
    source.buffer().hash(&mut hasher);
    hasher.digest(digest)
  }

  #[allow(clippy::too_many_arguments)]
  async fn analyze_every_entry(
    entry: Utf8PathBuf,
    pattern: &CopyPattern,
    context: &Utf8Path,
    output_path: &Utf8Path,
    from_type: FromType,
    file_dependencies: &DashSet<PathBuf>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    compilation: &Compilation,
    logger: &CompilationLogger,
  ) -> Option<RunPatternResult> {
    // Exclude directories
    if entry.is_dir() {
      return None;
    }
    if let Some(ignore) = &pattern.glob_options.ignore
      && ignore.iter().any(|ignore| ignore.matches(entry.as_str()))
    {
      return None;
    }

    let from = entry;

    logger.debug(format!("found '{from}'"));

    let absolute_filename = if from.is_absolute() {
      from.clone()
    } else {
      context.join(&from)
    };

    let to = if let Some(to) = pattern.to.as_ref() {
      let to = match to {
        ToOption::String(s) => s.to_owned(),
        ToOption::Fn(r) => {
          let to_result = r(ToFnCtx {
            context,
            absolute_filename: &absolute_filename,
          })
          .await;
          let to = match to_result {
            Ok(to) => to,
            Err(e) => {
              diagnostics
                .lock()
                .expect("failed to obtain lock of `diagnostics`")
                .push(Diagnostic::error(
                  "Run copy to fn error".into(),
                  e.to_string(),
                ));
              "".to_string()
            }
          };
          to
        }
      };

      to.clone()
        .as_path()
        .normalize()
        .to_string_lossy()
        .to_string()
    } else {
      "".into()
    };

    let to_type = if let Some(to_type) = pattern.to_type.as_ref() {
      to_type.clone()
    } else if TEMPLATE_RE.is_match(&to) {
      ToType::Template
    } else if Path::new(&to).extension().is_none() || to.ends_with(MAIN_SEPARATOR) {
      ToType::Dir
    } else {
      ToType::File
    };

    logger.log(format!("'to' option '{to}' determined as '{to_type}'"));

    let relative = pathdiff::diff_utf8_paths(&absolute_filename, context);
    let filename = if matches!(to_type, ToType::Dir) {
      if let Some(relative) = &relative {
        Utf8PathBuf::from(&to).join(relative)
      } else {
        to.into()
      }
    } else {
      to.into()
    };

    let filename = if filename.is_absolute() {
      pathdiff::diff_utf8_paths(filename, output_path)?
    } else {
      filename
    };

    logger.log(format!(
      "determined that '{from}' should write to '{filename}'"
    ));

    let source_filename = relative?;

    // If this came from a glob or dir, add it to the file dependencies
    if matches!(from_type, FromType::Dir | FromType::Glob) {
      logger.debug(format!("added '{absolute_filename}' as a file dependency",));

      file_dependencies.insert(absolute_filename.clone().into_std_path_buf());
    }

    // TODO cache

    logger.debug(format!("reading '{}'...", absolute_filename));
    // TODO inputFileSystem

    #[cfg(not(target_family = "wasm"))]
    let data = tokio::fs::read(absolute_filename.clone()).await;
    #[cfg(target_family = "wasm")]
    let data = std::fs::read(absolute_filename.clone());

    let source_vec = match data {
      Ok(data) => {
        logger.debug(format!("read '{}'...", absolute_filename));

        data
      }
      Err(e) => {
        let e: Error = DiagnosticError::from(e.boxed()).into();
        diagnostics
          .lock()
          .expect("failed to obtain lock of `diagnostics`")
          .push(e.into());
        return None;
      }
    };

    let mut source = RawSource::from(source_vec.clone());

    if let Some(transform) = &pattern.transform {
      match transform {
        Transformer::Fn(transformer) => {
          let transformed = transformer(source_vec, absolute_filename.as_str()).await;
          match transformed {
            Ok(code) => {
              source = code;
            }
            Err(e) => {
              diagnostics
                .lock()
                .expect("failed to obtain lock of `diagnostics`")
                .push(Diagnostic::error(
                  "Run copy transform fn error".into(),
                  e.to_string(),
                ));
            }
          };
        }
      }
    }

    let filename = if matches!(&to_type, ToType::Template) {
      logger.log(format!(
        "interpolating template '{}' for '${}'...`",
        filename, source_filename
      ));

      let content_hash = Self::get_content_hash(
        &source,
        &compilation.options.output.hash_function,
        &compilation.options.output.hash_digest,
        &compilation.options.output.hash_salt,
      );
      let content_hash = content_hash.rendered(compilation.options.output.hash_digest_length);
      let template_str = compilation
        .get_asset_path(
          &FilenameTemplate::from(filename.to_string()),
          PathData::default()
            .filename(source_filename.as_str())
            .content_hash(content_hash)
            .hash_optional(compilation.get_hash()),
        )
        .always_ok();

      logger.log(format!(
        "interpolated template '{template_str}' for '{}'",
        filename
      ));

      template_str
    } else {
      filename.as_str().normalize().to_string_lossy().to_string()
    };

    Some(RunPatternResult {
      source_filename,
      absolute_filename,
      filename,
      source,
      info: pattern.info.clone(),
      force: pattern.force,
      priority: pattern.priority,
    })
  }

  fn run_patter(
    compilation: &Compilation,
    pattern: &CopyPattern,
    _index: usize,
    file_dependencies: &DashSet<PathBuf>,
    context_dependencies: &DashSet<PathBuf>,
    diagnostics: &Mutex<Vec<Diagnostic>>,
    logger: &CompilationLogger,
  ) -> Option<Vec<Option<RunPatternResult>>> {
    let orig_from = &pattern.from;
    let normalized_orig_from = Utf8PathBuf::from(orig_from);

    let pattern_context = if pattern.context.is_none() {
      Some(Cow::Borrowed(compilation.options.context.as_path()))
    } else if let Some(ref ctx) = pattern.context
      && !ctx.is_absolute()
    {
      Some(Cow::Owned(compilation.options.context.as_path().join(ctx)))
    } else {
      pattern.context.as_deref().map(Into::into)
    };

    logger.log(format!(
      "starting to process a pattern from '{}' using '{:?}' context",
      normalized_orig_from, pattern_context
    ));

    let mut context =
      pattern_context.unwrap_or_else(|| Cow::Borrowed(compilation.options.context.as_path()));

    let abs_from = if normalized_orig_from.is_absolute() {
      normalized_orig_from
    } else {
      context.join(&normalized_orig_from)
    };

    logger.debug(format!("getting stats for '{}'...", abs_from));

    let from_type = if let Ok(meta) = fs::metadata(&abs_from) {
      if meta.is_dir() {
        logger.debug(format!("determined '{}' is a directory", abs_from));
        FromType::Dir
      } else if meta.is_file() {
        logger.debug(format!("determined '{}' is a file", abs_from));
        FromType::File
      } else {
        logger.debug(format!("determined '{}' is a unknown", abs_from));
        FromType::Glob
      }
    } else {
      logger.debug(format!("determined '{}' is a glob", abs_from));
      FromType::Glob
    };

    // Enable copy files starts with dot
    let mut dot_enable = pattern.glob_options.dot;

    /*
     * If input is a glob query like `/a/b/**/*.js`, we need to add common directory
     * to context_dependencies
     */
    let mut need_add_context_to_dependency = false;
    let glob_query = match from_type {
      FromType::Dir => {
        logger.debug(format!("added '{}' as a context dependency", abs_from));
        context_dependencies.insert(abs_from.clone().into_std_path_buf());
        context = abs_from.as_path().into();

        if dot_enable.is_none() {
          dot_enable = Some(true);
        }
        let mut escaped = Utf8PathBuf::from(GlobPattern::escape(abs_from.as_str()));
        escaped.push("**/*");

        escaped.as_str().to_string()
      }
      FromType::File => {
        logger.debug(format!("added '{}' as a file dependency", abs_from));
        file_dependencies.insert(abs_from.clone().into_std_path_buf());
        context = abs_from.parent().unwrap_or(Utf8Path::new("")).into();

        if dot_enable.is_none() {
          dot_enable = Some(true);
        }

        GlobPattern::escape(abs_from.as_str())
      }
      FromType::Glob => {
        need_add_context_to_dependency = true;
        let glob_query = if Path::new(orig_from).is_absolute() {
          orig_from.into()
        } else {
          context.join(orig_from).as_str().to_string()
        };
        // A glob pattern ending with /** should match all files within a directory, not just the directory itself.
        // Since the standard glob only matches directories, we append /* to align with webpack's behavior.
        if glob_query.ends_with("/**") {
          format!("{glob_query}/*")
        } else {
          glob_query
        }
      }
    };

    logger.log(format!("begin globbing '{glob_query}'..."));

    let glob_entries = glob::glob_with(
      &glob_query,
      MatchOptions {
        case_sensitive: pattern.glob_options.case_sensitive_match.unwrap_or(true),
        require_literal_separator: Default::default(),
        require_literal_leading_dot: !dot_enable.unwrap_or(false),
      },
    );

    match glob_entries {
      Ok(entries) => {
        let entries: Vec<_> = entries
          .filter_map(|entry| {
            let entry = entry.ok()?.assert_utf8();

            let filters = pattern.glob_options.ignore.as_ref();

            if let Some(filters) = filters {
              // If filters length is 0, exist is true by default
              let exist = filters.iter().all(|filter| !filter.matches(entry.as_str()));
              exist.then_some(entry)
            } else {
              Some(entry)
            }
          })
          .collect();

        if need_add_context_to_dependency
          && let Some(common_dir) = get_closest_common_parent_dir(
            &entries.iter().map(|it| it.as_path()).collect::<Vec<_>>(),
          )
        {
          context_dependencies.insert(common_dir.into_std_path_buf());
        }

        if entries.is_empty() {
          if pattern.no_error_on_missing {
            logger.log(
              "finished to process a pattern from '${normalizedOriginalFrom}' using '${pattern.context}' context to '${pattern.to}'"
            );
            return None;
          }

          diagnostics
            .lock()
            .expect("failed to obtain lock of `diagnostics`")
            .push(Diagnostic::error(
              "CopyRspackPlugin Error".into(),
              format!("unable to locate '{glob_query}' glob"),
            ));
        }

        let output_path = &compilation.options.output.path;

        let copied_result = entries
          .into_iter()
          .map(|entry| async {
            Self::analyze_every_entry(
              entry,
              pattern,
              &context,
              output_path,
              from_type,
              file_dependencies,
              diagnostics,
              compilation,
              logger,
            )
            .await
          })
          .collect::<rspack_futures::FuturesResults<Option<RunPatternResult>>>();

        if copied_result.is_empty() {
          if pattern.no_error_on_missing {
            return None;
          }

          // TODO err handler
          diagnostics
            .lock()
            .expect("failed to obtain lock of `diagnostics`")
            .push(Diagnostic::error(
              "CopyRspackPlugin Error".into(),
              format!("unable to locate '{glob_query}' glob"),
            ));
          return None;
        }

        Some(copied_result.into_inner())
      }
      Err(e) => {
        if pattern.no_error_on_missing {
          let to = if let Some(to) = &pattern.to {
            match to {
              ToOption::String(s) => s,
              ToOption::Fn(_) => "",
            }
          } else {
            ""
          };

          logger.log(format!(
            "finished to process a pattern from '{}' using '{}' context to '{:?}'",
            Utf8PathBuf::from(orig_from),
            context,
            to,
          ));

          return None;
        }

        diagnostics
          .lock()
          .expect("failed to obtain lock of `diagnostics`")
          .push(Diagnostic::error("Glob Error".into(), e.msg.to_string()));

        None
      }
    }
  }
}

#[plugin_hook(CompilationProcessAssets for CopyRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.CopyRspackPlugin");
  let start = logger.time("run pattern");
  let file_dependencies = DashSet::default();
  let context_dependencies = DashSet::default();
  let diagnostics = Mutex::new(Vec::new());

  let mut copied_result: Vec<(i32, RunPatternResult)> = self
    .patterns
    .iter()
    .enumerate()
    .map(|(index, pattern)| {
      CopyRspackPlugin::run_patter(
        compilation,
        pattern,
        index,
        &file_dependencies,
        &context_dependencies,
        &diagnostics,
        &logger,
      )
    })
    .collect::<Vec<_>>()
    .into_iter()
    .flatten()
    .flat_map(|item| {
      item
        .into_iter()
        .flatten()
        .map(|item| (item.priority, item))
        .collect::<Vec<_>>()
    })
    .collect();
  logger.time_end(start);

  let start = logger.time("emit assets");
  compilation
    .file_dependencies
    .extend(file_dependencies.into_iter().map(Into::into));
  compilation
    .context_dependencies
    .extend(context_dependencies.into_iter().map(Into::into));
  compilation.extend_diagnostics(std::mem::take(
    diagnostics
      .lock()
      .expect("failed to obtain lock of `diagnostics`")
      .deref_mut(),
  ));

  copied_result.sort_unstable_by(|a, b| a.0.cmp(&b.0));
  copied_result.into_iter().for_each(|(_priority, result)| {
    if let Some(exist_asset) = compilation.assets_mut().get_mut(&result.filename) {
      if !result.force {
        return;
      }
      exist_asset.set_source(Some(Arc::new(result.source)));
      if let Some(info) = result.info {
        set_info(&mut exist_asset.info, info);
      }
      exist_asset.info.source_filename = Some(result.source_filename.to_string());
      exist_asset.info.copied = Some(true);
    } else {
      let mut asset_info = AssetInfo {
        source_filename: Some(result.source_filename.to_string()),
        copied: Some(true),
        ..Default::default()
      };

      if let Some(info) = result.info {
        set_info(&mut asset_info, info);
      }

      compilation.emit_asset(
        result.filename,
        CompilationAsset {
          source: Some(Arc::new(result.source)),
          info: asset_info,
        },
      )
    }
  });
  logger.time_end(start);

  Ok(())
}

impl Plugin for CopyRspackPlugin {
  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}

fn get_closest_common_parent_dir(paths: &[&Utf8Path]) -> Option<Utf8PathBuf> {
  // If there are no matching files, return `None`.
  if paths.is_empty() {
    return None;
  }

  // Get the first file path and use it as the initial value for the common parent directory.
  let mut parent_dir: Utf8PathBuf = paths[0].parent()?.to_path_buf();

  // Iterate over the remaining file paths, updating the common parent directory as necessary.
  for path in paths.iter().skip(1) {
    // Find the common parent directory between the current file path and the previous common parent directory.
    while !path.starts_with(&parent_dir) {
      parent_dir = parent_dir.parent()?.into();
    }
  }

  Some(parent_dir)
}

fn set_info(target: &mut AssetInfo, info: Info) {
  if let Some(minimized) = info.minimized {
    target.minimized.replace(minimized);
  }

  if let Some(immutable) = info.immutable {
    target.immutable.replace(immutable);
  }

  if let Some(chunk_hash) = info.chunk_hash {
    target.chunk_hash = rustc_hash::FxHashSet::from_iter(chunk_hash);
  }

  if let Some(content_hash) = info.content_hash {
    target.content_hash = rustc_hash::FxHashSet::from_iter(content_hash);
  }

  if let Some(development) = info.development {
    target.development.replace(development);
  }

  if let Some(hot_module_replacement) = info.hot_module_replacement {
    target
      .hot_module_replacement
      .replace(hot_module_replacement);
  }

  if let Some(related) = info.related {
    target.related = AssetInfoRelated {
      source_map: related.source_map,
    };
  }

  if let Some(version) = info.version {
    target.version = version;
  }
}

// If this test fails, you should modify `set_info` function, according to your changes about AssetInfo
// Make sure every field of AssetInfo is considered
#[test]
fn ensure_info_fields() {
  let info = AssetInfo::default();
  std::hint::black_box(info);
}
