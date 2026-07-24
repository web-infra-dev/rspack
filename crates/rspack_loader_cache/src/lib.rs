use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
  sync::{
    Arc, LazyLock, RwLock,
    atomic::{AtomicU64, Ordering},
  },
  time::UNIX_EPOCH,
};

use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  CacheOptions, Content, Loader, LoaderContext, RunnerContext, rspack_sources::SourceMap,
};
use rspack_error::Result;
use rspack_loader_runner::{DisplayWithSuffix, Identifier};
use serde::{Deserialize, Serialize};

mod plugin;

pub use plugin::CacheLoaderPlugin;

pub const CACHE_LOADER_IDENTIFIER: &str = "builtin:cache-loader";

type CacheStore = Arc<RwLock<HashMap<String, Arc<CacheEntry>>>>;

static NEXT_CACHE_ID: AtomicU64 = AtomicU64::new(1);
static CACHE_STORES: LazyLock<RwLock<HashMap<u64, CacheStore>>> =
  LazyLock::new(|| RwLock::new(HashMap::new()));

pub(crate) fn create_cache() -> u64 {
  let cache_id = NEXT_CACHE_ID.fetch_add(1, Ordering::Relaxed);
  CACHE_STORES
    .write()
    .expect("cache loader stores should not be poisoned")
    .insert(cache_id, CacheStore::default());
  cache_id
}

pub(crate) fn remove_cache(cache_id: u64) {
  CACHE_STORES
    .write()
    .expect("cache loader stores should not be poisoned")
    .remove(&cache_id);
}

fn cache_store(cache_id: u64) -> CacheStore {
  CACHE_STORES
    .write()
    .expect("cache loader stores should not be poisoned")
    .entry(cache_id)
    .or_default()
    .clone()
}

#[cacheable]
#[derive(Debug)]
pub struct CacheLoader {
  identifier: Identifier,
  cache_id: u64,
}

impl CacheLoader {
  pub fn new(identifier: Identifier, cache_id: u64) -> Self {
    debug_assert!(identifier.starts_with(CACHE_LOADER_IDENTIFIER));
    Self {
      identifier,
      cache_id,
    }
  }

  fn cache(&self) -> CacheStore {
    cache_store(self.cache_id)
  }

  fn should_cache(loader_context: &LoaderContext<RunnerContext>) -> bool {
    matches!(&loader_context.context.options.cache, CacheOptions::Disabled)
  }

  async fn is_cache_valid(entry: &CacheEntry) -> bool {
    for dependency in entry
      .dependencies
      .iter()
      .chain(&entry.context_dependencies)
    {
      let Ok(metadata) = tokio::fs::metadata(&dependency.path).await else {
        return false;
      };
      if metadata_mtime(&metadata) != Some(dependency.mtime) {
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
    if !Self::should_cache(loader_context) {
      return Ok(());
    }

    let key = loader_context
      .remaining_request()
      .display_with_suffix(loader_context.resource());
    let data = CacheLoaderData { key: key.clone() };
    let loader_index = loader_context.loader_index as usize;
    loader_context.loader_items[loader_index]
      .set_data(serde_json::to_value(data).expect("cache loader data should be serializable"));

    let cache = self.cache();
    let entry = cache
      .read()
      .expect("cache loader store should not be poisoned")
      .get(&key)
      .cloned();
    let Some(entry) = entry else {
      return Ok(());
    };

    if !Self::is_cache_valid(&entry).await {
      cache
        .write()
        .expect("cache loader store should not be poisoned")
        .remove(&key);
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

    let source_map = match &entry.source_map {
      Some(source_map) => match SourceMap::from_json(source_map.clone()) {
        Ok(source_map) => Some(source_map),
        Err(_) => {
          cache
            .write()
            .expect("cache loader store should not be poisoned")
            .remove(&key);
          return Ok(());
        }
      },
      None => None,
    };
    loader_context.finish_with((entry.content.clone(), source_map, None));
    Ok(())
  }

  async fn run(&self, loader_context: &mut LoaderContext<RunnerContext>) -> Result<()> {
    let (content, source_map, additional_data) = loader_context.take_all();
    let Some(content) = content else {
      return Ok(());
    };

    if Self::should_cache(loader_context)
      && loader_context.cacheable
      && additional_data.is_none()
      && let Ok(data) = serde_json::from_value::<CacheLoaderData>(
        loader_context.current_loader().data().clone(),
      )
    {
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
        let entry = Arc::new(CacheEntry {
          dependencies,
          context_dependencies,
          content: content.clone(),
          source_map: source_map.as_ref().map(SourceMap::to_json),
        });
        self
          .cache()
          .write()
          .expect("cache loader store should not be poisoned")
          .insert(data.key, entry);
      }
    }

    loader_context.finish_with((content, source_map, additional_data));
    Ok(())
  }
}

#[derive(Debug, Deserialize, Serialize)]
struct CacheLoaderData {
  key: String,
}

#[derive(Debug, Clone)]
struct CacheEntry {
  dependencies: Vec<CacheDependency>,
  context_dependencies: Vec<CacheDependency>,
  content: Content,
  source_map: Option<String>,
}

#[derive(Debug, Clone)]
struct CacheDependency {
  path: String,
  mtime: u128,
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
    let mtime = metadata_mtime(&metadata)?;
    details.push(CacheDependency {
      path: dependency.to_string_lossy().into_owned(),
      mtime,
    });
  }
  Some(details)
}

fn metadata_mtime(metadata: &std::fs::Metadata) -> Option<u128> {
  metadata
    .modified()
    .ok()
    .and_then(|mtime| mtime.duration_since(UNIX_EPOCH).ok())
    .map(|duration| duration.as_nanos())
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use rspack_core::Content;

  use super::{
    CacheDependency, CacheEntry, CacheLoader, create_cache, metadata_mtime, remove_cache,
  };

  #[test]
  fn shares_cache_between_loaders() {
    let cache_id = create_cache();
    let first = CacheLoader::new("builtin:cache-loader".into(), cache_id);
    let second = CacheLoader::new("builtin:cache-loader".into(), cache_id);
    first
      .cache()
      .write()
      .expect("cache loader store should not be poisoned")
      .insert(
        "key".to_string(),
        Arc::new(CacheEntry {
          dependencies: Vec::new(),
          context_dependencies: Vec::new(),
          content: Content::String("cached".to_string()),
          source_map: None,
        }),
      );

    assert!(
      second
        .cache()
        .read()
        .expect("cache loader store should not be poisoned")
        .contains_key("key")
    );
    remove_cache(cache_id);
  }

  #[tokio::test]
  async fn invalidates_changed_dependencies() {
    let file = tempfile::NamedTempFile::new().expect("should create temp file");
    let metadata = file.as_file().metadata().expect("should read metadata");
    let mut entry = CacheEntry {
      dependencies: vec![CacheDependency {
        path: file.path().to_string_lossy().into_owned(),
        mtime: metadata_mtime(&metadata).expect("should have mtime"),
      }],
      context_dependencies: Vec::new(),
      content: Content::String(String::new()),
      source_map: None,
    };

    assert!(CacheLoader::is_cache_valid(&entry).await);
    entry.dependencies[0].mtime += 1;
    assert!(!CacheLoader::is_cache_valid(&entry).await);
  }
}
