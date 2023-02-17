#![feature(let_chains)]
use std::{
  collections::HashMap,
  fmt::Display,
  fs,
  path::{Path, PathBuf, MAIN_SEPARATOR},
};

use glob::MatchOptions;
use regex::Regex;
use rspack_core::{
  rspack_sources::RawSource, AssetInfo, Compilation, CompilationAsset, Filename, FromType, Pattern,
  Plugin, ToType,
};
use rspack_error::Diagnostic;
use sugar_path::{AsPath, SugarPath};

#[derive(Debug)]
struct Logger(&'static str);

impl Logger {
  pub fn log(&self, msg: impl Display) {
    println!("[{}]: {}", self.0, msg);
  }

  pub fn debug(&self, msg: impl Display) {
    println!("[{}]debug: {}", self.0, msg);
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
  #[allow(clippy::unwrap_in_result)]
  fn run_patter(
    &self,
    compilation: &mut Compilation,
    pattern: &Pattern,
    _index: usize,
  ) -> Option<Vec<Option<RunPatternResult>>> {
    let orig_from = &pattern.from;
    let normalized_orig_from = PathBuf::from(orig_from);
    let mut context = pattern
      .context
      .clone()
      .unwrap_or(compilation.options.context.as_path().to_path_buf());

    LOGGER.log(format!(
      "starting to process a pattern from '{}' using '{:?}' context",
      normalized_orig_from.display(),
      pattern.context.as_ref().map(|p| p.display())
    ));

    let abs_from = if normalized_orig_from.is_absolute() {
      normalized_orig_from.clone()
    } else {
      context.join(&normalized_orig_from)
    };

    LOGGER.debug(format!("getting stats for '{}'...", abs_from.display()));

    let from_type = if let Ok(meta) = fs::metadata(&abs_from) {
      if meta.is_dir() {
        LOGGER.debug(format!(
          "determined '{}' is a directory",
          abs_from.display()
        ));
        FromType::Dir
      } else if meta.is_file() {
        LOGGER.debug(format!("determined '{}' is a file", abs_from.display()));
        FromType::File
      } else {
        LOGGER.debug(format!("determined '{}' is a unknown", abs_from.display()));
        FromType::Glob
      }
    } else {
      LOGGER.debug(format!("determined '{}' is a glob", abs_from.display()));
      FromType::Glob
    };

    // Enable dot in directory
    let mut dot_enable = pattern.glob_options.as_ref().and_then(|opt| opt.dot);

    let glob_query = match from_type {
      FromType::Dir => {
        LOGGER.debug(format!(
          "added '{}' as a context dependency",
          abs_from.display()
        ));
        compilation.context_dependencies.insert(abs_from.clone());
        context = abs_from.clone();

        if dot_enable.is_none() {
          dot_enable = Some(true);
        }

        abs_from.join("**/*").to_string_lossy().to_string()
      }
      FromType::File => {
        LOGGER.debug(format!(
          "added '{}' as a file dependency",
          abs_from.display()
        ));
        compilation.file_dependencies.insert(abs_from.clone());
        context = abs_from
          .parent()
          .map(|p| p.to_path_buf())
          .unwrap_or(PathBuf::new());

        if dot_enable.is_none() {
          dot_enable = Some(true);
        }

        abs_from.to_string_lossy().to_string()
      }
      FromType::Glob => {
        // TODO add glob parent
        // compilation.contextDependencies.add(contextDependencies);
        // logger.debug(format!(
        //   "added '{}' as a context dependency",
        //   abs_from.display()
        // ));
        if Path::new(orig_from).is_absolute() {
          orig_from.into()
        } else {
          context.join(orig_from).to_string_lossy().to_string()
        }
      }
    };

    LOGGER.log(format!("begin globbing '{glob_query}'..."));

    let glob_entries = glob::glob_with(
      &glob_query,
      MatchOptions {
        case_sensitive: pattern
          .glob_options
          .as_ref()
          .and_then(|opt| opt.case_sensitive_match)
          .unwrap_or(true),
        require_literal_separator: Default::default(),
        require_literal_leading_dot: !dot_enable.unwrap_or(false),
      },
    );

    match glob_entries {
      Ok(entries) => {
        let entries: Vec<_> = entries.collect();

        if entries.is_empty() {
          if pattern.no_error_on_missing {
            LOGGER.log(
              "finished to process a pattern from '${normalizedOriginalFrom}' using '${pattern.context}' context to '${pattern.to}'"
            );
            return None;
          }

          compilation.push_diagnostic(Diagnostic::error(
            "CopyRspackPlugin Error".into(),
            format!("unable to locate '{glob_query}' glob"),
            0,
            0,
          ))
        }

        let copied_result: Vec<_> = entries
          .into_iter()
          .map(|entry| {
            if entry.is_err() {
              // TODO error handler
              return None;
            }

            let entry = entry.expect("UNREACHABLE");

            // Exclude directories
            if entry.is_dir() {
              return None;
            }

            let from = entry.as_path().to_path_buf();

            LOGGER.debug(format!("found '{}'", from.display()));

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

            LOGGER.log(format!("'to' option '{to}' determinated as '{to_type}'"));

            let filename = if matches!(to_type, ToType::Dir) {
              // TODO: relative path
              let relative = absolute_filename.strip_prefix(&context).ok();

              if let Some(relative) = relative {
                PathBuf::from(&to).join(relative)
              } else {
                to.into()
              }
            } else {
              to.into()
            };

            let filename = if filename.is_absolute() {
              filename
                .strip_prefix(&compilation.options.output.path)
                .expect("TODO")
                .into()
            } else {
              filename
            };

            LOGGER.log(format!(
              "determined that '{}' should write to '{}'",
              from.display(),
              filename.display()
            ));

            let source_filename = absolute_filename.strip_prefix(&context).unwrap();

            // If this came from a glob or dir, add it to the file dependencies
            if matches!(from_type, FromType::Dir | FromType::Glob) {
              LOGGER.debug(format!(
                "added '{}' as a file dependency",
                absolute_filename.display()
              ));

              compilation
                .file_dependencies
                .insert(absolute_filename.clone());
            }

            // TODO cache

            LOGGER.debug(format!("reading '{}'...", absolute_filename.display()));
            // TODO inputFileSystem

            let source = match fs::read(absolute_filename.clone()) {
              Ok(data) => {
                LOGGER.debug(format!("read '{}'...", absolute_filename.display()));

                RawSource::Buffer(data)
              }
              Err(e) => {
                let rspack_err = rspack_error::Error::from(e);
                compilation.push_batch_diagnostic(rspack_err.into());
                return None;
              }
            };

            let filename = if matches!(&to_type, ToType::Template) {
              LOGGER.log(format!(
                "interpolating template '{}' for '${}'...`",
                filename.display(),
                source_filename.display()
              ));

              // TODO hash
              let hash = compilation.hash.clone();
              let ext = source_filename.extension().expect("This must be some");
              let base = source_filename.file_name().expect("This must be some");
              let name = &base.to_str().unwrap().strip_suffix(ext.to_str().unwrap());

              let template_str = Filename::from(filename.to_string_lossy().to_string()).render(
                rspack_core::FilenameRenderOptions {
                  name: name.map(Into::into),
                  path: None,
                  extension: Some(ext.to_string_lossy().to_string()),
                  id: Some(source_filename.to_string_lossy().to_string()),
                  contenthash: Some(hash.clone()),
                  chunkhash: Some(hash.clone()),
                  hash: Some(hash),
                  query: None,
                },
              );

              LOGGER.log(format!(
                "interpolated template '{template_str}' for '{}'",
                filename.display()
              ));

              template_str
            } else {
              filename.normalize().to_string_lossy().to_string()
            };

            Some(RunPatternResult {
              source_filename: source_filename.into(),
              absolute_filename,
              filename,
              source,
              info: None,
              force: pattern.force,
            })
          })
          .collect();

        if copied_result.is_empty() {
          if pattern.no_error_on_missing {
            return None;
          }

          // TODO err handler
          compilation.push_diagnostic(Diagnostic::error(
            "CopyRspackPlugin Error".into(),
            format!("unable to locate '{glob_query}' glob"),
            0,
            0,
          ));
          return None;
        }

        Some(copied_result)
      }
      Err(e) => {
        if pattern.no_error_on_missing {
          LOGGER.log(format!(
            "finished to process a pattern from '{}' using '{}' context to '{:?}'",
            normalized_orig_from.display(),
            context.display(),
            pattern.to
          ));

          return None;
        }

        compilation.push_diagnostic(Diagnostic::error(
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

impl Plugin for CopyPlugin {
  fn name(&self) -> &'static str {
    "copy-rspack-plugin"
  }

  fn process_assets_stage_additional<'life0, 'life1, 'async_trait>(
    &'life0 mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::ProcessAssetsArgs<'life1>,
  ) -> core::pin::Pin<
    Box<
      dyn core::future::Future<Output = rspack_core::PluginProcessAssetsOutput>
        + core::marker::Send
        + 'async_trait,
    >,
  >
  where
    'life0: 'async_trait,
    'life1: 'async_trait,
    Self: 'async_trait,
  {
    let mut copied_map: HashMap<i32, Vec<RunPatternResult>> = HashMap::default();
    self
      .patterns
      .iter()
      .enumerate()
      .for_each(|(index, pattern)| {
        let mut pattern = pattern.clone();
        if pattern.context.is_none() {
          pattern.context = Some(args.compilation.options.context.as_path().into());
        } else if let Some(ctx) = pattern.context.clone() && !ctx.is_absolute() {
          pattern.context = Some(args.compilation.options.context.join(ctx))
        };

        let copied_result = self.run_patter(args.compilation, &pattern, index);
        if let Some(copied_result) = copied_result {
          let filtered_copied_result = copied_result.into_iter().flatten();

          if let Some(res) = copied_map.get_mut(&pattern.priority) {
            res.extend(filtered_copied_result);
          } else {
            copied_map.insert(pattern.priority, filtered_copied_result.collect());
          }
        }
      });

    let mut copied_result: Vec<(i32, Vec<RunPatternResult>)> = copied_map.into_iter().collect();
    copied_result.sort_by(|a, b| a.0.cmp(&b.0));
    copied_result
      .into_iter()
      .fold(vec![], |mut acc: Vec<RunPatternResult>, curr| {
        acc.extend(curr.1.into_iter());
        acc
      })
      .into_iter()
      .for_each(|result| {
        if let Some(exist_asset) = args.compilation.assets.get_mut(&result.filename) {
          if !result.force {
            return;
          }
          exist_asset.set_source(Some(Box::new(result.source)));
          // TODO set info { copied: true, sourceFilename }
        } else {
          args.compilation.emit_asset(
            result.filename,
            CompilationAsset {
              source: Some(Box::new(result.source)),
              info: AssetInfo::default(),
            },
          )
        }
      });

    Box::pin(async move { Ok(()) })
  }
}
