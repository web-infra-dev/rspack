use std::{collections::HashMap, fmt::Debug, path::PathBuf, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use cow_utils::CowUtils;
use napi::bindgen_prelude::Buffer;
use rspack_fs::WritableFileSystem;
use rspack_paths::Utf8Path;
use rspack_util::{base64, current_time, fx_hash::FxHashMap};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use url::Url;

use super::lockfile::{LockfileCache, LockfileEntry};
use crate::http_uri::HttpUriPluginOptions;

/// This enum is used for avoiding [Buffer::to_vec] overhead
pub(super) enum BufferOrBytes {
  Buffer(Buffer),
  Bytes(Vec<u8>),
}

impl Clone for BufferOrBytes {
  fn clone(&self) -> Self {
    match self {
      Self::Buffer(buffer) => Self::Bytes(buffer.to_vec()),
      Self::Bytes(bytes) => Self::Bytes(bytes.clone()),
    }
  }
}

pub struct HttpResponse {
  pub status: u16,
  pub headers: FxHashMap<String, String>,
  pub body: Buffer,
}

#[async_trait]
pub trait HttpClient: Send + Sync + std::fmt::Debug {
  async fn get(&self, url: &str, headers: &FxHashMap<String, String>) -> Result<HttpResponse>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FetchResultMeta {
  store_cache: bool,
  store_lock: bool,
  valid_until: u64,
  etag: Option<String>,
  fresh: bool,
}

#[derive(Clone)]
pub(super) struct ContentFetchResult {
  pub(crate) entry: LockfileEntry,
  content: BufferOrBytes,
  meta: FetchResultMeta,
}

impl ContentFetchResult {
  #[inline(always)]
  pub(super) fn content(&self) -> &[u8] {
    match &self.content {
      BufferOrBytes::Buffer(buffer) => buffer.as_ref(),
      BufferOrBytes::Bytes(items) => items.as_ref(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct RedirectFetchResult {
  pub(crate) location: String,
  meta: FetchResultMeta,
}

pub(super) enum FetchResultType {
  Content(ContentFetchResult),
  #[allow(dead_code)]
  Redirect(RedirectFetchResult),
}

pub(super) struct HttpCache {
  cache_location: Option<PathBuf>,
  lockfile_cache: LockfileCache,
  filesystem: Arc<dyn WritableFileSystem + Send + Sync>,
  http_client: Arc<dyn HttpClient>,
}

impl HttpCache {
  pub(super) fn new(
    cache_location: Option<String>,
    lockfile_location: Option<String>,
    filesystem: Arc<dyn WritableFileSystem + Send + Sync>,
    http_client: Arc<dyn HttpClient>,
  ) -> Self {
    let cache_location = cache_location.map(PathBuf::from);
    let lockfile_path = lockfile_location.map(PathBuf::from);
    HttpCache {
      cache_location,
      lockfile_cache: LockfileCache::new(lockfile_path, filesystem.clone()),
      filesystem: filesystem.clone(),
      http_client,
    }
  }

  pub(super) async fn fetch_content(
    &self,
    url: &str,
    options: &HttpUriPluginOptions,
  ) -> Result<FetchResultType> {
    let cached_result = self.read_from_cache(url).await?;

    if let Some(ref cached) = cached_result
      && (!options.upgrade || cached.meta.fresh)
    {
      return Ok(FetchResultType::Content(cached.clone()));
    }

    self.fetch_content_raw(url, cached_result).await
  }

  async fn fetch_content_raw(
    &self,
    url: &str,
    cached_result: Option<ContentFetchResult>,
  ) -> Result<FetchResultType> {
    let request_time = current_time();
    let mut headers = FxHashMap::default();

    // Add webpack-like headers
    headers.insert(
      "accept-encoding".to_string(),
      "gzip, deflate, br".to_string(),
    );
    headers.insert("user-agent".to_string(), "webpack".to_string());

    if let Some(cached) = &cached_result
      && let Some(etag) = &cached.meta.etag
    {
      headers.insert("if-none-match".to_string(), etag.clone());
    }

    let response = self.http_client.get(url, &headers).await?;
    let status = response.status;
    let headers = response.headers;
    let etag = headers.get("etag").cloned();
    let location = headers.get("location").cloned();
    let cache_control = headers.get("cache-control").cloned();

    let (store_lock, store_cache, valid_until) = parse_cache_control(&cache_control, request_time);

    // Handle 304 Not Modified (similar to webpack)
    if status == 304
      && let Some(cached) = cached_result
    {
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

    // Improved handling of redirects to match webpack
    if let Some(location) = location
      && (301..=308).contains(&status)
    {
      // Resolve relative redirects like webpack does
      let absolute_location = match Url::parse(&location) {
        Ok(loc) => loc.to_string(), // Already absolute
        Err(_) => {
          // Relative URL, resolve against original
          match Url::parse(url) {
            Ok(base_url) => base_url
              .join(&location)
              .map(|u| u.to_string())
              .unwrap_or(location.clone()),
            Err(_) => location.clone(), // Can't resolve, use as is
          }
        }
      };

      // If we had a cached redirect that's unchanged, use the cached meta
      // if let Some(cached) = &cached_result
      //   && let FetchResultType::Redirect(cached_redirect) =
      //     fetch_cache_result_to_fetch_result_type(cached)
      //   && cached_redirect.location == absolute_location
      //   && cached_redirect.meta.valid_until >= valid_until
      //   && cached_redirect.meta.store_lock == store_lock
      //   && cached_redirect.meta.store_cache == store_cache
      //   && cached_redirect.meta.etag == etag
      // {
      //   return Ok(FetchResultType::Redirect(RedirectFetchResult {
      //     meta: FetchResultMeta {
      //       fresh: true,
      //       ..cached_redirect.meta
      //     },
      //     ..cached_redirect
      //   }));
      // }

      return Ok(FetchResultType::Redirect(RedirectFetchResult {
        location: absolute_location,
        meta: FetchResultMeta {
          fresh: true,
          store_lock,
          store_cache,
          valid_until,
          etag,
        },
      }));
    }

    if !(200..=299).contains(&status) {
      return Err(anyhow::anyhow!(
        "Request failed with status: {}\n{}",
        status,
        String::from_utf8_lossy(&response.body)
      ));
    }

    let content = response.body;
    let integrity = compute_integrity(&content);
    let content_type = headers
      .get("content-type")
      .unwrap_or(&String::new())
      .clone();

    let entry = LockfileEntry {
      resolved: url.to_string(),
      integrity: integrity.clone(),
      content_type,
      valid_until,
      etag: etag.clone(),
    };

    let result = ContentFetchResult {
      entry: entry.clone(),
      content: BufferOrBytes::Buffer(content),
      meta: FetchResultMeta {
        fresh: true,
        store_lock,
        store_cache,
        valid_until,
        etag: etag.clone(),
      },
    };

    if store_cache || store_lock {
      let should_update = cached_result.is_none_or(|cached| {
        valid_until > cached.meta.valid_until
          || etag != cached.meta.etag
          || integrity != cached.entry.integrity
      });

      if should_update {
        if store_cache {
          self.write_to_cache(url, result.content()).await?;
        }

        let lockfile = self.lockfile_cache.get_lockfile().await?;
        let mut lock_guard = lockfile.lock().await;

        // Update the lockfile entry
        lock_guard.entries_mut().insert(url.to_string(), entry);
        drop(lock_guard);
        self.lockfile_cache.save_lockfile().await?;
      }
    }

    Ok(FetchResultType::Content(result))
  }

  async fn read_from_cache(&self, resource: &str) -> Result<Option<ContentFetchResult>> {
    if let Some(cache_location) = &self.cache_location {
      let lockfile = self.lockfile_cache.get_lockfile().await?;
      let lock_guard = lockfile.lock().await;

      if let Some(entry) = lock_guard.get_entry(resource) {
        let cache_key = self.get_cache_key(&entry.resolved);
        let cache_path_buf = cache_location.join(&cache_key);
        let cache_path = Utf8Path::from_path(&cache_path_buf).expect("Invalid cache path");

        if let Ok(content) = self.filesystem.read_file(cache_path).await {
          let meta = FetchResultMeta {
            store_cache: true,
            store_lock: true,
            valid_until: entry.valid_until,
            etag: entry.etag.clone(),
            fresh: entry.valid_until >= current_time(),
          };

          let result = ContentFetchResult {
            entry: entry.clone(),
            content: BufferOrBytes::Bytes(content),
            meta,
          };

          return Ok(Some(result));
        }
      }
    }
    Ok(None)
  }

  async fn write_to_cache(&self, resource: &str, content: &[u8]) -> Result<()> {
    if let Some(cache_location) = &self.cache_location {
      // Generate cache key using webpack-compatible format
      let cache_key = self.get_cache_key(resource);

      // Create the full path to the cache file
      let cache_path_buf = PathBuf::from(cache_location).join(&cache_key);
      let cache_path = Utf8Path::from_path(&cache_path_buf).expect("Invalid cache path");

      // Create parent directories
      if let Some(parent) = cache_path.parent() {
        let parent_path = parent.to_string();
        let parent_utf8_path = Utf8Path::new(&parent_path);
        self.filesystem.create_dir_all(parent_utf8_path).await.ok();
      }

      // Write the cache file
      self.filesystem.write(cache_path, content).await.ok();
    }
    Ok(())
  }

  /// Get a cache key for a URL, compatible with webpack's getCacheKey function
  fn get_cache_key(&self, url_str: &str) -> String {
    // Parse the URL
    let url = match Url::parse(url_str) {
      Ok(url) => url,
      Err(_) => {
        let digest = Sha512::digest(url_str.as_bytes());
        let hex_digest = self.to_hex_string(&digest)[..20].to_string();
        return format!("invalid-url_{hex_digest}");
      }
    };

    // Extract components similar to webpack's _getCacheKey function
    let folder = self.to_safe_path(&url.origin().ascii_serialization());
    let pathname = self.to_safe_path(url.path());

    // Extract query (search part)
    let query = self.to_safe_path(url.query().unwrap_or(""));

    // Get extension using the Path functionality, just like webpack does
    let path = std::path::Path::new(pathname.as_str());
    let ext_opt = path.extension().and_then(|e| e.to_str());

    // Limit extension to 20 chars as webpack does
    let ext = if let Some(ext) = ext_opt {
      let ext_str = format!(".{ext}");
      if ext_str.len() > 20 {
        String::new()
      } else {
        ext_str
      }
    } else {
      String::new()
    };

    // Create basename similar to webpack
    let basename = if !ext.is_empty() && pathname.ends_with(&ext) {
      pathname[..pathname.len() - ext.len()].to_string()
    } else {
      pathname
    };

    // Create hash of URL for uniqueness using hex encoding like webpack
    let mut hasher = Sha512::new();
    hasher.update(url_str.as_bytes());
    let digest = hasher.finalize();
    // Convert to hex string and take first 20 chars
    let hash_digest = self.to_hex_string(&digest)[..20].to_string();

    // Construct the final key exactly as webpack does
    // Take only the last 50 chars of the folder
    let folder_component = if folder.len() > 50 {
      folder[folder.len() - 50..].to_string()
    } else {
      folder
    };

    // Combine basename and query, limited to 150 chars
    let name_component = if !query.is_empty() {
      format!("{basename}_{query}")
    } else {
      basename
    };
    let name_component = if name_component.len() > 150 {
      name_component[..150].to_string()
    } else {
      name_component
    };

    format!("{folder_component}/{name_component}_{hash_digest}{ext}")
  }

  /// Convert a string to a safe path component (similar to webpack's toSafePath)
  fn to_safe_path(&self, input: &str) -> String {
    input
      .cow_replace(
        &[':', '/', '\\', '<', '>', ':', '"', '|', '?', '*', '\0'] as &[char],
        "_",
      )
      .into_owned()
  }

  /// Convert a byte array to a hex string
  fn to_hex_string(&self, bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for b in bytes {
      use std::fmt::Write;
      write!(result, "{b:02x}").expect("write hex failed");
    }
    result
  }
}

pub(super) async fn fetch_content(
  url: &str,
  options: &HttpUriPluginOptions,
) -> Result<FetchResultType> {
  let http_cache = HttpCache::new(
    options.cache_location.clone(),
    options.lockfile_location.clone(),
    options.filesystem.clone(),
    options.http_client.clone(),
  );

  http_cache.fetch_content(url, options).await
}

fn parse_cache_control(cache_control: &Option<String>, request_time: u64) -> (bool, bool, u64) {
  cache_control.as_ref().map_or((true, true, 0), |header| {
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
        .map_or(request_time, |seconds| request_time + seconds * 1000)
    };

    (store_lock, store_cache, valid_until)
  })
}

fn compute_integrity(content: &[u8]) -> String {
  let mut hasher = Sha512::new();
  hasher.update(content);
  let digest = hasher.finalize();
  // Use base64 for integrity as that's the standard format
  format!("sha512-{}", base64::encode_to_string(digest))
}
