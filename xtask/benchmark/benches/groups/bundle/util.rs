use std::{
  collections::HashSet,
  io,
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack::builder::{Builder, CompilerBuilder};
use rspack_core::{
  Compiler, Experiments, Mode, ModuleOptions, ModuleRule, ModuleRuleEffect, ModuleRuleUse,
  ModuleRuleUseLoader, Resolve, RuleSetCondition,
};
use rspack_fs::{MemoryFileSystem, ReadableFileSystem, WritableFileSystem};
use rspack_regex::RspackRegex;
use serde_json::json;

// Because `CompilerBuilder` is not `Clone`
pub type CompilerBuilderGenerator = Arc<dyn Fn() -> CompilerBuilder + Send + Sync>;

pub struct BuilderOptions {
  pub project: &'static str,
  pub entry: &'static str,
}

pub fn basic_compiler_builder(options: BuilderOptions) -> CompilerBuilder {
  let mut builder = Compiler::builder();

  let dir = resolve_bench_project_dir(options.project);

  builder
    .context(dir.to_string_lossy().to_string())
    .entry("main", options.entry)
    .module(ModuleOptions::builder().rule(ModuleRule {
      test: Some(RuleSetCondition::Regexp(
        RspackRegex::new("\\.(j|t)s(x)?$").unwrap(),
      )),
      effect: ModuleRuleEffect {
        r#use: ModuleRuleUse::Array(vec![ModuleRuleUseLoader {
        loader: "builtin:swc-loader".to_string(),
        options: Some(json!({
            "jsc": {
                "parser": {
                    "syntax": "typescript",
                    "tsx": true,
                },
                "transform": {
                    "react": {
                        "runtime": "automatic",
                    },
                }
            },
        }).to_string()),
      }]),
        ..Default::default()
      },
      ..Default::default()
    }))
    .cache(rspack_core::CacheOptions::Disabled)
    .resolve(Resolve {
      extensions: Some(vec!["...".to_string(), ".jsx".to_string()]),
      symlinks: Some(false),
      ..Default::default()
    })
    .experiments(Experiments::builder().css(true))
    .output_filesystem(Arc::new(MemoryFileSystem::default()))
    .enable_loader_swc();

  builder
}

pub async fn prepare_projects(
  projects: Vec<(&'static str, CompilerBuilderGenerator)>,
) -> Vec<(&'static str, CompilerBuilderGenerator)> {
  let mut prepared_projects = Vec::with_capacity(projects.len());

  for (name, builder) in projects {
    let input_filesystem = preload_input_filesystem(name)
      .await
      .unwrap_or_else(|err| panic!("failed to preload benchmark project `{name}`: {err}"));

    prepared_projects.push((
      name,
      Arc::new(move || {
        let mut compiler = builder();
        compiler.input_filesystem(input_filesystem.clone());
        compiler
      }) as CompilerBuilderGenerator,
    ));
  }

  prepared_projects
}

pub fn derive_projects(
  projects: Vec<(&'static str, CompilerBuilderGenerator)>,
) -> Vec<(String, CompilerBuilderGenerator)> {
  projects
    .into_iter()
    .flat_map(|(name, builder)| {
      let mut projects = Vec::new();

      {
        let builder = builder.clone();
        projects.push((
          format!("{name}-development"),
          Arc::new(move || {
            let mut builder = builder();
            builder.mode(Mode::Development);
            builder
          }) as CompilerBuilderGenerator,
        ));
      }

      {
        let builder = builder.clone();
        projects.push((
          format!("{name}-production-sourcemap"),
          Arc::new(move || {
            let mut builder = builder();
            builder.mode(Mode::Production);
            builder.devtool(rspack::builder::Devtool::SourceMap);
            builder
          }),
        ));
      }

      projects
    })
    .collect()
}

fn resolve_bench_project_dir(project: &str) -> PathBuf {
  let benchcases_dir = std::env::var("RSPACK_BENCHCASES_DIR")
    .expect("RSPACK_BENCHCASES_DIR is required and must be an absolute path, e.g. RSPACK_BENCHCASES_DIR=/path/to/.bench/rspack-benchcases");

  PathBuf::from(benchcases_dir)
    .canonicalize()
    .unwrap()
    .join(project)
}

async fn preload_input_filesystem(project: &str) -> io::Result<Arc<dyn ReadableFileSystem>> {
  let project_dir = resolve_bench_project_dir(project);
  let memory_fs = Arc::new(MemoryFileSystem::default());
  let mut materialized_paths = HashSet::new();
  let mut active_canonical_dirs = HashSet::new();

  preload_directory(
    &project_dir,
    &memory_fs,
    &mut materialized_paths,
    &mut active_canonical_dirs,
  )
  .await?;

  Ok(memory_fs)
}

async fn preload_directory(
  path: &Path,
  memory_fs: &MemoryFileSystem,
  materialized_paths: &mut HashSet<PathBuf>,
  active_canonical_dirs: &mut HashSet<PathBuf>,
) -> io::Result<()> {
  if !materialized_paths.insert(path.to_path_buf()) {
    return Ok(());
  }

  let metadata = std::fs::metadata(path)?;
  let path = path_to_utf8(path)?;

  if metadata.is_dir() {
    let canonical_path = Path::new(path).canonicalize()?;
    if !active_canonical_dirs.insert(canonical_path.clone()) {
      return Ok(());
    }

    memory_fs
      .create_dir_all(path.into())
      .await
      .map_err(fs_error_to_io_error)?;

    for entry in std::fs::read_dir(path)? {
      let entry = entry?;
      let entry_path = entry.path();
      Box::pin(preload_directory(
        &entry_path,
        memory_fs,
        materialized_paths,
        active_canonical_dirs,
      ))
      .await?;
    }

    active_canonical_dirs.remove(&canonical_path);
    return Ok(());
  }

  if metadata.is_file() {
    if let Some(parent) = Path::new(path).parent() {
      let parent = path_to_utf8(parent)?;
      memory_fs
        .create_dir_all(parent.into())
        .await
        .map_err(fs_error_to_io_error)?;
    }

    let content = std::fs::read(path)?;
    memory_fs
      .write(path.into(), &content)
      .await
      .map_err(fs_error_to_io_error)?;
  }

  Ok(())
}

fn path_to_utf8(path: &Path) -> io::Result<&str> {
  path.to_str().ok_or_else(|| {
    io::Error::new(
      io::ErrorKind::InvalidData,
      format!("benchmark path must be valid UTF-8: {}", path.display()),
    )
  })
}

fn fs_error_to_io_error(error: rspack_fs::Error) -> io::Error {
  io::Error::other(error.to_string())
}
