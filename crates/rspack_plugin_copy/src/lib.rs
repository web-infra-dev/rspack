use std::{
  borrow::Cow,
  fmt::Display,
  hash::Hash,
  ops::DerefMut,
  path::{MAIN_SEPARATOR, Path, PathBuf},
  sync::{Arc, LazyLock, Mutex},
};

use cow_utils::CowUtils;
use derive_more::Debug;
use fast_glob::glob_match;
use futures::{StreamExt, future::BoxFuture, stream::FuturesOrdered};
use regex::Regex;
use rspack_core::{
  AssetInfo, AssetInfoRelated, Compilation, CompilationAsset, CompilationLogger,
  CompilationProcessAssets, Filename, GlobMatchOptions, Logger, PathData, Plugin,
  escape_glob_pattern, extract_glob_base_dir, find_files_by_glob,
  rspack_sources::{BoxSource, RawBufferSource, SourceExt},
  unescape_glob_path,
};
use rspack_error::{Diagnostic, Error, Result};
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHashDigest, RspackHasher};
use rspack_hook::{plugin, plugin_hook};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::fx_hash::FxDashSet;
use sugar_path::SugarPath;

mod pattern_cache;

use pattern_cache::CachedPatternResult;

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
  Box<dyn for<'a> Fn(Vec<u8>, &'a str) -> BoxFuture<'a, Result<BoxSource>> + Sync + Send>;

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
  pub copy_permissions: Option<bool>,
  #[debug(skip)]
  pub transform_fn: Option<TransformerFn>,
  pub cache: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct CopyGlobOptions {
  pub case_sensitive_match: Option<bool>,
  pub dot: Option<bool>,
  pub ignore: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct RunPatternResult {
  pub source_filename: Utf8PathBuf,
  pub absolute_filename: Utf8PathBuf,
  pub filename: String,
  pub source: BoxSource,
  pub info: Option<Info>,
  pub force: bool,
}

#[plugin]
#[derive(Debug)]
pub struct CopyRspackPlugin {
  pub patterns: Vec<CopyPattern>,
  pattern_cache: Mutex<Vec<Option<CachedPatternResult>>>,
}

struct PendingPattern<'a> {
  index: usize,
  pattern: &'a CopyPattern,
  cacheable: bool,
  file_dependencies: FxDashSet<PathBuf>,
  context_dependencies: FxDashSet<PathBuf>,
  diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
}

static TEMPLATE_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"\[\\*([\w:]+)\\*\]").expect("This never fail"));

fn normalize_glob_path_separators(path: &str) -> Cow<'_, str> {
  if cfg!(windows) {
    path.cow_replace('\\', "/")
  } else {
    Cow::Borrowed(path)
  }
}

impl CopyRspackPlugin {
  pub fn new(patterns: Vec<CopyPattern>) -> Self {
    let pattern_cache = Mutex::new(vec![None; patterns.len()]);
    Self::new_inner(patterns, pattern_cache)
  }

  fn is_cacheable(pattern: &CopyPattern) -> bool {
    pattern.transform_fn.is_none()
      && !matches!(pattern.to, Some(ToOption::Fn(_)))
      && !pattern.copy_permissions.unwrap_or(false)
      && !matches!(pattern.to_type, Some(ToType::Template))
      && !matches!(pattern.to, Some(ToOption::String(ref to)) if TEMPLATE_RE.is_match(to))
  }

  fn get_content_hash(
    source: &BoxSource,
    function: &HashFunction,
    digest: &HashDigest,
    salt: &HashSalt,
  ) -> RspackHashDigest {
    let mut hasher = RspackHasher::with_salt(function, salt);
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
    file_dependencies: &FxDashSet<PathBuf>,
    diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
    compilation: &Compilation,
    logger: &CompilationLogger,
  ) -> Result<Option<RunPatternResult>> {
    // Exclude directories
    if entry.is_dir() {
      return Ok(None);
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
          let result = r(ToFnCtx {
            context,
            absolute_filename: &absolute_filename,
          })
          .await;

          match result {
            Ok(to) => to,
            Err(e) => {
              diagnostics
                .lock()
                .expect("failed to obtain lock of `diagnostics`")
                .push(Diagnostic::error(
                  "Run copy to fn error".into(),
                  e.to_string(),
                ));
              String::new()
            }
          }
        }
      };

