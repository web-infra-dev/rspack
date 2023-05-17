#![feature(let_chains)]
use std::{
  fs,
  path::{Path, PathBuf, MAIN_SEPARATOR},
  sync::Arc,
};

use async_trait::async_trait;
use dashmap::DashSet;
use glob::MatchOptions;
use regex::Regex;
use rspack_core::{
  rspack_sources::RawSource, AssetInfo, Compilation, CompilationAsset, Filename, FromType,
  PathData, Pattern, Plugin, ToType,
};
use rspack_error::Diagnostic;
use sugar_path::{AsPath, SugarPath};
use tracing::Level;
use xxhash_rust::xxh3::Xxh3;

#[derive(Debug)]
struct Logger(&'static str);

impl Logger {
  pub fn log(&self, msg: &str) {
    tracing::event!(Level::INFO, msg);
  }

  pub fn debug(&self, msg: &str) {
    tracing::event!(Level::DEBUG, msg);
  }
}

#[derive(Debug, Clone)]
pub struct RunPatternResult {
  pub source_filename: PathBuf,
  pub absolute_filename: PathBuf,
  pub filename: String,
  pub source: RawSource,
  pub info: Option<AssetInfo>,
  pub force: bool,
  pub priority: i32,
}

static LOGGER: Logger = Logger("copy-rspack-plugin");

#[derive(Debug)]
pub struct CopyPlugin {
  pub patterns: Vec<Pattern>,
}

lazy_static::lazy_static! {
  /// This is an example for using doc comment attributes
  static ref TEMPLATE_RE: Regex = Regex::new(r"\[\\*([\w:]+)\\*\]").expect("This never fail");
}

impl CopyPlugin {
  pub fn new(patterns: Vec<Pattern>) -> Self {
    Self { patterns }
  }

  fn get_content_hash(source: &RawSource) -> u64 {
    let mut hasher = Xxh3::default();
    match &source {
      RawSource::Buffer(buffer) => {
        hasher.update(buffer);
      }
      RawSource::Source(source) => {
        hasher.update(source.as_bytes());
      }
    }
    hasher.digest()
  }

  #[allow(clippy::too_many_arguments)]
  async fn analyze_every_entry(
    entry: PathBuf,
    pattern: &Pattern,
    context: &Path,
    output_path: &Path,
    from_type: FromType,
    file_dependencies: &DashSet<PathBuf>,
    diagnostics: &DashSet<Diagnostic>,
    compilation: &Compilation,
  ) -> Option<RunPatternResult> {
    // Exclude directories
    if entry.is_dir() {
      return None;
    }
    if let Some(ignore) = &pattern.glob_options.ignore && ignore.iter().any(|ignore| {
      ignore.matches(&entry.to_string_lossy())
    }) {
      return None
    }

    let from = entry.as_path().to_path_buf();

    LOGGER.debug(&format!("found '{}'", from.display()));

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

    LOGGER.log(&format!("'to' option '{to}' determined as '{to_type}'"));

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

    LOGGER.log(&format!(
      "determined that '{}' should write to '{}'",
      from.display(),
      filename.display()
    ));

    let source_filename = relative?;

    // If this came from a glob or dir, add it to the file dependencies
    if matches!(from_type, FromType::Dir | FromType::Glob) {
      LOGGER.debug(&format!(
        "added '{}' as a file dependency",
        absolute_filename.display()
      ));

      file_dependencies.insert(absolute_filename.clone());
    }

    // TODO cache

    LOGGER.debug(&format!("reading '{}'...", absolute_filename.display()));
    // TODO inputFileSystem

    let source = match tokio::fs::read(absolute_filename.clone()).await {
      Ok(data) => {
        LOGGER.debug(&format!("read '{}'...", absolute_filename.display()));

        RawSource::Buffer(data)
      }
      Err(e) => {
        let rspack_err: Vec<Diagnostic> = rspack_error::Error::from(e).into();
        for err in rspack_err {
          diagnostics.insert(err);
        }
        return None;
      }
    };

    let filename = if matches!(&to_type, ToType::Template) {
      LOGGER.log(&format!(
        "interpolating template '{}' for '${}'...`",
        filename.display(),
        source_filename.display()
      ));

      let content_hash = format!("{:016x}", Self::get_content_hash(&source));
      let template_str = compilation.get_asset_path(
        &Filename::from(filename.to_string_lossy().to_string()),
        PathData::default()
          .filename(&source_filename)
          .content_hash(&content_hash)
          .hash(&content_hash),
      );

      LOGGER.log(&format!(
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
      info: None,
      force: pattern.force,
      priority: pattern.priority,
    })
  }

  fn run_patter(
    compilation: &Compilation,
    pattern: &Pattern,
    _index: usize,
    file_dependencies: &DashSet<PathBuf>,
    context_dependencies: &DashSet<PathBuf>,
    diagnostics: &DashSet<Diagnostic>,
  ) -> Option<Vec<Option<RunPatternResult>>> {
    let orig_from = &pattern.from;
    let normalized_orig_from = PathBuf::from(orig_from);
    let mut context = pattern
      .context
      .clone()
      .unwrap_or(compilation.options.context.as_path().to_path_buf());

    LOGGER.log(&format!(
      "starting to process a pattern from '{}' using '{:?}' context",
      normalized_orig_from.display(),
      pattern.context.as_ref().map(|p| p.display())
    ));

    let abs_from = if normalized_orig_from.is_absolute() {
      normalized_orig_from
    } else {
      context.join(&normalized_orig_from)
    };

    LOGGER.debug(&format!("getting stats for '{}'...", abs_from.display()));

    let from_type = if let Ok(meta) = fs::metadata(&abs_from) {
      if meta.is_dir() {
        LOGGER.debug(&format!(
          "determined '{}' is a directory",
          abs_from.display()
        ));
        FromType::Dir
      } else if meta.is_file() {
        LOGGER.debug(&format!("determined '{}' is a file", abs_from.display()));
        FromType::File
      } else {
        LOGGER.debug(&format!("determined '{}' is a unknown", abs_from.display()));
        FromType::Glob
      }
    } else {
      LOGGER.debug(&format!("determined '{}' is a glob", abs_from.display()));
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
        LOGGER.debug(&format!(
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
        LOGGER.debug(&format!(
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

    LOGGER.log(&format!("begin globbing '{glob_query}'..."));

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

        if need_add_context_to_dependency &&
        let Some(common_dir) = get_closest_common_parent_dir(
          &entries.iter().map(|it| it.as_path()).collect(),
        ) {
          context_dependencies.insert(common_dir);
        }

        if entries.is_empty() {
          if pattern.no_error_on_missing {
            LOGGER.log(
              "finished to process a pattern from '${normalizedOriginalFrom}' using '${pattern.context}' context to '${pattern.to}'"
            );
            return None;
          }

          diagnostics.insert(Diagnostic::error(
            "CopyRspackPlugin Error".into(),
            format!("unable to locate '{glob_query}' glob"),
            0,
            0,
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
            )
            .await
          })
          .collect::<rspack_futures::FuturesResults<Option<RunPatternResult>>>();

        if copied_result.is_empty() {
          if pattern.no_error_on_missing {
            return None;
          }

          // TODO err handler
          diagnostics.insert(Diagnostic::error(
            "CopyRspackPlugin Error".into(),
            format!("unable to locate '{glob_query}' glob"),
            0,
            0,
          ));
          return None;
        }

        Some(copied_result.into_inner())
      }
      Err(e) => {
        if pattern.no_error_on_missing {
          LOGGER.log(&format!(
            "finished to process a pattern from '{}' using '{}' context to '{:?}'",
            PathBuf::from(orig_from).display(),
            context.display(),
            pattern.to
          ));

          return None;
        }

        diagnostics.insert(Diagnostic::error(
          "Glob Error".into(),
          e.msg.to_string(),
          0,
          0,
        ));

        None
      }
    }
  }
}

#[async_trait]
impl Plugin for CopyPlugin {
  fn name(&self) -> &'static str {
    "copy-rspack-plugin"
  }

  async fn process_assets_stage_additional(
    &mut self,
    _ctx: rspack_core::PluginContext,
    mut args: rspack_core::ProcessAssetsArgs<'_>,
  ) -> rspack_core::PluginProcessAssetsOutput {
    let file_dependencies = DashSet::default();
    let context_dependencies = DashSet::default();
    let diagnostics = DashSet::default();

    let mut copied_result: Vec<(i32, RunPatternResult)> = self
      .patterns
      .iter()
      .enumerate()
      .map(|(index, pattern)| {
        let mut pattern = pattern.clone();
        if pattern.context.is_none() {
          pattern.context = Some(args.compilation.options.context.as_path().into());
        } else if let Some(ctx) = pattern.context.clone() && !ctx.is_absolute() {
          pattern.context = Some(args.compilation.options.context.join(ctx))
        };

        Self::run_patter(
          args.compilation,
          &pattern,
          index,
          &file_dependencies,
          &context_dependencies,
          &diagnostics,
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

    let compilation = &mut args.compilation;
    compilation.file_dependencies.extend(file_dependencies);
    compilation
      .context_dependencies
      .extend(context_dependencies);
    compilation.push_batch_diagnostic(diagnostics.into_iter().collect());

    copied_result.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    copied_result.into_iter().for_each(|(_priority, result)| {
      if let Some(exist_asset) = args.compilation.assets_mut().get_mut(&result.filename) {
        if !result.force {
          return;
        }
        exist_asset.set_source(Some(Arc::new(result.source)));
        // TODO set info { copied: true, sourceFilename }
      } else {
        args.compilation.emit_asset(
          result.filename,
          CompilationAsset {
            source: Some(Arc::new(result.source)),
            info: AssetInfo::default(),
          },
        )
      }
    });

    Ok(())
  }
}

fn get_closest_common_parent_dir(paths: &Vec<&Path>) -> Option<PathBuf> {
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

#[test]
fn test_escape() {
  assert_eq!(escape_glob_chars("a/b/**/*.js"), r#"a/b/\*\*/\*.js"#);
  assert_eq!(escape_glob_chars("a/b/c"), r#"a/b/c"#);
}
