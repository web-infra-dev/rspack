use std::{collections::HashSet, path::PathBuf, time::UNIX_EPOCH};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{Content, Loader, LoaderContext, RunnerContext, rspack_sources::SourceMap};
use rspack_error::Result;
use rspack_loader_runner::{DisplayWithSuffix, Identifier};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod plugin;

pub use plugin::CacheLoaderPlugin;

pub const CACHE_LOADER_IDENTIFIER: &str = "builtin:cache-loader";

#[cacheable]
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CacheLoaderOptions {
  /// Directory used to store cache entries.
  pub cache_directory: Option<String>,
  /// Extra identifier used to invalidate all entries.
  pub cache_identifier: Option<String>,
}

#[cacheable]
#[derive(Debug)]
pub struct CacheLoader {
  identifier: Identifier,
  options: CacheLoaderOptions,
}

impl CacheLoader {
  pub fn new(identifier: Identifier, options: CacheLoaderOptions) -> Self {
    debug_assert!(identifier.starts_with(CACHE_LOADER_IDENTIFIER));
    Self {
      identifier,
      options,
    }
  }

  fn cache_directory(&self, compiler_context: &Utf8Path) -> Utf8PathBuf {
    if let Some(directory) = self.options.cache_directory.as_deref() {
      let directory = Utf8Path::new(directory);
      if directory.is_absolute() {
        directory.to_path_buf()
      } else {
        compiler_context.join(directory)
      }
    } else {
      compiler_context.join("node_modules/.cache/cache-loader")
    }
  }

  fn cache_identifier(&self, loader_context: &LoaderContext<RunnerContext>) -> String {
    self.options.cache_identifier.clone().unwrap_or_else(|| {
      format!(
        "builtin:cache-loader:{} {:?}",
        env!("CARGO_PKG_VERSION"),
        loader_context.context.options.mode
      )
    })
  }

  fn cache_key(&self, loader_context: &LoaderContext<RunnerContext>, request: &str) -> Utf8PathBuf {
    let compiler_context = loader_context.context.options.context.as_path();
    let identifier = self.cache_identifier(loader_context);
    let digest = Sha256::digest(format!("{identifier}\n{request}").as_bytes());
    let filename = format!("{digest:x}.json");
    self.cache_directory(compiler_context).join(filename)
  }

  async fn read_cache(&self, key: &Utf8Path) -> Option<CacheEntry> {
    let content = tokio::fs::read(key.as_std_path()).await.ok()?;
    serde_json::from_slice(&content).ok()
  }

  async fn write_cache(&self, key: &Utf8Path, entry: &CacheEntry) {
    let Ok(content) = serde_json::to_vec(entry) else {
      return;
    };
    let Some(parent) = key.parent() else {
      return;
    };
    if tokio::fs::create_dir_all(parent.as_std_path())
      .await
      .is_err()
    {
      return;
    }
    // Match cache-loader's best-effort behavior: cache I/O must not fail a build.
    let _ = tokio::fs::write(key.as_std_path(), content).await;
  }

  async fn is_cache_valid(&self, entry: &CacheEntry) -> bool {
    for dependency in entry.dependencies.iter().chain(&entry.context_dependencies) {
      let Ok(metadata) = tokio::fs::metadata(&dependency.path).await else {
        return false;
      };
      if metadata_mtime_ms(&metadata) != Some(dependency.mtime) {
        return false;
      }
    }
    true
  }
}

#[cacheable_dyn]
#[async_trait::async_trait]
impl Loader<RunnerContext> for CacheLoader {
  fn identifier(&self) -> Identifier {
    self.identifier
  }

