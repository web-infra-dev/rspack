use std::{
  collections::{HashMap, HashSet},
  io,
  path::{Path, PathBuf},
  sync::Arc,
};

use rspack::builder::{Builder, CompilerBuilder};
use rspack_core::{
  Compiler, Experiments, Mode, ModuleOptions, ModuleRule, ModuleRuleEffect, ModuleRuleUse,
  ModuleRuleUseLoader, Resolve, RuleSetCondition,
};
use rspack_fs::{
  FileMetadata, FilePermissions, MemoryFileSystem, ReadableFileSystem, WritableFileSystem,
};
use rspack_paths::{AssertUtf8, Utf8Path, Utf8PathBuf};
use rspack_regex::RspackRegex;
use serde_json::json;

// Because `CompilerBuilder` is not `Clone`
pub type CompilerBuilderGenerator = Arc<dyn Fn() -> CompilerBuilder + Send + Sync>;

pub struct BuilderOptions {
  pub project: &'static str,
  pub entry: &'static str,
}

#[derive(Debug)]
struct PreloadedInputFileSystem {
  memory: Arc<MemoryFileSystem>,
  symlink_metadata: HashMap<Utf8PathBuf, FileMetadata>,
  symlink_targets: HashMap<Utf8PathBuf, Utf8PathBuf>,
}

impl PreloadedInputFileSystem {
  fn new(
    memory: Arc<MemoryFileSystem>,
    symlink_metadata: HashMap<Utf8PathBuf, FileMetadata>,
    symlink_targets: HashMap<Utf8PathBuf, Utf8PathBuf>,
  ) -> Self {
    Self {
      memory,
      symlink_metadata,
      symlink_targets,
    }
  }

  fn canonicalized_path(&self, path: &Utf8Path) -> Utf8PathBuf {
    let path = normalize_utf8_path(path);
    self
      .symlink_targets
      .iter()
      .filter_map(|(symlink_path, target_path)| {
        path
          .strip_prefix(symlink_path)
          .ok()
          .map(|rest| (symlink_path, target_path, rest))
      })
      .max_by_key(|(symlink_path, _, _)| symlink_path.as_str().len())
      .map(|(_, target_path, rest)| {
        if rest.as_str().is_empty() {
          target_path.clone()
        } else {
          target_path.join(rest)
        }
      })
      .unwrap_or(path)
  }
}

#[async_trait::async_trait]
impl ReadableFileSystem for PreloadedInputFileSystem {
  async fn read(&self, path: &Utf8Path) -> rspack_fs::Result<Vec<u8>> {
    self.memory.read(&normalize_utf8_path(path)).await
  }

  fn read_sync(&self, path: &Utf8Path) -> rspack_fs::Result<Vec<u8>> {
    self.memory.read_sync(&normalize_utf8_path(path))
  }

  async fn metadata(&self, path: &Utf8Path) -> rspack_fs::Result<FileMetadata> {
    self.memory.metadata(&normalize_utf8_path(path)).await
  }

  fn metadata_sync(&self, path: &Utf8Path) -> rspack_fs::Result<FileMetadata> {
    self.memory.metadata_sync(&normalize_utf8_path(path))
  }

  async fn symlink_metadata(&self, path: &Utf8Path) -> rspack_fs::Result<FileMetadata> {
    let path = normalize_utf8_path(path);
    if let Some(metadata) = self.symlink_metadata.get(&path) {
      return Ok(metadata.clone());
    }
    self.memory.metadata(&path).await
  }

  async fn canonicalize(&self, path: &Utf8Path) -> rspack_fs::Result<Utf8PathBuf> {
    let path = self.canonicalized_path(path);
    self.memory.metadata(&path).await?;
    Ok(path)
  }

  async fn read_dir(&self, dir: &Utf8Path) -> rspack_fs::Result<Vec<String>> {
    ReadableFileSystem::read_dir(self.memory.as_ref(), &normalize_utf8_path(dir)).await
  }

