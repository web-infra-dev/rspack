use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use async_trait::async_trait;
use cow_utils::CowUtils;
use rspack_base64::encode_to_string;
use rspack_fs::WritableFileSystem;
use rspack_paths::Utf8Path;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

use crate::{
  http_uri::HttpUriPluginOptions,
  lockfile::{LockfileCache, LockfileEntry},
};

pub struct HttpRequest {
  pub url: String,
  pub headers: HashMap<String, String>,
}

pub struct HttpResponse {
  pub status: u16,
  pub headers: HashMap<String, String>,
  pub body: Vec<u8>,
}

#[async_trait]
pub trait HttpClient: Send + Sync + std::fmt::Debug {
  async fn get(&self, url: &str, headers: &HashMap<String, String>) -> Result<HttpResponse>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FetchResultMeta {
  store_cache: bool,
  store_lock: bool,
  valid_until: u64,
  etag: Option<String>,
  fresh: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentFetchResult {
  entry: LockfileEntry,
  content: Vec<u8>,
  meta: FetchResultMeta,
}

impl ContentFetchResult {
  pub fn content(&self) -> &[u8] {
    &self.content
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedirectFetchResult {
  location: String,
  meta: FetchResultMeta,
}

pub enum FetchResultType {
  Content(ContentFetchResult),
  #[allow(dead_code)]
  Redirect(RedirectFetchResult),
}

pub struct HttpCache {
  cache_location: Option<PathBuf>,
  lockfile_cache: LockfileCache,
  filesystem: Arc<dyn WritableFileSystem + Send + Sync>,
  http_client: Arc<dyn HttpClient>,
}

impl HttpCache {
  pub fn new(
    cache_location: Option<String>,
    lockfile_location: Option<String>,
    filesystem: Arc<dyn WritableFileSystem + Send + Sync>,
    http_client: Option<Arc<dyn HttpClient>>,
  ) -> Self {
    let cache_location = cache_location.map(PathBuf::from);
    let lockfile_path = lockfile_location.map(PathBuf::from);
    HttpCache {
      cache_location,
      lockfile_cache: LockfileCache::new(lockfile_path, filesystem.clone()),
      filesystem: filesystem.clone(),
      http_client: http_client.expect("http_client must be provided"),
    }
  }

  pub async fn fetch_content(
    &self,
    url: &str,
    _options: &HttpUriPluginOptions,
  ) -> Result<FetchResultType> {
    let cached_result = self.read_from_cache(url).await?;

    if let Some(ref cached) = cached_result {
      if cached.meta.fresh {
        return Ok(FetchResultType::Content(cached.clone()));
      }
    }

    self.fetch_content_raw(url, cached_result).await
  }

  async fn fetch_content_raw(
    &self,
    url: &str,
    cached_result: Option<ContentFetchResult>,
  ) -> Result<FetchResultType> {
    let request_time = current_time();
    let mut headers = HashMap::new();

    if let Some(cached) = &cached_result {
      if let Some(etag) = &cached.meta.etag {
        headers.insert("If-None-Match".to_string(), etag.clone());
      }
    }

    let response = self.http_client.get(url, &headers).await?;
    let status = response.status;
    let headers = response.headers;
    let etag = headers.get("etag").cloned();
    let location = headers.get("location").cloned();
    let cache_control = headers.get("cache-control").cloned();

    let (store_lock, store_cache, valid_until) = parse_cache_control(&cache_control, request_time);

    if status == 304 {
      if let Some(cached) = cached_result {
        let new_valid_until = valid_until.max(cached.meta.valid_until);
        return Ok(FetchResultType::Content(ContentFetchResult {
          meta: FetchResultMeta {
            fresh: true,
            store_lock,
            store_cache,
            valid_until: new_valid_until,
            etag: etag.or(cached.meta.etag),
          },
          ..cached
        }));
      }
    }

    if let Some(location) = location {
      if (300..=399).contains(&status) {
        return Ok(FetchResultType::Redirect(RedirectFetchResult {
          location,
          meta: FetchResultMeta {
            fresh: true,
            store_lock,
            store_cache,
            valid_until,
            etag,
          },
        }));
      }
    }

    if !(200..=299).contains(&status) {
      return Err(anyhow::anyhow!("Request failed with status: {}", status));
    }

    let content = response.body;
    let integrity = compute_integrity(&content);
    let entry = LockfileEntry {
      resolved: url.to_string(),
      integrity: integrity.clone(),
      content_type: headers
        .get("content-type")
        .unwrap_or(&"".to_string())
        .to_string(),
      valid_until,
      etag: etag.clone(),
    };

    let result = ContentFetchResult {
      entry: entry.clone(),
      content: content.to_vec(),
      meta: FetchResultMeta {
        fresh: true,
        store_lock,
        store_cache,
        valid_until,
        etag: etag.clone(),
      },
    };

    if store_cache || store_lock {
      let should_update = cached_result
        .map(|cached| {
          valid_until > cached.meta.valid_until
            || etag != cached.meta.etag
            || integrity != cached.entry.integrity
        })
        .unwrap_or(true);

      if should_update {
        if store_cache {
          self.write_to_cache(url, &result.content).await?;
        }
        let lockfile = self.lockfile_cache.get_lockfile().await?;
        lockfile
          .lock()
          .await
          .entries_mut()
          .insert(url.to_string(), entry);
        self.lockfile_cache.save_lockfile().await?;
      }
    }

    Ok(FetchResultType::Content(result))
  }

  async fn read_from_cache(&self, resource: &str) -> Result<Option<ContentFetchResult>> {
    if let Some(cache_location) = &self.cache_location {
      let lockfile = self.lockfile_cache.get_lockfile().await?;
      let lockfile = lockfile.lock().await;
      let cache_path_buf = cache_location.join(Path::new(&*resource.cow_replace('/', "_")));
      let cache_path = Utf8Path::from_path(&cache_path_buf).expect("Invalid cache path");

      if let Some(entry) = lockfile.get_entry(resource) {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let is_valid = entry.valid_until > current_time;

        if is_valid && self.filesystem.read_file(cache_path).await.is_ok() {
          let cached_content = self
            .filesystem
            .read_file(cache_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read cached content: {:?}", e))?;

          let result = ContentFetchResult {
            entry: entry.clone(),
            content: cached_content,
            meta: FetchResultMeta {
              fresh: true,
              store_cache: false,
              store_lock: false,
              valid_until: entry.valid_until,
              etag: entry.etag.clone(),
            },
          };
          return Ok(Some(result));
        }
      }
    }
    Ok(None)
  }

  async fn write_to_cache(&self, resource: &str, content: &[u8]) -> Result<()> {
    if let Some(cache_location) = &self.cache_location {
      let cache_location_path =
        Utf8Path::from_path(cache_location).expect("Invalid cache location path");
      self
        .filesystem
        .create_dir_all(cache_location_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create cache directory: {:?}", e))?;
      let cache_path_buf = cache_location.join(Path::new(&*resource.cow_replace('/', "_")));
      let cache_path = Utf8Path::from_path(&cache_path_buf).expect("Invalid cache path");
      self
        .filesystem
        .write(cache_path, content)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to write to cache: {:?}", e))?;
    }
    Ok(())
  }
}

pub async fn fetch_content(url: &str, options: &HttpUriPluginOptions) -> Result<FetchResultType> {
  let http_cache = HttpCache::new(
    options.cache_location.clone(),
    options.lockfile_location.clone(),
    options.filesystem.clone(),
    options.http_client.clone(),
  );

  http_cache.fetch_content(url, options).await
}

fn parse_cache_control(cache_control: &Option<String>, request_time: u64) -> (bool, bool, u64) {
  cache_control
    .as_ref()
    .map(|header| {
      let pairs: HashMap<_, _> = header
        .split(',')
        .filter_map(|part| {
          let mut parts = part.splitn(2, '=');
          Some((
            parts.next()?.trim().cow_to_ascii_lowercase(),
            parts.next().map(|v| v.trim().to_string()),
          ))
        })
        .collect();

      let store_lock = !pairs.contains_key("no-cache");
      let store_cache = !pairs.contains_key("no-cache");
      let valid_until = if pairs.contains_key("must-revalidate") {
        0
      } else {
        pairs
          .get("max-age")
          .and_then(|max_age| max_age.as_ref().and_then(|v| v.parse::<u64>().ok()))
          .map(|seconds| request_time + seconds * 1000)
          .unwrap_or(request_time)
      };

      (store_lock, store_cache, valid_until)
    })
    .unwrap_or((true, true, request_time))
}

fn current_time() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_millis() as u64
}

fn compute_integrity(content: &[u8]) -> String {
  let mut hasher = Sha512::new();
  hasher.update(content);
  let digest = hasher.finalize();
  format!("sha512-{}", encode_to_string(digest))
}