  async fn pitch(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let remaining_request = loader_context
      .remaining_request()
      .display_with_suffix(loader_context.resource());
    let key = self.cache_key(loader_context, &remaining_request);
    let data = CacheLoaderData {
      key: key.as_str().to_string(),
      remaining_request: remaining_request.clone(),
    };
    let loader_index = loader_context.loader_index as usize;
    loader_context.loader_items[loader_index]
      .set_data(serde_json::to_value(data).expect("cache loader data should be serializable"));

    let Some(entry) = self.read_cache(&key).await else {
      return Ok(());
    };
    if entry.remaining_request != remaining_request || !self.is_cache_valid(&entry).await {
      return Ok(());
    }

    for dependency in &entry.dependencies {
      loader_context
        .file_dependencies
        .insert(PathBuf::from(&dependency.path));
    }
    for dependency in &entry.context_dependencies {
      loader_context
        .context_dependencies
        .insert(PathBuf::from(&dependency.path));
    }

    let content = match entry.content {
      CachedContent::String(content) => Content::String(content),
      CachedContent::Buffer(content) => Content::Buffer(content),
    };
    let source_map = match entry.source_map {
      Some(source_map) => {
        let Ok(source_map) = SourceMap::from_json(source_map) else {
          return Ok(());
        };
        Some(source_map)
      }
      None => None,
    };
    loader_context.finish_with((content, source_map, None));
    Ok(())
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let (content, source_map, additional_data) = loader_context.take_all();
    let Some(content) = content else {
      return Ok(());
    };

    // Additional data is type-erased on the Rust side. Avoid caching it instead
    // of returning an incomplete result on a cache hit.
    if additional_data.is_none() {
      let data =
        serde_json::from_value::<CacheLoaderData>(loader_context.current_loader().data().clone())
          .ok();
      if let Some(data) = data {
        let mut file_dependencies = loader_context.file_dependencies.clone();
        file_dependencies.extend(
          loader_context
            .loader_items
            .iter()
            .map(|loader| loader.path().as_std_path().to_path_buf())
            .filter(|path| path.is_absolute()),
        );
        let dependencies = dependency_details(file_dependencies).await;
        let context_dependencies =
          dependency_details(loader_context.context_dependencies.iter().cloned()).await;

        if let (Some(dependencies), Some(context_dependencies)) =
          (dependencies, context_dependencies)
        {
          let cached_content = match &content {
            Content::String(content) => CachedContent::String(content.clone()),
            Content::Buffer(content) => CachedContent::Buffer(content.clone()),
          };
          let entry = CacheEntry {
            remaining_request: data.remaining_request,
            dependencies,
            context_dependencies,
            content: cached_content,
            source_map: source_map.as_ref().map(SourceMap::to_json),
          };
          self.write_cache(Utf8Path::new(&data.key), &entry).await;
        }
      }
    }

    loader_context.finish_with((content, source_map, additional_data));
    Ok(())
  }
}

#[derive(Debug, Deserialize, Serialize)]
struct CacheLoaderData {
  key: String,
  remaining_request: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CacheEntry {
  remaining_request: String,
  dependencies: Vec<CacheDependency>,
  context_dependencies: Vec<CacheDependency>,
  content: CachedContent,
  source_map: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CacheDependency {
  path: String,
  mtime: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
enum CachedContent {
  String(String),
  Buffer(Vec<u8>),
}

async fn dependency_details(
  dependencies: impl IntoIterator<Item = PathBuf>,
) -> Option<Vec<CacheDependency>> {
  let mut seen = HashSet::new();
  let mut details = Vec::new();
  for dependency in dependencies {
    if !seen.insert(dependency.clone()) {
      continue;
    }
    let metadata = tokio::fs::metadata(&dependency).await.ok()?;
    let mtime = metadata_mtime_ms(&metadata)?;
    details.push(CacheDependency {
      path: dependency.to_string_lossy().into_owned(),
      mtime,
    });
  }
  Some(details)
}

fn metadata_mtime_ms(metadata: &std::fs::Metadata) -> Option<u64> {
  metadata
    .modified()
    .ok()
    .and_then(|mtime| mtime.duration_since(UNIX_EPOCH).ok())
    .map(|duration| duration.as_millis() as u64)
}

#[cfg(test)]
mod tests {
  use super::{
    CacheDependency, CacheEntry, CacheLoader, CacheLoaderOptions, CachedContent, metadata_mtime_ms,
  };

  #[test]
  fn reads_metadata_mtime() {
    let file = tempfile::NamedTempFile::new().expect("should create temp file");
    let metadata = file.as_file().metadata().expect("should read metadata");
    assert!(metadata_mtime_ms(&metadata).expect("should have mtime") > 0);
  }

  #[tokio::test]
  async fn invalidates_changed_dependencies() {
    let file = tempfile::NamedTempFile::new().expect("should create temp file");
    let metadata = file.as_file().metadata().expect("should read metadata");
    let mut entry = CacheEntry {
      remaining_request: String::new(),
      dependencies: vec![CacheDependency {
        path: file.path().to_string_lossy().into_owned(),
        mtime: metadata_mtime_ms(&metadata).expect("should have mtime"),
      }],
      context_dependencies: Vec::new(),
      content: CachedContent::String(String::new()),
      source_map: None,
    };
    let loader = CacheLoader::new("builtin:cache-loader".into(), CacheLoaderOptions::default());

    assert!(loader.is_cache_valid(&entry).await);
    entry.dependencies[0].mtime += 1;
    assert!(!loader.is_cache_valid(&entry).await);
  }
}