  fn read_dir_sync(&self, dir: &Utf8Path) -> rspack_fs::Result<Vec<String>> {
    self.memory.read_dir_sync(&normalize_utf8_path(dir))
  }

  async fn permissions(&self, _path: &Utf8Path) -> rspack_fs::Result<Option<FilePermissions>> {
    Ok(None)
  }
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

fn resolve_benchcases_dir() -> PathBuf {
  let benchcases_dir = std::env::var("RSPACK_BENCHCASES_DIR")
    .expect("RSPACK_BENCHCASES_DIR is required and must be an absolute path, e.g. RSPACK_BENCHCASES_DIR=/path/to/.bench/rspack-benchcases");

  PathBuf::from(benchcases_dir).canonicalize().unwrap()
}

fn resolve_bench_project_dir(project: &str) -> PathBuf {
  resolve_benchcases_dir().join(project)
}

async fn preload_input_filesystem(project: &str) -> io::Result<Arc<dyn ReadableFileSystem>> {
  let _ = project;
  let benchcases_dir = resolve_benchcases_dir();
  let memory_fs = Arc::new(MemoryFileSystem::default());
  let mut materialized_paths = HashSet::new();
  let mut active_canonical_dirs = HashSet::new();
  let mut symlink_metadata = HashMap::new();
  let mut symlink_targets = HashMap::new();

  preload_directory(
    &benchcases_dir,
    &memory_fs,
    &mut materialized_paths,
    &mut active_canonical_dirs,
    &mut symlink_metadata,
    &mut symlink_targets,
  )
  .await?;

  Ok(Arc::new(PreloadedInputFileSystem::new(
    memory_fs,
    symlink_metadata,
    symlink_targets,
  )))
}

async fn preload_directory(
  path: &Path,
  memory_fs: &MemoryFileSystem,
  materialized_paths: &mut HashSet<PathBuf>,
  active_canonical_dirs: &mut HashSet<PathBuf>,
  symlink_metadata: &mut HashMap<Utf8PathBuf, FileMetadata>,
  symlink_targets: &mut HashMap<Utf8PathBuf, Utf8PathBuf>,
) -> io::Result<()> {
  if !materialized_paths.insert(path.to_path_buf()) {
    return Ok(());
  }

  let symlink_meta = std::fs::symlink_metadata(path)?;
  if symlink_meta.file_type().is_symlink() {
    symlink_metadata.insert(
      path_to_utf8_pathbuf(path)?,
      FileMetadata::try_from(symlink_meta).map_err(fs_error_to_io_error)?,
    );
    symlink_targets.insert(
      path_to_utf8_pathbuf(path)?,
      path_to_utf8_pathbuf(&path.canonicalize()?)?,
    );
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
        symlink_metadata,
        symlink_targets,
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

fn path_to_utf8_pathbuf(path: &Path) -> io::Result<Utf8PathBuf> {
  Utf8PathBuf::from_path_buf(path.to_path_buf()).map_err(|path| {
    io::Error::new(
      io::ErrorKind::InvalidData,
      format!("benchmark path must be valid UTF-8: {}", path.display()),
    )
  })
}

fn fs_error_to_io_error(error: rspack_fs::Error) -> io::Error {
  io::Error::other(error.to_string())
}

fn normalize_utf8_path(path: &Utf8Path) -> Utf8PathBuf {
  let mut normalized = PathBuf::new();

  for component in path.as_std_path().components() {
    match component {
      std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
      std::path::Component::RootDir => normalized.push(std::path::MAIN_SEPARATOR.to_string()),
      std::path::Component::CurDir => {}
      std::path::Component::ParentDir => {
        normalized.pop();
      }
      std::path::Component::Normal(part) => normalized.push(part),
    }
  }

  if normalized.as_os_str().is_empty() && path.is_absolute() {
    normalized.push(std::path::MAIN_SEPARATOR.to_string());
  }

  normalized.assert_utf8()
}
