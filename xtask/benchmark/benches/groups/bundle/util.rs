use std::{io, path::PathBuf, sync::Arc};

use rspack::builder::{Builder, CompilerBuilder};
use rspack_core::{
  Compiler, Experiments, Mode, ModuleOptions, ModuleRule, ModuleRuleEffect, ModuleRuleUse,
  ModuleRuleUseLoader, Resolve, RuleSetCondition,
};
use rspack_fs::{MemoryFileSystem, ReadableFileSystem, WritableFileSystem};
use rspack_regex::RspackRegex;
use serde_json::json;
use walkdir::WalkDir;

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

  for entry in WalkDir::new(&project_dir).follow_links(true) {
    let entry = entry.map_err(io::Error::other)?;
    let path = entry.path();
    let path = path.to_str().ok_or_else(|| {
      io::Error::new(
        io::ErrorKind::InvalidData,
        format!("benchmark path must be valid UTF-8: {}", path.display()),
      )
    })?;

    if entry.file_type().is_dir() {
      memory_fs
        .create_dir_all(path.into())
        .await
        .map_err(fs_error_to_io_error)?;
      continue;
    }

    if entry.file_type().is_file() {
      let content = std::fs::read(entry.path())?;
      if let Some(parent) = entry.path().parent() {
        let parent = parent.to_str().ok_or_else(|| {
          io::Error::new(
            io::ErrorKind::InvalidData,
            format!("benchmark path must be valid UTF-8: {}", parent.display()),
          )
        })?;
        memory_fs
          .create_dir_all(parent.into())
          .await
          .map_err(fs_error_to_io_error)?;
      }
      memory_fs
        .write(path.into(), &content)
        .await
        .map_err(fs_error_to_io_error)?;
    }
  }

  Ok(memory_fs)
}

fn fs_error_to_io_error(error: rspack_fs::Error) -> io::Error {
  io::Error::other(error.to_string())
}
