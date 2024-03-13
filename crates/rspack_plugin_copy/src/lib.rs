#![feature(let_chains)]
use std::{
  fmt::Display,
  fs,
  hash::Hash,
  path::{Path, PathBuf, MAIN_SEPARATOR},
  sync::{Arc, Mutex},
};

use dashmap::DashSet;
use glob::{MatchOptions, Pattern as GlobPattern};
use regex::Regex;
use rspack_core::{
  rspack_sources::RawSource, AssetInfo, AssetInfoRelated, Compilation, CompilationAsset,
  CompilationLogger, Filename, Logger, PathData, Plugin,
};
use rspack_error::{Diagnostic, DiagnosticError, Error, ErrorExt, Result};
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash, RspackHashDigest};
use rspack_hook::{plugin, plugin_hook, AsyncSeries};
use sugar_path::{AsPath, SugarPath};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct CopyPattern {
  pub from: String,
  pub to: Option<String>,
  pub context: Option<PathBuf>,
  pub to_type: Option<ToType>,
  pub no_error_on_missing: bool,
  pub info: Option<Info>,
  pub force: bool,
  pub priority: i32,
  pub glob_options: CopyGlobOptions,
}

#[derive(Debug, Clone)]
pub struct CopyGlobOptions {
  pub case_sensitive_match: Option<bool>,
  pub dot: Option<bool>,
  pub ignore: Option<Vec<GlobPattern>>,
}

