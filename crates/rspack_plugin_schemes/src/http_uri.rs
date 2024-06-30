use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use rspack_core::{
  ApplyContext, CompilerOptions, Content, ModuleFactoryCreateData,
  NormalModuleFactoryResolveForScheme, NormalModuleReadResource, Plugin, PluginContext,
  ResourceData,
};
use rspack_error::AnyhowError;
use rspack_hook::{plugin, plugin_hook};
use serde::{Deserialize, Serialize};

static EXTERNAL_HTTP_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(//|https?://|#)").expect("Invalid regex"));

static CACHE: Lazy<Arc<Mutex<HashMap<String, CacheEntry>>>> =
  Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

const CACHE_DURATION: Duration = Duration::from_secs(60 * 60); // Cache duration of 1 hour
const CACHE_FILE: &str = "http_uri_cache.json"; // Cache file path

#[derive(Serialize, Deserialize, Clone)]
struct CacheEntry {
  content: Vec<u8>,
  timestamp: Instant,
}

#[plugin]
#[derive(Debug, Default)]
pub struct HttpUriPlugin;

#[plugin_hook(NormalModuleFactoryResolveForScheme for HttpUriPlugin)]
async fn resolve_for_scheme(
  &self,
  _data: &mut ModuleFactoryCreateData,
  resource_data: &mut ResourceData,
) -> Result<Option<bool>> {
  if resource_data.get_scheme().is_http() && EXTERNAL_HTTP_REQUEST.is_match(&resource_data.resource)
  {
    return Ok(None);
  }
  Ok(None)
}

#[plugin_hook(NormalModuleReadResource for HttpUriPlugin)]
async fn read_resource(&self, resource_data: &ResourceData) -> Result<Option<Content>> {
  if resource_data.get_scheme().is_http() {
    let url = &resource_data.resource;
    let mut cache = CACHE.lock().unwrap();

    // Check if the URL is already in the cache and if it's still valid
    if let Some(entry) = cache.get(url) {
      if entry.timestamp.elapsed() < CACHE_DURATION {
        return Ok(Some(Content::Buffer(entry.content.clone())));
      } else {
        // Remove expired cache entry
        cache.remove(url);
      }
    }

    drop(cache); // Release the lock before making the HTTP request

    let client = Client::new();
    let response = client
      .get(url)
      .send()
      .await
      .context("Failed to send HTTP request")
      .map_err(|err| AnyhowError::from(err))?;
    let content = response
      .bytes()
      .await
      .context("Failed to read response bytes")
      .map_err(|err| AnyhowError::from(err))?
      .to_vec();

    // Store the response content in the cache
    let mut cache = CACHE.lock().unwrap();
    cache.insert(
      url.clone(),
      CacheEntry {
        content: content.clone(),
        timestamp: Instant::now(),
      },
    );

    // Save the cache to disk
    if let Err(err) = save_cache_to_disk(&*cache).await {
      eprintln!("Failed to save cache to disk: {}", err);
    }

    return Ok(Some(Content::Buffer(content)));
  }
  Ok(None)
}

async fn load_cache_from_disk() -> Result<HashMap<String, CacheEntry>> {
  if Path::new(CACHE_FILE).exists() {
    let data = fs::read(CACHE_FILE)
      .await
      .context("Failed to read cache file")?;
    let cache: HashMap<String, CacheEntry> =
      serde_json::from_slice(&data).context("Failed to deserialize cache")?;
    Ok(cache)
  } else {
    Ok(HashMap::new())
  }
}

async fn save_cache_to_disk(cache: &HashMap<String, CacheEntry>) -> Result<()> {
  let data = serde_json::to_vec(cache).context("Failed to serialize cache")?;
  fs::write(CACHE_FILE, data)
    .await
    .context("Failed to write cache file")?;
  Ok(())
}

fn parse_cache_control(
  cache_control: Option<&str>,
  request_time: Instant,
) -> (bool, bool, Instant) {
  let mut store_cache = true;
  let mut store_lock = true;
  let mut valid_until = Instant::now();

  if let Some(cache_control) = cache_control {
    let parsed = parse_key_value_pairs(cache_control);
    if parsed.get("no-cache").is_some() {
      store_cache = false;
      store_lock = false;
    }
    if let Some(max_age) = parsed.get("max-age") {
      if let Ok(max_age) = max_age.parse::<u64>() {
        valid_until = request_time + Duration::from_secs(max_age);
      }
    }
    if parsed.get("must-revalidate").is_some() {
      valid_until = request_time;
    }
  }

  (store_cache, store_lock, valid_until)
}

fn parse_key_value_pairs(input: &str) -> HashMap<String, String> {
  input
    .split(',')
    .map(|item| {
      let mut parts = item.splitn(2, '=');
      let key = parts.next().unwrap().trim().to_string();
      let value = parts.next().unwrap_or("").trim().to_string();
      (key, value)
    })
    .collect()
}

#[async_trait::async_trait]
impl Plugin for HttpUriPlugin {
  fn name(&self) -> &'static str {
    "rspack.HttpUriPlugin"
  }

  async fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    // Load the cache from disk
    let loaded_cache = load_cache_from_disk().await?;
    let mut cache = CACHE.lock().unwrap();
    *cache = loaded_cache;
    drop(cache);

    ctx
      .context
      .normal_module_factory_hooks
      .resolve_for_scheme
      .tap(resolve_for_scheme::new(self));
    ctx
      .context
      .normal_module_hooks
      .read_resource
      .tap(read_resource::new(self));
    Ok(())
  }
}