      to.as_path().normalize().to_string_lossy().to_string()
    } else {
      String::new()
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
      if let Some(filename) = pathdiff::diff_utf8_paths(filename, output_path) {
        filename
      } else {
        return Ok(None);
      }
    } else {
      filename
    };

    logger.log(format!(
      "determined that '{from}' should write to '{filename}'"
    ));

    let Some(source_filename) = relative else {
      return Ok(None);
    };

    // If this came from a glob or dir, add it to the file dependencies
    if matches!(from_type, FromType::Dir | FromType::Glob) {
      logger.debug(format!("added '{absolute_filename}' as a file dependency",));

      file_dependencies.insert(
        absolute_filename
          .to_path_buf()
          .into_std_path_buf()
          .normalize()
          .into_owned(),
      );
    }

    // TODO cache

    logger.debug(format!("reading '{absolute_filename}'..."));
    // TODO inputFileSystem

    let data = compilation.input_filesystem.read(&absolute_filename).await;

    let source_vec = match data {
      Ok(data) => {
        logger.debug(format!("read '{absolute_filename}'..."));

        data
      }
      Err(e) => {
        let e: Error = e.into();
        diagnostics
          .lock()
          .expect("failed to obtain lock of `diagnostics`")
          .push(e.into());
        return Ok(None);
      }
    };

    let source = if let Some(transformer) = &pattern.transform_fn {
      let mut source = RawBufferSource::from(source_vec.clone()).boxed();
      logger.debug(format!("transforming content for '{absolute_filename}'..."));
      handle_transform(
        transformer,
        source_vec,
        absolute_filename.clone(),
        &mut source,
        diagnostics,
      )
      .await;
      source
    } else {
      RawBufferSource::from(source_vec).boxed()
    };

    let filename = if matches!(&to_type, ToType::Template) {
      logger.log(format!(
        "interpolating template '{filename}' for '${source_filename}'...`"
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
          &Filename::from(filename.to_string()),
          PathData::default()
            .filename(source_filename.as_str())
            .content_hash(content_hash)
            .hash_optional(compilation.get_hash()),
        )
        .await?;

      logger.log(format!(
        "interpolated template '{template_str}' for '{filename}'"
      ));

      template_str
    } else {
      filename.as_str().normalize().to_string_lossy().to_string()
    };
    let filename = normalize_glob_path_separators(&filename).into_owned();

    Ok(Some(RunPatternResult {
      source_filename,
      absolute_filename,
      filename,
      source,
      info: pattern.info.clone(),
      force: pattern.force,
    }))
  }

  async fn run_pattern(
    compilation: &Compilation,
    pattern: &CopyPattern,
    file_dependencies: &FxDashSet<PathBuf>,
    context_dependencies: &FxDashSet<PathBuf>,
    diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
    logger: &CompilationLogger,
  ) -> Result<Option<Vec<RunPatternResult>>> {
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
      "starting to process a pattern from '{normalized_orig_from}' using '{pattern_context:?}' context"
    ));

    let mut context =
      pattern_context.unwrap_or_else(|| Cow::Borrowed(compilation.options.context.as_path()));

    let abs_from = if normalized_orig_from.is_absolute() {
      normalized_orig_from
    } else {
      context.join(&normalized_orig_from)
    };

    logger.debug(format!("getting stats for '{abs_from}'..."));

    let from_type = if let Ok(meta) = compilation.input_filesystem.metadata(&abs_from).await {
      if meta.is_directory {
        logger.debug(format!("determined '{abs_from}' is a directory"));
        FromType::Dir
      } else if meta.is_file {
        logger.debug(format!("determined '{abs_from}' is a file"));
        FromType::File
      } else {
        logger.debug(format!("determined '{abs_from}' is a unknown"));
        FromType::Glob
      }
    } else {
      logger.debug(format!("determined '{abs_from}' is a glob"));
      FromType::Glob
    };

    // Enable copy files starts with dot
    let mut dot_enable = pattern.glob_options.dot;

    let glob_query = match from_type {
      FromType::Dir => {
        logger.debug(format!("added '{abs_from}' as a context dependency"));
        context_dependencies.insert(abs_from.clone().into_std_path_buf());
        context = abs_from.as_path().into();

        if dot_enable.is_none() {
          dot_enable = Some(true);
        }
        let from = normalize_glob_path_separators(abs_from.as_str());
        let escaped = escape_glob_pattern(&from);
        format!("{}/**/*", escaped.trim_end_matches('/'))
      }
      FromType::File => {
        logger.debug(format!("added '{abs_from}' as a file dependency"));
        file_dependencies.insert(
          abs_from
            .clone()
            .into_std_path_buf()
            .normalize()
            .into_owned(),
        );
        context = abs_from.parent().unwrap_or(Utf8Path::new("")).into();

        if dot_enable.is_none() {
          dot_enable = Some(true);
        }

        let from = normalize_glob_path_separators(abs_from.as_str());
        escape_glob_pattern(&from)
      }
      FromType::Glob => {
        let mut glob_query = if Path::new(orig_from).is_absolute() {
          orig_from.into()
        } else {
          context.join(orig_from).as_str().to_string()
        };
        if cfg!(windows) {
          glob_query = glob_query.cow_replace('\\', "/").into_owned();
        }
        // A glob pattern ending with /** should match all files within a directory, not just the directory itself.
        // Since the standard glob only matches directories, we append /* to align with webpack's behavior.
        if glob_query.ends_with("/**") {
          format!("{glob_query}/*")
        } else {
          glob_query
        }
      }
    };

    if matches!(from_type, FromType::Glob) {
      let glob_base_dir = unescape_glob_path(extract_glob_base_dir(&glob_query));
      context_dependencies.insert(PathBuf::from(glob_base_dir).normalize().into_owned());
    }

    logger.log(format!("begin globbing '{glob_query}'..."));

    let glob_match_options = GlobMatchOptions {
      case_sensitive: pattern.glob_options.case_sensitive_match.unwrap_or(true),
      require_literal_leading_dot: !dot_enable.unwrap_or(false),
    };

    let glob_entries = find_files_by_glob(
      &glob_query,
      &glob_match_options,
      compilation.input_filesystem.clone(),
    )
    .await;

    match glob_entries {
      Ok(mut entries) => {
        entries.retain(|entry| {
          pattern.glob_options.ignore.as_ref().is_none_or(|filters| {
            filters
              .iter()
              .all(|filter| !glob_match(filter.as_bytes(), entry.as_str().as_bytes()))
          })
        });

        if entries.is_empty() {
          if pattern.no_error_on_missing {
            logger.log(
              "finished to process a pattern from '${normalizedOriginalFrom}' using '${pattern.context}' context to '${pattern.to}'"
            );
            return Ok(None);
          }

          diagnostics
            .lock()
            .expect("failed to obtain lock of `diagnostics`")
            .push(Diagnostic::error(
              "CopyRspackPlugin Error".into(),
              format!("unable to locate '{glob_query}' glob"),
            ));
          return Ok(None);
        }

        let output_path = &compilation.options.output.path;

        let copied_result = entries
          .into_iter()
          .map(|entry| {
            Self::analyze_every_entry(
              entry,
              pattern,
              &context,
              output_path,
              from_type,
              file_dependencies,
              diagnostics.clone(),
              compilation,
              logger,
            )
          })
          .collect::<FuturesOrdered<_>>()
          .collect::<Vec<_>>()
          .await
          .into_iter()
          .collect::<Result<Vec<_>>>()?;

        if copied_result.is_empty() {
          if pattern.no_error_on_missing {
            return Ok(None);
          }

          // TODO err handler
          diagnostics
            .lock()
            .expect("failed to obtain lock of `diagnostics`")
            .push(Diagnostic::error(
              "CopyRspackPlugin Error".into(),
              format!("unable to locate '{glob_query}' glob"),
            ));
          return Ok(None);
        }

        Ok(Some(copied_result.into_iter().flatten().collect()))
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

          return Ok(None);
        }

        diagnostics
          .lock()
          .expect("failed to obtain lock of `diagnostics`")
          .push(Diagnostic::error("Glob Error".into(), e.to_string()));

        Ok(None)
      }
    }
  }

  fn emit_pattern_results(
    &self,
    compilation: &mut Compilation,
    results_by_pattern: Vec<Option<Vec<RunPatternResult>>>,
  ) -> Vec<(Utf8PathBuf, Utf8PathBuf)> {
    let mut ordered_patterns = results_by_pattern
      .into_iter()
      .enumerate()
      .collect::<Vec<_>>();
    ordered_patterns.sort_unstable_by_key(|(index, _)| (self.patterns[*index].priority, *index));

    let mut permission_copies = Vec::new();
    for (index, results) in ordered_patterns {
      let copy_permissions = self.patterns[index].copy_permissions.unwrap_or(false);
      for result in results.into_iter().flatten() {
        let permission_copy = copy_permissions.then(|| {
          (
            result.absolute_filename.clone(),
            compilation.options.output.path.join(&result.filename),
          )
        });

        if let Some(exist_asset) = compilation.assets_mut().get_mut(&result.filename) {
          if !result.force {
            continue;
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
            CompilationAsset::new(Some(Arc::new(result.source)), asset_info),
          );
        }

        if let Some(permission_copy) = permission_copy {
          permission_copies.push(permission_copy);
        }
      }
    }

    permission_copies
  }
}

