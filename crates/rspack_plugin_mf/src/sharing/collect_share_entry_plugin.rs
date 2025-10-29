use std::{
  collections::hash_map::Entry,
  sync::{Arc, LazyLock},
};

use regex::Regex;
use rspack_core::{
  Compilation, CompilationAsset, CompilationProcessAssets, CompilerCompilation, Logger,
  ModuleFactoryCreateData, NormalModuleCreateData, NormalModuleFactoryModule, Plugin,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;
use tokio::sync::RwLock;

use super::provide_shared_plugin::{ProvideOptions, ProvideVersion};

static RELATIVE_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(?:\.\.?(?:/|$))").expect("invalid relative request regex"));
static ABSOLUTE_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(?:/|[A-Za-z]:\\|\\\\)").expect("invalid absolute request regex"));

const DEFAULT_FILENAME: &str = "collect-share-entries.json";

#[derive(Debug, Clone)]
struct CollectShareEntryMeta {
  request: String,
  share_key: String,
  share_scope: String,
  is_prefix: bool,
  provide: ProvideOptions,
}

#[derive(Debug, Clone, Default)]
struct CollectShareEntryRecord {
  share_scope: String,
  requests: FxHashSet<CollectedShareRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CollectedShareRequest {
  request: String,
  version: String,
}

#[derive(Debug, Serialize)]
struct CollectShareEntryAsset<'a> {
  shared: FxHashMap<&'a str, CollectShareEntryAssetItem<'a>>,
}

#[derive(Debug, Serialize)]
struct CollectShareEntryAssetItem<'a> {
  #[serde(rename = "shareScope")]
  share_scope: &'a str,
  requests: &'a [[String; 2]],
}

#[derive(Debug)]
pub struct CollectShareEntryPluginOptions {
  pub provides: Vec<(String, ProvideOptions)>,
  pub filename: Option<String>,
}

#[plugin]
#[derive(Debug)]
pub struct CollectShareEntryPlugin {
  provides: Arc<Vec<CollectShareEntryMeta>>,
  match_provides: RwLock<FxHashMap<String, CollectShareEntryMeta>>,
  prefix_match_provides: RwLock<FxHashMap<String, CollectShareEntryMeta>>,
  resolved_entries: RwLock<FxHashMap<String, CollectShareEntryRecord>>,
  filename: String,
}

impl CollectShareEntryPlugin {
  pub fn new(options: CollectShareEntryPluginOptions) -> Self {
    let provides: Vec<CollectShareEntryMeta> = options
      .provides
      .into_iter()
      .map(|(request, provide)| {
        let provide = provide.clone();
        let share_key = provide.share_key.clone();
        let share_scope = provide.share_scope.clone();
        let is_prefix = request.ends_with('/');
        CollectShareEntryMeta {
          request,
          share_key,
          share_scope,
          is_prefix,
          provide,
        }
      })
      .collect();

    Self::new_inner(
      Arc::new(provides),
      Default::default(),
      Default::default(),
      Default::default(),
      options
        .filename
        .unwrap_or_else(|| DEFAULT_FILENAME.to_string()),
    )
  }

  async fn record_entry(
    &self,
    share_key: String,
    share_scope: String,
    request: String,
    provide: &ProvideOptions,
    resource_data: &rspack_loader_runner::ResourceData,
  ) {
    let Some(version) = infer_version(provide, resource_data) else {
      return;
    };
    let mut resolved_entries = self.resolved_entries.write().await;
    match resolved_entries.entry(share_key) {
      Entry::Occupied(mut entry) => {
        let record = entry.get_mut();
        record.share_scope = share_scope;
        record
          .requests
          .insert(CollectedShareRequest { request, version });
      }
      Entry::Vacant(entry) => {
        let mut requests = FxHashSet::default();
        requests.insert(CollectedShareRequest { request, version });
        entry.insert(CollectShareEntryRecord {
          share_scope,
          requests,
        });
      }
    }
  }
}

