use std::{
  collections::hash_map::Entry,
  path::{Path, PathBuf},
  sync::{Arc, OnceLock},
};

use cow_utils::CowUtils;
use rspack_core::{
  BoxModule, Compilation, CompilationAsset, CompilationProcessAssets, CompilerThisCompilation,
  Context, DependencyCategory, DependencyType, ModuleFactoryCreateData,
  NormalModuleFactoryFactorize, Plugin, ResolveOptionsWithDependencyType, ResolveResult, Resolver,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;
use tokio::sync::RwLock;

use super::consume_shared_plugin::{
  ABSOLUTE_REQUEST, ConsumeOptions, ConsumeVersion, MatchedConsumes, RELATIVE_REQUEST,
  resolve_matched_configs,
};

const DEFAULT_FILENAME: &str = "collect-share-entries.json";

#[derive(Debug, Clone, Default)]
struct CollectShareEntryRecord {
  share_scope: String,
  requests: FxHashSet<CollectedShareRequest>,
}

fn normalize_request_path(path: &str) -> String {
  path.cow_replace('\\', "/").into_owned()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CollectedShareRequest {
  request: String,
  version: String,
}

#[derive(Debug, Serialize)]
struct CollectShareEntryAssetItem<'a> {
  #[serde(rename = "shareScope")]
  share_scope: &'a str,
  requests: &'a [[String; 2]],
}

#[derive(Debug)]
pub struct CollectShareEntryPluginOptions {
  pub consumes: Vec<(String, Arc<ConsumeOptions>)>,
  pub filename: Option<String>,
}

#[plugin]
#[derive(Debug)]
pub struct CollectShareEntryPlugin {
  options: CollectShareEntryPluginOptions,
  resolver: OnceLock<Arc<Resolver>>,
  compiler_context: OnceLock<Context>,
  matched_consumes: OnceLock<Arc<MatchedConsumes>>,
  resolved_entries: RwLock<FxHashMap<String, CollectShareEntryRecord>>,
}

impl CollectShareEntryPlugin {
  pub fn new(options: CollectShareEntryPluginOptions) -> Self {
    // let consumes: Vec<CollectShareEntryMeta> = options
    //   .consumes
    //   .into_iter()
    //   .map(|(request, consume)| {
    //     let consume = consume.clone();
    //     let share_key = consume.share_key.clone();
    //     let share_scope = consume.share_scope.clone();
    //     let is_prefix = request.ends_with('/');
    //     CollectShareEntryMeta {
    //       request,
    //       share_key,
    //       share_scope,
    //       is_prefix,
    //       consume,
    //     }
    //   })
    //   .collect();

    Self::new_inner(
      options,
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
    )
  }

  /// 根据模块请求路径推断版本信息
  /// 例如：../../../.eden-mono/temp/node_modules/.pnpm/react-dom@18.3.1_react@18.3.1/node_modules/react-dom/index.js
  /// 会找到 react-dom 的 package.json 并读取 version 字段
  async fn infer_version(&self, request: &str) -> Option<String> {
    // 将请求路径转换为 Path
    let path = Path::new(request);

    // 查找包含 node_modules 的路径段
    let mut node_modules_found = false;
    let mut package_path = None;

    for component in path.components() {
      let comp_str = component.as_os_str().to_string_lossy();
      if comp_str == "node_modules" {
        node_modules_found = true;
        continue;
      }

      if node_modules_found {
        // 下一个组件应该是包名
        package_path = Some(comp_str.to_string());
        break;
      }
    }

    if let Some(package_name) = package_path {
      // 构建 package.json 的完整路径
      let mut package_json_path = PathBuf::new();
      let mut found_node_modules = false;

      for component in path.components() {
        let comp_str = component.as_os_str().to_string_lossy();
        package_json_path.push(comp_str.as_ref());

        if comp_str == "node_modules" {
          found_node_modules = true;
          // 添加包名目录
          package_json_path.push(&package_name);
          // 添加 package.json
          package_json_path.push("package.json");
          break;
        }
      }

      if found_node_modules && package_json_path.exists() {
        // 尝试读取 package.json
        if let Ok(content) = std::fs::read_to_string(&package_json_path)
          && let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
          // 读取 version 字段
          && let Some(version) = json.get("version").and_then(|v| v.as_str())
        {
          return Some(version.to_string());
        }
      }
    }

    None
  }

  fn init_context(&self, compilation: &Compilation) {
    self
      .compiler_context
      .set(compilation.options.context.clone())
      .expect("failed to set compiler context");
  }

  fn get_context(&self) -> Context {
    self
      .compiler_context
      .get()
      .expect("init_context first")
      .clone()
  }

  fn init_resolver(&self, compilation: &Compilation) {
    self
      .resolver
      .set(
        compilation
          .resolver_factory
          .get(ResolveOptionsWithDependencyType {
            resolve_options: None,
            resolve_to_context: false,
            dependency_category: DependencyCategory::Esm,
          }),
      )
      .expect("failed to set resolver for multiple times");
  }

  fn get_resolver(&self) -> Arc<Resolver> {
    self.resolver.get().expect("init_resolver first").clone()
  }

  async fn init_matched_consumes(&self, compilation: &mut Compilation, resolver: Arc<Resolver>) {
    let config = resolve_matched_configs(compilation, resolver, &self.options.consumes).await;
    self
      .matched_consumes
      .set(Arc::new(config))
      .expect("failed to set matched consumes");
  }

  fn get_matched_consumes(&self) -> Arc<MatchedConsumes> {
    self
      .matched_consumes
      .get()
      .expect("init_matched_consumes first")
      .clone()
  }

  async fn record_entry(
    &self,
    context: &Context,
    request: &str,
    config: Arc<ConsumeOptions>,
    mut add_diagnostic: impl FnMut(Diagnostic),
  ) {
    let direct_fallback = matches!(&config.import, Some(i) if RELATIVE_REQUEST.is_match(i) | ABSOLUTE_REQUEST.is_match(i));
    let import_resolved = match &config.import {
      None => None,
      Some(import) => {
        let resolver = self.get_resolver();
        resolver
          .resolve(
            if direct_fallback {
              self.get_context()
            } else {
              context.clone()
            }
            .as_ref(),
            import,
          )
          .await
          .map_err(|_e| {
            add_diagnostic(Diagnostic::error(
              "ModuleNotFoundError".into(),
              format!("resolving fallback for shared module {request}"),
            ))
          })
          .ok()
      }
    }
    .and_then(|i| match i {
      ResolveResult::Resource(r) => Some(r.path.as_str().to_string()),
      ResolveResult::Ignored => None,
    });

    // Prefer inferring version from resolved path; fall back to required_version
    let inferred_version = if let Some(resolved_path) = import_resolved.as_ref() {
      self.infer_version(resolved_path).await
    } else {
      None
    };

    let version = inferred_version.or_else(|| {
      config.required_version.as_ref().and_then(|v| match v {
        ConsumeVersion::Version(v) => Some(v.clone()),
        ConsumeVersion::False => None,
      })
    });

    // 如果无法获取版本信息，直接结束方法
    let version = match version {
      Some(v) => v,
      None => return, // 直接结束 record_entry 方法
    };

    let share_key = config.share_key.clone();
    let share_scope = config.share_scope.clone();
    let mut resolved_entries = self.resolved_entries.write().await;
    match resolved_entries.entry(share_key) {
      Entry::Occupied(mut entry) => {
        let record = entry.get_mut();
        record.share_scope = share_scope;
        record.requests.insert(CollectedShareRequest {
          request: normalize_request_path(
            &import_resolved
              .clone()
              .unwrap_or_else(|| request.to_string()),
          ),
          version: version.to_string(),
        });
      }
      Entry::Vacant(entry) => {
        let mut requests = FxHashSet::default();
        requests.insert(CollectedShareRequest {
          request: normalize_request_path(
            &import_resolved
              .clone()
              .unwrap_or_else(|| request.to_string()),
          ),
          version: version.to_string(),
        });
        entry.insert(CollectShareEntryRecord {
          share_scope,
          requests,
        });
      }
    }
  }
}

#[plugin_hook(CompilerThisCompilation for CollectShareEntryPlugin)]
async fn this_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut rspack_core::CompilationParams,
) -> Result<()> {
  if self.compiler_context.get().is_none() {
    self.init_context(compilation);
  }
  if self.resolver.get().is_none() {
    self.init_resolver(compilation);
  }
  if self.matched_consumes.get().is_none() {
    self
      .init_matched_consumes(compilation, self.get_resolver())
      .await;
  }
  Ok(())
}