#[plugin_hook(CompilationProcessAssets for CopyRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.CopyRspackPlugin");
  let start = logger.time("run pattern");
  let mut file_dependencies = Vec::new();
  let mut context_dependencies = Vec::new();
  let mut diagnostics = Vec::new();
  let cache_counter = logger.cache("copy pattern cache");

  let mut results_by_pattern = vec![None; self.patterns.len()];
  let mut pending_patterns = Vec::new();
  {
    let mut pattern_cache = self
      .pattern_cache
      .lock()
      .expect("failed to obtain lock of `pattern_cache`");
    pattern_cache.resize_with(self.patterns.len(), || None);

    for (index, pattern) in self.patterns.iter().enumerate() {
      let cacheable = CopyRspackPlugin::is_cacheable(pattern);
      let cached = &mut pattern_cache[index];

      if cacheable
        && (compilation.is_lazy_watch_rebuild
          || !compilation.modified_files.is_empty()
          || !compilation.removed_files.is_empty())
        && let Some(cached) = cached.as_ref()
        && !cached.is_invalidated(
          compilation
            .modified_files
            .iter()
            .chain(compilation.removed_files.iter())
            .map(|changed| changed.as_ref()),
        )
      {
        cache_counter.hit();
        file_dependencies.extend(cached.file_dependencies.iter().cloned());
        context_dependencies.extend(cached.context_dependencies.iter().cloned());
        results_by_pattern[index] = Some(cached.results.clone());
        continue;
      }

      cache_counter.miss();
      *cached = None;
      pending_patterns.push(PendingPattern {
        index,
        pattern,
        cacheable,
        file_dependencies: FxDashSet::default(),
        context_dependencies: FxDashSet::default(),
        diagnostics: Arc::new(Mutex::new(Vec::new())),
      });
    }
  }
  logger.cache_end(cache_counter);
  let pending_results = pending_patterns
    .iter()
    .map(|pending| {
      CopyRspackPlugin::run_pattern(
        compilation,
        pending.pattern,
        &pending.file_dependencies,
        &pending.context_dependencies,
        pending.diagnostics.clone(),
        &logger,
      )
    })
    .collect::<FuturesOrdered<_>>()
    .collect::<Vec<_>>()
    .await;

  let mut first_error = None;
  if !pending_patterns.is_empty() {
    let mut pattern_cache = self
      .pattern_cache
      .lock()
      .expect("failed to obtain lock of `pattern_cache`");
    for (pending, results) in pending_patterns.into_iter().zip(pending_results) {
      let results = match results {
        Ok(results) => results,
        Err(error) => {
          if first_error.is_none() {
            first_error = Some(error);
          }
          continue;
        }
      };
      let pattern_file_dependencies = pending.file_dependencies.into_iter().collect::<Vec<_>>();
      let pattern_context_dependencies =
        pending.context_dependencies.into_iter().collect::<Vec<_>>();
      let pattern_diagnostics = std::mem::take(
        pending
          .diagnostics
          .lock()
          .expect("failed to obtain lock of `pattern_diagnostics`")
          .deref_mut(),
      );
      let has_diagnostics = !pattern_diagnostics.is_empty();
      file_dependencies.extend(pattern_file_dependencies.iter().cloned());
      context_dependencies.extend(pattern_context_dependencies.iter().cloned());
      diagnostics.extend(pattern_diagnostics);

      if pending.cacheable
        && !has_diagnostics
        && let Some(results) = results.as_ref()
      {
        pattern_cache[pending.index] = Some(CachedPatternResult {
          results: results.clone(),
          file_dependencies: pattern_file_dependencies,
          context_dependencies: pattern_context_dependencies,
        });
      }

      results_by_pattern[pending.index] = results;
    }
  }
  if let Some(error) = first_error {
    return Err(error);
  }

  logger.time_end(start);

  let start = logger.time("emit assets");
  compilation
    .file_dependencies
    .extend(file_dependencies.into_iter().map(Into::into));
  compilation
    .context_dependencies
    .extend(context_dependencies.into_iter().map(Into::into));
  compilation.extend_diagnostics(diagnostics);

  let permission_copies = self.emit_pattern_results(compilation, results_by_pattern);
  logger.time_end(start);

  // Handle permission copying after all assets are emitted
  for (source_path, dest_path) in permission_copies.iter() {
    if let Ok(Some(permissions)) = compilation.input_filesystem.permissions(source_path).await {
      // Make sure the output directory exists
      if let Some(parent) = dest_path.parent() {
        compilation
          .output_filesystem
          .create_dir_all(parent)
          .await
          .unwrap_or_else(|e| {
            logger.warn(format!("Failed to create directory {parent:?}: {e}"));
          });
      }

      // Make sure the file exists before trying to set permissions
      if !dest_path.exists() {
        logger.warn(format!(
          "Destination file {dest_path:?} does not exist, cannot copy permissions"
        ));
        continue;
      }

      if let Err(e) = compilation
        .output_filesystem
        .set_permissions(dest_path, permissions)
        .await
      {
        logger.warn(format!(
          "Failed to copy permissions from {source_path:?} to {dest_path:?}: {e}"
        ));
      } else {
        logger.log(format!(
          "Successfully copied permissions from {source_path:?} to {dest_path:?}"
        ));
      }
    }
  }

  Ok(())
}

impl Plugin for CopyRspackPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
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

async fn handle_transform(
  transformer: &TransformerFn,
  source_vec: Vec<u8>,
  absolute_filename: Utf8PathBuf,
  source: &mut BoxSource,
  diagnostics: Arc<Mutex<Vec<Diagnostic>>>,
) {
  match transformer(source_vec, absolute_filename.as_str()).await {
    Ok(code) => {
      *source = code;
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
  }
}

// If this test fails, you should modify `set_info` function, according to your changes about AssetInfo
// Make sure every field of AssetInfo is considered
#[test]
fn ensure_info_fields() {
  let info = AssetInfo::default();
  std::hint::black_box(info);
}