#[plugin_hook(CompilerCompilation for CollectShareEntryPlugin)]
async fn compilation(
  &self,
  _compilation: &mut Compilation,
  _params: &mut rspack_core::CompilationParams,
) -> Result<()> {
  {
    let mut match_provides = self.match_provides.write().await;
    match_provides.clear();
    for meta in self.provides.iter() {
      if !meta.is_prefix
        && !RELATIVE_REQUEST.is_match(&meta.request)
        && !ABSOLUTE_REQUEST.is_match(&meta.request)
      {
        match_provides.insert(meta.request.clone(), meta.clone());
      }
    }
  }

  {
    let mut prefix_matches = self.prefix_match_provides.write().await;
    prefix_matches.clear();
    for meta in self.provides.iter() {
      if meta.is_prefix {
        prefix_matches.insert(meta.request.clone(), meta.clone());
      }
    }
  }

  {
    let mut resolved_entries = self.resolved_entries.write().await;
    resolved_entries.clear();
    for meta in self.provides.iter() {
      resolved_entries
        .entry(meta.share_key.clone())
        .or_insert_with(|| CollectShareEntryRecord {
          share_scope: meta.share_scope.clone(),
          requests: FxHashSet::default(),
        });
    }
  }
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for CollectShareEntryPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let entries = self.resolved_entries.read().await;

  let mut shared: FxHashMap<&str, CollectShareEntryAssetItem<'_>> = FxHashMap::default();
  let mut ordered_requests: FxHashMap<&str, Vec<[String; 2]>> = FxHashMap::default();

  for (share_key, record) in entries.iter() {
    if record.requests.is_empty() {
      continue;
    }
    let mut requests: Vec<[String; 2]> = record
      .requests
      .iter()
      .map(|item| [item.request.clone(), item.version.clone()])
      .collect();
    requests.sort_by(|a, b| a[0].cmp(&b[0]).then(a[1].cmp(&b[1])));
    ordered_requests.insert(share_key.as_str(), requests);
  }

  for (share_key, record) in entries.iter() {
    if record.requests.is_empty() {
      continue;
    }
    let requests = ordered_requests
      .get(share_key.as_str())
      .map(|v| v.as_slice())
      .unwrap_or(&[]);
    shared.insert(
      share_key.as_str(),
      CollectShareEntryAssetItem {
        share_scope: record.share_scope.as_str(),
        requests,
      },
    );
  }

  let asset = CollectShareEntryAsset { shared };
  let json = serde_json::to_string_pretty(&asset)
    .expect("CollectShareEntryPlugin: failed to serialize share entries");

  compilation.emit_asset(
    self.filename.clone(),
    CompilationAsset::new(
      Some(RawStringSource::from(json).boxed()),
      Default::default(),
    ),
  );
  Ok(())
}

#[plugin_hook(NormalModuleFactoryModule for CollectShareEntryPlugin)]
async fn normal_module_factory_module(
  &self,
  _data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
  _module: &mut rspack_core::BoxModule,
) -> Result<()> {
  let resource_data = &create_data.resource_resolve_data;
  if resource_data.resource().is_empty() {
    return Ok(());
  }

  let request = create_data.raw_request.clone();
  let resource = resource_data.resource().to_string();

  // Exact match
  if let Some(meta) = self.match_provides.read().await.get(&request).cloned() {
    self
      .record_entry(
        meta.share_key,
        meta.share_scope,
        resource.clone(),
        &meta.provide,
        resource_data,
      )
      .await;
    return Ok(());
  }

  // Prefix match
  let mut matched_prefix: Option<(CollectShareEntryMeta, String)> = None;
  {
    let prefix_matches = self.prefix_match_provides.read().await;
    for (prefix, meta) in prefix_matches.iter() {
      if request.starts_with(prefix) {
        let remainder = request[prefix.len()..].to_string();
        matched_prefix = Some((meta.clone(), remainder));
        break;
      }
    }
  }

  if let Some((meta, remainder)) = matched_prefix {
    let share_key = format!("{}{}", meta.share_key, remainder);
    self
      .record_entry(
        share_key,
        meta.share_scope,
        resource.clone(),
        &meta.provide,
        resource_data,
      )
      .await;
  } else if RELATIVE_REQUEST.is_match(&request) || ABSOLUTE_REQUEST.is_match(&request) {
    // Direct resource mapping for absolute/relative requests
    if let Some(meta) = self
      .provides
      .iter()
      .find(|entry| entry.request == request && !entry.is_prefix)
      .cloned()
    {
      self
        .record_entry(
          meta.share_key,
          meta.share_scope,
          resource,
          &meta.provide,
          resource_data,
        )
        .await;
    }
  }

  Ok(())
}

impl Plugin for CollectShareEntryPlugin {
  fn name(&self) -> &'static str {
    "rspack.CollectShareEntryPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .normal_module_factory_hooks
      .module
      .tap(normal_module_factory_module::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}

fn infer_version(
  provide: &ProvideOptions,
  resource_data: &rspack_loader_runner::ResourceData,
) -> Option<String> {
  if let Some(version) = provide.version.as_ref() {
    if let ProvideVersion::Version(v) = version {
      return Some(v.clone());
    }
  }

  if let Some(description) = resource_data.description() {
    if let Some(obj) = description.json().as_object() {
      if let Some(version) = obj.get("version").and_then(|v| v.as_str()) {
        if !version.is_empty() {
          return Some(version.to_string());
        }
      }
    }
  }

  None
}