#[plugin_hook(CompilationProcessAssets for CollectShareEntryPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  // Ensure we have entries for unresolved consumes without cloning the full map.
  let consumes = self.get_matched_consumes();
  let resolver = self.get_resolver();
  let context = self.get_context();

  let existing_keys: FxHashSet<String> = {
    let entries = self.resolved_entries.read().await;
    entries.keys().cloned().collect()
  };

  let mut new_records: Vec<(String, CollectShareEntryRecord)> = Vec::new();
  for (request, config) in &consumes.unresolved {
    if existing_keys.contains(&config.share_key) {
      continue;
    }

    let import_resolved = match &config.import {
      None => None,
      Some(import) => resolver.resolve(context.as_ref(), import).await.ok(),
    }
    .and_then(|i| match i {
      ResolveResult::Resource(r) => Some(r.path.as_str().to_string()),
      ResolveResult::Ignored => None,
    });

    let inferred_version = if let Some(resolved_path) = import_resolved.as_ref() {
      self.infer_version(resolved_path).await
    } else {
      None
    };
    let version = inferred_version.or_else(|| {
      config.required_version.as_ref().and_then(|v| match v {
        ConsumeVersion::Version(v) => Some(v.clone()),
        ConsumeVersion::False => None,
      })
    });
    let Some(version) = version else { continue };

    let mut requests = FxHashSet::default();
    requests.insert(CollectedShareRequest {
      request: normalize_request_path(
        &import_resolved
          .clone()
          .unwrap_or_else(|| request.to_string()),
      ),
      version: version.to_string(),
    });
    new_records.push((
      config.share_key.clone(),
      CollectShareEntryRecord {
        share_scope: config.share_scope.clone(),
        requests,
      },
    ));
  }

  if !new_records.is_empty() {
    let mut entries = self.resolved_entries.write().await;
    for (key, record) in new_records {
      entries.entry(key).or_insert(record);
    }
  }

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

  let json = serde_json::to_string_pretty(&serde_json::json!({ "shared": shared }))
    .expect("CollectShareEntryPlugin: failed to serialize share entries");

  // 获取文件名，如果不存在则使用默认值
  let filename = self
    .options
    .filename
    .clone()
    .unwrap_or_else(|| DEFAULT_FILENAME.to_string());

  compilation.emit_asset(
    filename,
    CompilationAsset::new(
      Some(RawStringSource::from(json).boxed()),
      Default::default(),
    ),
  );
  Ok(())
}

