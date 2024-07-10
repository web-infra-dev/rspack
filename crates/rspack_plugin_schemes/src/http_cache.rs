use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context as AnyhowContext, Error as AnyhowError};
use reqwest::Client;
use rspack_base64::encode_to_string;
use rspack_error::{error, Result};
use serde::{Deserialize, Serialize};
use sha2::{digest::Digest, Sha512};

use crate::{
  http_uri::HttpUriPluginOptions,
  lockfile::{LockfileCache, LockfileEntry},
};

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
  pub fn content(&self) -> &Vec<u8> {
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
  Redirect(RedirectFetchResult),
}

type FetchResult = Result<FetchResultType, AnyhowError>;

pub struct HttpCache {
  cache_location: Option<String>,
  lockfile_cache: LockfileCache,
}

impl HttpCache {
  pub fn new(cache_location: Option<String>) -> Self {
    HttpCache {
      cache_location,
      lockfile_cache: LockfileCache::new(),
    }
  }

  pub async fn fetch_content(&self, url: &str, options: &HttpUriPluginOptions) -> FetchResult {
    let cached_result = self.read_from_cache(url).await?;
    let cached_result_clone = cached_result.clone();
    if let Some(cached) = cached_result {
      if cached.meta.valid_until >= current_time() {
        return Ok(FetchResultType::Content(cached));
      }
    }
    self.fetch_content_raw(url, cached_result_clone).await
  }

  async fn fetch_content_raw(
    &self,
    url: &str,
    cached_result: Option<ContentFetchResult>,
  ) -> FetchResult {
    let client = Client::new();
    let request_time = current_time();
    let mut request = client.get(url);

    if let Some(cached) = &cached_result {
      if let Some(etag) = &cached.meta.etag {
        request = request.header("If-None-Match", etag);
      }
    }

    let response = request.send().await.context("Failed to send request")?;
    let status = response.status();
    let headers = response.headers().clone();
    let etag = headers
      .get("etag")
      .and_then(|v| v.to_str().ok())
      .map(String::from);
    let location = headers
      .get("location")
      .and_then(|v| v.to_str().ok())
      .map(String::from);
    let cache_control = headers
      .get("cache-control")
      .and_then(|v| v.to_str().ok())
      .map(String::from);

    let (store_lock, store_cache, valid_until) = parse_cache_control(&cache_control, request_time);

    if status == 304 {
      if let Some(cached) = cached_result {
        if cached.meta.valid_until < valid_until
          || cached.meta.store_lock != store_lock
          || cached.meta.store_cache != store_cache
          || cached.meta.etag != etag
        {
          return Ok(FetchResultType::Content(ContentFetchResult {
            meta: FetchResultMeta {
              fresh: true,
              ..cached.meta
            },
            ..cached
          }));
        }
      }
    }

    if let Some(location) = location {
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

    let content = response
      .bytes()
      .await
      .context("Failed to read response bytes")?;
    if !status.is_success() {
      return Err(anyhow::anyhow!("Request failed with status: {}", status));
    }

    let integrity = compute_integrity(&content);
    let entry = LockfileEntry {
      resolved: url.to_string(),
      integrity,
      content_type: headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string(),
    };

    let result = ContentFetchResult {
      entry: entry.clone(),
      content: content.to_vec(),
      meta: FetchResultMeta {
        fresh: true,
        store_lock,
        store_cache,
        valid_until,
        etag,
      },
    };

    if store_cache {
      self.write_to_cache(url, &result).await?;
    }

    Ok(FetchResultType::Content(result))
  }

  async fn read_from_cache(
    &self,
    resource: &str,
  ) -> Result<Option<ContentFetchResult>, anyhow::Error> {
    if let Some(cache_location) = &self.cache_location {
      let lockfile = self.lockfile_cache.get_lockfile(cache_location).await?;
      if lockfile.get_entry(resource).is_some() {
        let cache_path = format!("{}/{}", cache_location, resource.replace('/', "_"));
        if Path::new(&cache_path).exists() {
          let cached_content = fs::read_to_string(&cache_path)
            .context("Failed to read cached content")
            .map_err(|err| {
              error!("{}", err.to_string());
              err
            })?;
          let deserialized_content: ContentFetchResult = serde_json::from_str(&cached_content)
            .context("Failed to deserialize cached content")?;
          return Ok(Some(deserialized_content));
        }
      }
    }
    Ok(None)
  }

  async fn write_to_cache(
    &self,
    resource: &str,
    content: &ContentFetchResult,
  ) -> Result<(), anyhow::Error> {
    if let Some(cache_location) = &self.cache_location {
      fs::create_dir_all(cache_location).context("Failed to create cache directory")?;
      let serialized_content =
        serde_json::to_string(content).context("Failed to serialize content")?;
      let cache_path = format!("{}/{}", cache_location, resource.replace('/', "_"));
      fs::write(cache_path, serialized_content).context("Failed to write to cache")
    } else {
      Ok(())
    }
  }
}

pub async fn fetch_content(url: &str, options: &HttpUriPluginOptions) -> FetchResult {
  let http_cache = HttpCache::new(options.cache_location.clone());
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
          Some((parts.next()?.trim(), parts.next()?.trim()))
        })
        .collect();

      let store_lock = !pairs.contains_key("no-store");
      let store_cache = !pairs.contains_key("no-cache");
      let valid_until = pairs
        .get("max-age")
        .and_then(|&max_age| max_age.parse::<u64>().ok())
        .map(|seconds| request_time + seconds * 1000)
        .unwrap_or(0);

      (store_lock, store_cache, valid_until)
    })
    .unwrap_or((true, true, request_time + 3600))
}

fn current_time() -> u64 {
  use std::time::{SystemTime, UNIX_EPOCH};
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_secs()
}

fn compute_integrity(content: &[u8]) -> String {
  let mut hasher = Sha512::new();
  hasher.update(content);
  let digest = hasher.finalize();
  format!("sha512-{}", encode_to_string(digest))
}
