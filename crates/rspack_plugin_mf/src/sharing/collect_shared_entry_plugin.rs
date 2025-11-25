use std::{
  collections::hash_map::Entry,
  path::{Path, PathBuf},
  sync::{Arc, OnceLock},
};

use regex::Regex;
use rspack_core::{
  BoxModule, Compilation, CompilationAsset, CompilationProcessAssets, CompilerThisCompilation,
  Context, DependenciesBlock, DependencyCategory, DependencyType, Module, ModuleFactoryCreateData,
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

const DEFAULT_FILENAME: &str = "collect-shared-entries.json";

#[derive(Debug, Clone, Default)]
struct CollectSharedEntryRecord {
  share_scope: String,
  requests: FxHashSet<CollectedShareRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CollectedShareRequest {
  request: String,
  version: String,
}

#[derive(Debug, Serialize)]
struct CollectSharedEntryAssetItem<'a> {
  #[serde(rename = "shareScope")]
  share_scope: &'a str,
  requests: &'a [[String; 2]],
}

#[derive(Debug)]
pub struct CollectSharedEntryPluginOptions {
  pub consumes: Vec<(String, Arc<ConsumeOptions>)>,
  pub filename: Option<String>,
}

#[plugin]
#[derive(Debug)]
pub struct CollectSharedEntryPlugin {
  options: CollectSharedEntryPluginOptions,
  resolver: OnceLock<Arc<Resolver>>,
  compiler_context: OnceLock<Context>,
  matched_consumes: OnceLock<Arc<MatchedConsumes>>,
  resolved_entries: RwLock<FxHashMap<String, CollectSharedEntryRecord>>,
}

impl CollectSharedEntryPlugin {
  pub fn new(options: CollectSharedEntryPluginOptions) -> Self {
    Self::new_inner(
      options,
      Default::default(),
      Default::default(),
      Default::default(),
      Default::default(),
    )
  }

  /// Infer package version from a module request path
  /// Example: ../../../.eden-mono/temp/node_modules/.pnpm/react-dom@18.3.1_react@18.3.1/node_modules/react-dom/index.js
  /// It locates react-dom's package.json and reads the version field
  async fn infer_version(&self, request: &str) -> Option<String> {
    // 1) Try pnpm store path pattern: .pnpm/<pkg>@<version>_
    let pnpm_re = Regex::new(r"/\\.pnpm/[^/]*@([^/_]+)").ok();
    if let Some(re) = pnpm_re {
      if let Some(caps) = re.captures(request) {
        if let Some(m) = caps.get(1) {
          return Some(m.as_str().to_string());
        }
      }
    }

    // 2) Fallback: walk to node_modules/<pkg>[/...] and read package.json
    let path = Path::new(request);
    let mut package_json_path = PathBuf::new();
    let mut found_node_modules = false;
    let mut need_two_segments = false;
    let mut captured = false;

    for component in path.components() {
      let comp_str = component.as_os_str().to_string_lossy();
      package_json_path.push(comp_str.as_ref());
      if !found_node_modules && comp_str == "node_modules" {
        found_node_modules = true;
        continue;
      }
      if found_node_modules && !captured {
        if comp_str.starts_with('@') {
          // scoped package: need scope + name
          need_two_segments = true;
          continue;
        } else {
          if need_two_segments {
            // this is the name after scope
            package_json_path.push("package.json");
            captured = true;
            break;
          } else {
            // unscoped package name is this segment
            package_json_path.push("package.json");
            captured = true;
            break;
          }
        }
      }
    }

    if captured && package_json_path.exists() {
      if let Ok(content) = std::fs::read_to_string(&package_json_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
          if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
            return Some(version.to_string());
          }
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

    // First try to infer version from the import_resolved path
    let version = if let Some(ref resolved_path) = import_resolved {
      if let Some(inferred) = self.infer_version(resolved_path).await {
        Some(ConsumeVersion::Version(inferred))
      } else {
        // If inference fails, return None and skip this entry
        None
      }
    } else {
      // If there is no resolved path, also return None
      None
    };

    // If no version info can be obtained, exit early
    let version = match version {
      Some(v) => v,
      None => return, // Early return from record_entry
    };

    let share_key = config.share_key.clone();
    let share_scope = config.share_scope.clone();
    let mut resolved_entries = self.resolved_entries.write().await;
    match resolved_entries.entry(share_key) {
      Entry::Occupied(mut entry) => {
        let record = entry.get_mut();
        record.share_scope = share_scope;
        record.requests.insert(CollectedShareRequest {
          request: import_resolved
            .clone()
            .unwrap_or_else(|| request.to_string()),
          version: version.to_string(),
        });
      }
      Entry::Vacant(entry) => {
        let mut requests = FxHashSet::default();
        requests.insert(CollectedShareRequest {
          request: import_resolved
            .clone()
            .unwrap_or_else(|| request.to_string()),
          version: version.to_string(),
        });
        entry.insert(CollectSharedEntryRecord {
          share_scope,
          requests,
        });
      }
    }
  }
}

#[plugin_hook(CompilerThisCompilation for CollectSharedEntryPlugin)]
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

#[plugin_hook(CompilationProcessAssets for CollectSharedEntryPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  // Traverse ConsumeSharedModule in the graph and collect real resolved module paths from fallback
  let module_graph = compilation.get_module_graph();
  let mut ordered_requests: FxHashMap<String, Vec<[String; 2]>> = FxHashMap::default();
  let mut share_scopes: FxHashMap<String, String> = FxHashMap::default();

  for (_id, module) in module_graph.modules().into_iter() {
    let module_type = module.module_type();
    if !matches!(module_type, rspack_core::ModuleType::ConsumeShared) {
      continue;
    }

    if let Some(consume) = module
      .as_any()
      .downcast_ref::<super::consume_shared_module::ConsumeSharedModule>()
    {
      // Parse share_scope and share_key from readable_identifier
      let ident = consume.readable_identifier(&Context::default()).to_string();
      // Format: "consume shared module ({scope}) {share_key}@..."
      let (scope, key) = {
        let mut scope = String::new();
        let mut key = String::new();
        if let Some(start) = ident.find("(")
          && let Some(end) = ident.find(")")
          && end > start
        {
          scope = ident[start + 1..end].to_string();
        }
        if let Some(pos) = ident.find(") ") {
          let rest = &ident[pos + 2..];
          let at = rest.find('@').unwrap_or(rest.len());
          key = rest[..at].to_string();
        }
        (scope, key)
      };
      if key.is_empty() {
        continue;
      }
      // Collect target modules from dependencies and async blocks
      let mut target_modules = Vec::new();
      for dep_id in consume.get_dependencies() {
        if let Some(target_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
          target_modules.push(*target_id);
        }
      }
      for block_id in consume.get_blocks() {
        if let Some(block) = module_graph.block_by_id(block_id) {
          for dep_id in block.get_dependencies() {
            if let Some(target_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
              target_modules.push(*target_id);
            }
          }
        }
      }

      // Add real module resource paths to the map and infer version
      let mut reqs = ordered_requests.remove(&key).unwrap_or_default();
      for target_id in target_modules {
        if let Some(target) = module_graph.module_by_identifier(&target_id) {
          if let Some(name) = target.name_for_condition() {
            let resource: String = name.into();
            let version = self
              .infer_version(&resource)
              .await
              .unwrap_or_else(|| "".to_string());
            let pair = [resource, version];
            if !reqs.iter().any(|p| p[0] == pair[0] && p[1] == pair[1]) {
              reqs.push(pair);
            }
          }
        }
      }
      reqs.sort_by(|a, b| a[0].cmp(&b[0]).then(a[1].cmp(&b[1])));
      ordered_requests.insert(key.clone(), reqs);
      if !scope.is_empty() {
        share_scopes.insert(key.clone(), scope);
      }
    }
  }

  // Build asset content
  let mut shared: FxHashMap<&str, CollectSharedEntryAssetItem<'_>> = FxHashMap::default();
  for (share_key, requests) in ordered_requests.iter() {
    let scope = share_scopes
      .get(share_key)
      .map(|s| s.as_str())
      .unwrap_or("");
    shared.insert(
      share_key.as_str(),
      CollectSharedEntryAssetItem {
        share_scope: scope,
        requests: requests.as_slice(),
      },
    );
  }

  let json = serde_json::to_string_pretty(&shared)
    .expect("CollectSharedEntryPlugin: failed to serialize share entries");

  // Get filename, or use default when absent
  let filename = self
    .options
    .filename
    .as_ref()
    .map(|f| f.clone())
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

#[plugin_hook(NormalModuleFactoryFactorize for CollectSharedEntryPlugin)]
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

  // Reuse the matching logic from consume_shared_plugin
  let consumes = self.get_matched_consumes();

  // 1. Exact match - use `unresolved`
  if let Some(matched) = consumes.unresolved.get(request) {
    self
      .record_entry(&data.context, request, matched.clone(), |d| {
        data.diagnostics.push(d)
      })
      .await;
    return Ok(None);
  }

  // 2. Prefix match - use `prefixed`
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

impl Plugin for CollectSharedEntryPlugin {
  fn name(&self) -> &'static str {
    "rspack.CollectSharedEntryPlugin"
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