#[derive(Debug, Clone)]
pub struct RunPatternResult {
  pub source_filename: PathBuf,
  pub absolute_filename: PathBuf,
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

lazy_static::lazy_static! {
  /// This is an example for using doc comment attributes
  static ref TEMPLATE_RE: Regex = Regex::new(r"\[\\*([\w:]+)\\*\]").expect("This never fail");
}

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
    match &source {
      RawSource::Buffer(buffer) => {
        buffer.hash(&mut hasher);
      }
      RawSource::Source(source) => {
        source.hash(&mut hasher);
      }
    }
    hasher.digest(digest)
  }

  #[allow(clippy::too_many_arguments)]
  async fn analyze_every_entry(
    entry: PathBuf,
    pattern: &CopyPattern,
    context: &Path,
    output_path: &Path,
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
      && ignore
        .iter()
        .any(|ignore| ignore.matches(&entry.to_string_lossy()))
    {
      return None;
    }

    let from = entry.as_path().to_path_buf();

    logger.debug(format!("found '{}'", from.display()));

    let absolute_filename = if from.is_absolute() {
      from.clone()
    } else {
      context.join(&from)
    };

    let to = if let Some(to) = pattern.to.as_ref() {
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

    let relative = pathdiff::diff_paths(&absolute_filename, context);
    let filename = if matches!(to_type, ToType::Dir) {
      if let Some(relative) = &relative {
        PathBuf::from(&to).join(relative)
      } else {
        to.into()
      }
    } else {
      to.into()
    };

    let filename = if filename.is_absolute() {
      pathdiff::diff_paths(filename, output_path)?
    } else {
      filename
    };

    logger.log(format!(
      "determined that '{}' should write to '{}'",
      from.display(),
      filename.display()
    ));

    let source_filename = relative?;

    // If this came from a glob or dir, add it to the file dependencies
    if matches!(from_type, FromType::Dir | FromType::Glob) {
      logger.debug(format!(
        "added '{}' as a file dependency",
        absolute_filename.display()
      ));

      file_dependencies.insert(absolute_filename.clone());
    }

    // TODO cache

    logger.debug(format!("reading '{}'...", absolute_filename.display()));
    // TODO inputFileSystem

    let source = match tokio::fs::read(absolute_filename.clone()).await {
      Ok(data) => {
        logger.debug(format!("read '{}'...", absolute_filename.display()));

        RawSource::Buffer(data)
      }
      Err(e) => {
        let e: Error = DiagnosticError::from(e.boxed()).into();
        let rspack_err: Vec<Diagnostic> = vec![e.into()];
        diagnostics
          .lock()
          .expect("failed to obtain lock of `diagnostics`")
          .extend(rspack_err);
        return None;
      }
    };

    let filename = if matches!(&to_type, ToType::Template) {
      logger.log(format!(
        "interpolating template '{}' for '${}'...`",
        filename.display(),
        source_filename.display()
      ));

      let content_hash = Self::get_content_hash(
        &source,
        &compilation.options.output.hash_function,
        &compilation.options.output.hash_digest,
        &compilation.options.output.hash_salt,
      );
      let content_hash = content_hash.rendered(compilation.options.output.hash_digest_length);
      let template_str = compilation.get_asset_path(
        &Filename::from(filename.to_string_lossy().to_string()),
        PathData::default()
          .filename(&source_filename.to_string_lossy())
          .content_hash(content_hash)
          .hash_optional(compilation.get_hash()),
      );

      logger.log(format!(
        "interpolated template '{template_str}' for '{}'",
        filename.display()
      ));

      template_str
    } else {
      filename.normalize().to_string_lossy().to_string()
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
    let normalized_orig_from = PathBuf::from(orig_from);
    let mut context = pattern
      .context
      .clone()
      .unwrap_or(compilation.options.context.as_path().to_path_buf());

    logger.log(format!(
      "starting to process a pattern from '{}' using '{:?}' context",
      normalized_orig_from.display(),
      pattern.context.as_ref().map(|p| p.display())
    ));

    let abs_from = if normalized_orig_from.is_absolute() {
      normalized_orig_from
    } else {
      context.join(&normalized_orig_from)
    };

    logger.debug(format!("getting stats for '{}'...", abs_from.display()));

    let from_type = if let Ok(meta) = fs::metadata(&abs_from) {
      if meta.is_dir() {
        logger.debug(format!(
          "determined '{}' is a directory",
          abs_from.display()
        ));
        FromType::Dir
      } else if meta.is_file() {
        logger.debug(format!("determined '{}' is a file", abs_from.display()));
        FromType::File
      } else {
        logger.debug(format!("determined '{}' is a unknown", abs_from.display()));
        FromType::Glob
      }
    } else {
      logger.debug(format!("determined '{}' is a glob", abs_from.display()));
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
        logger.debug(format!(
          "added '{}' as a context dependency",
          abs_from.display()
        ));
        context_dependencies.insert(abs_from.clone());
        context = abs_from.clone();

        if dot_enable.is_none() {
          dot_enable = Some(true);
        }
        let mut escaped = PathBuf::from(escape_glob_chars(abs_from.to_str()?));
        escaped.push("**/*");

        escaped.to_string_lossy().to_string()
      }
      FromType::File => {
        logger.debug(format!(
          "added '{}' as a file dependency",
          abs_from.display()
        ));
        file_dependencies.insert(abs_from.clone());
        context = abs_from
          .parent()
          .map(|p| p.to_path_buf())
          .unwrap_or(PathBuf::new());

        if dot_enable.is_none() {
          dot_enable = Some(true);
        }

        escape_glob_chars(abs_from.to_str()?)
      }
      FromType::Glob => {
        need_add_context_to_dependency = true;
        if Path::new(orig_from).is_absolute() {
          orig_from.into()
        } else {
          context.join(orig_from).to_string_lossy().to_string()
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
            let entry = entry.ok()?;

            let filters = pattern.glob_options.ignore.as_ref();

            if let Some(filters) = filters {
              // If filters length is 0, exist is true by default
              let exist = filters
                .iter()
                .all(|filter| !filter.matches(&entry.to_string_lossy()));
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
          context_dependencies.insert(common_dir);
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
          logger.log(format!(
            "finished to process a pattern from '{}' using '{}' context to '{:?}'",
            PathBuf::from(orig_from).display(),
            context.display(),
            pattern.to
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

#[plugin_hook(AsyncSeries<Compilation> for CopyRspackPlugin, stage = Compilation::PROCESS_ASSETS_STAGE_ADDITIONAL)]
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
      let mut pattern = pattern.clone();
      if pattern.context.is_none() {
        pattern.context = Some(compilation.options.context.as_path().into());
      } else if let Some(ctx) = pattern.context.clone()
        && !ctx.is_absolute()
      {
        pattern.context = Some(compilation.options.context.as_path().join(ctx))
      };

      CopyRspackPlugin::run_patter(
        compilation,
        &pattern,
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
  compilation.file_dependencies.extend(file_dependencies);
  compilation
    .context_dependencies
    .extend(context_dependencies);
  compilation.push_batch_diagnostic(
    diagnostics
      .lock()
      .expect("failed to obtain lock of `diagnostics`")
      .drain(..)
      .collect(),
  );

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
      // TODO set info { copied: true, sourceFilename }
    } else {
      let mut asset_info = Default::default();
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
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}

fn get_closest_common_parent_dir(paths: &[&Path]) -> Option<PathBuf> {
  // If there are no matching files, return `None`.
  if paths.is_empty() {
    return None;
  }

  // Get the first file path and use it as the initial value for the common parent directory.
  let mut parent_dir: PathBuf = paths[0].parent()?.into();

  // Iterate over the remaining file paths, updating the common parent directory as necessary.
  for path in paths.iter().skip(1) {
    // Find the common parent directory between the current file path and the previous common parent directory.
    while !path.starts_with(&parent_dir) {
      parent_dir = parent_dir.parent()?.into();
    }
  }

  Some(parent_dir)
}

fn escape_glob_chars(s: &str) -> String {
  let mut escaped = String::with_capacity(s.len());
  for c in s.chars() {
    match c {
      '*' | '?' | '[' | ']' => escaped.push('\\'),
      _ => {}
    }
    escaped.push(c);
  }
  escaped
}

fn set_info(target: &mut AssetInfo, info: Info) {
  if let Some(minimized) = info.minimized {
    target.minimized = minimized;
  }

  if let Some(immutable) = info.immutable {
    target.immutable = immutable;
  }

  if let Some(chunk_hash) = info.chunk_hash {
    target.chunk_hash = rustc_hash::FxHashSet::from_iter(chunk_hash);
  }

  if let Some(content_hash) = info.content_hash {
    target.content_hash = rustc_hash::FxHashSet::from_iter(content_hash);
  }

  if let Some(development) = info.development {
    target.development = development;
  }

  if let Some(hot_module_replacement) = info.hot_module_replacement {
    target.hot_module_replacement = hot_module_replacement;
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

#[test]
fn test_escape() {
  assert_eq!(escape_glob_chars("a/b/**/*.js"), r#"a/b/\*\*/\*.js"#);
  assert_eq!(escape_glob_chars("a/b/c"), r#"a/b/c"#);
}

// If this test fails, you should modify `set_info` function, according to your changes about AssetInfo
// Make sure every field of AssetInfo is considered
#[test]
fn ensure_info_fields() {
  let info = AssetInfo::default();
  std::hint::black_box(info);
}