#[plugin_hook(NormalModuleFactoryFactorize for CollectShareEntryPlugin)]
async fn factorize(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<BoxModule>> {
  let dep = data.dependencies[0]
    .as_module_dependency()
    .expect("should be module dependency");
  if matches!(
    dep.dependency_type(),
    DependencyType::ConsumeSharedFallback | DependencyType::ProvideModuleForShared
  ) {
    return Ok(None);
  }
  let request = dep.request();

  // 直接复用 consume_shared_plugin 的匹配逻辑
  let consumes = self.get_matched_consumes();

  // 1. 精确匹配 - 使用 unresolved
  if let Some(matched) = consumes.unresolved.get(request) {
    self
      .record_entry(&data.context, request, matched.clone(), |d| {
        data.diagnostics.push(d)
      })
      .await;
    return Ok(None);
  }

  // 2. 前缀匹配 - 使用 prefixed
  for (prefix, options) in &consumes.prefixed {
    if request.starts_with(prefix) {
      let remainder = &request[prefix.len()..];
      self
        .record_entry(
          &data.context,
          request,
          Arc::new(ConsumeOptions {
            import: options.import.as_ref().map(|i| i.to_owned() + remainder),
            import_resolved: options.import_resolved.clone(),
            share_key: options.share_key.clone() + remainder,
            share_scope: options.share_scope.clone(),
            required_version: options.required_version.clone(),
            package_name: options.package_name.clone(),
            strict_version: options.strict_version,
            singleton: options.singleton,
            eager: options.eager,
          }),
          |d| data.diagnostics.push(d),
        )
        .await;
      return Ok(None);
    }
  }

  Ok(None)
}

impl Plugin for CollectShareEntryPlugin {
  fn name(&self) -> &'static str {
    "rspack.CollectShareEntryPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compiler_hooks
      .this_compilation
      .tap(this_compilation::new(self));
    ctx
      .normal_module_factory_hooks
      .factorize
      .tap(factorize::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
