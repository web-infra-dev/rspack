use std::{
  fmt,
  path::Path,
  sync::{Arc, LazyLock},
};

use regex::Regex;
use rspack_core::{
  BoxDependency, BoxModule, Compilation, CompilationParams, CompilerCompilation,
  CompilerFinishMake, DependencyType, EntryOptions, ModuleFactoryCreateData,
  NormalModuleCreateData, NormalModuleFactoryModule, Plugin,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_loader_runner::ResourceData;
use rustc_hash::FxHashMap;
use tokio::sync::RwLock;

use super::{
  RequestMatchKey, find_ancestor_description_data, find_exact_match, find_prefix_match,
  provide_shared_dependency::ProvideSharedDependency,
  provide_shared_module_factory::ProvideSharedModuleFactory,
};
use crate::{ConsumeVersion, ShareScope};

static RELATIVE_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(\/|[A-Za-z]:\\|\\\\|\.\.?(\/|$))").expect("Invalid regex"));
static ABSOLUTE_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(\/|[A-Za-z]:\\|\\\\)").expect("Invalid regex"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvideOptions {
  #[doc(hidden)]
  pub config_id: usize,
  pub request: Option<String>,
  pub layer: Option<String>,
  pub share_key: String,
  pub share_scope: ShareScope,
  pub version: Option<ProvideVersion>,
  pub eager: bool,
  pub singleton: Option<bool>,
  pub required_version: Option<ConsumeVersion>,
  pub strict_version: Option<bool>,
  pub tree_shaking_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedProvideOptions {
  config_id: usize,
  pub request: Option<String>,
  pub layer: Option<String>,
  pub share_key: String,
  pub share_scope: ShareScope,
  pub version: ProvideVersion,
  pub eager: bool,
  pub singleton: Option<bool>,
  pub required_version: Option<ConsumeVersion>,
  pub strict_version: Option<bool>,
  pub tree_shaking_mode: Option<String>,
}

impl ProvideOptions {
  fn to_versioned(&self) -> VersionedProvideOptions {
    VersionedProvideOptions {
      config_id: self.config_id,
      request: self.request.clone(),
      layer: self.layer.clone(),
      share_key: self.share_key.clone(),
      share_scope: self.share_scope.clone(),
      version: self.version.clone().unwrap_or_default(),
      eager: self.eager,
      singleton: self.singleton,
      required_version: self.required_version.clone(),
      strict_version: self.strict_version,
      tree_shaking_mode: self.tree_shaking_mode.clone(),
    }
  }
}

#[rspack_cacheable::cacheable]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ProvideVersion {
  Version(String),
  #[default]
  False,
}

impl fmt::Display for ProvideVersion {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ProvideVersion::Version(v) => write!(f, "{v}"),
      ProvideVersion::False => write!(f, "0"),
    }
  }
}

fn insert_unique_config<T: PartialEq>(
  configs: &mut FxHashMap<RequestMatchKey, Vec<T>>,
  key: RequestMatchKey,
  config: T,
) {
  let entries = configs.entry(key).or_default();
  if !entries.contains(&config) {
    entries.push(config);
  }
}

fn insert_resolved_config(
  configs: &mut FxHashMap<RequestMatchKey, Vec<VersionedProvideOptions>>,
  key: RequestMatchKey,
  config: VersionedProvideOptions,
) {
  let entries = configs.entry(key).or_default();
  if let Some(existing) = entries
    .iter_mut()
    .find(|existing| existing.config_id == config.config_id)
  {
    *existing = config;
  } else {
    entries.push(config);
  }
}

fn insert_unique_prefix_config<T: PartialEq>(
  configs: &mut Vec<(RequestMatchKey, Vec<T>)>,
  key: RequestMatchKey,
  config: T,
) {
  if let Some((_, entries)) = configs.iter_mut().find(|(existing, _)| existing == &key) {
    if !entries.contains(&config) {
      entries.push(config);
    }
  } else {
    configs.push((key, vec![config]));
  }
}

fn provide_dependencies(
  configs: &FxHashMap<RequestMatchKey, Vec<VersionedProvideOptions>>,
) -> Vec<ProvideSharedDependency> {
  configs
    .iter()
    .flat_map(|(lookup_key, configs)| {
      configs.iter().map(move |config| {
        ProvideSharedDependency::new(
          config.share_scope.clone(),
          config.share_key.clone(),
          config.version.clone(),
          config
            .request
            .clone()
            .unwrap_or_else(|| lookup_key.request().to_string()),
          config.eager,
          config.singleton,
          config.required_version.clone(),
          config.strict_version,
          config.layer.clone(),
          config.tree_shaking_mode.clone(),
        )
      })
    })
    .collect()
}

#[plugin]
#[derive(Debug)]
pub struct ProvideSharedPlugin {
  provides: Vec<(String, ProvideOptions)>,
  resolved_provide_map: RwLock<FxHashMap<RequestMatchKey, Vec<VersionedProvideOptions>>>,
  match_provides: RwLock<FxHashMap<RequestMatchKey, Vec<ProvideOptions>>>,
  prefix_match_provides: RwLock<Vec<(RequestMatchKey, Vec<ProvideOptions>)>>,
}

impl ProvideSharedPlugin {
  pub fn new(mut provides: Vec<(String, ProvideOptions)>) -> Self {
    for (config_id, (_, config)) in provides.iter_mut().enumerate() {
      config.config_id = config_id;
    }
    Self::new_inner(
      provides,
      Default::default(),
      Default::default(),
      Default::default(),
    )
  }

  /// For secondary entry points (e.g. `@mui/material/styles`) whose own
  /// `package.json` has no `version`, walk up to the parent package and use
  /// its version — but only when the shared key matches
  /// `<parent_name>/<relative_path>`.
  fn find_parent_package_version(description_path: &Path, share_key: &str) -> Option<String> {
    let entry_dir = if description_path
      .file_name()
      .is_some_and(|name| name == "package.json")
    {
      description_path.parent()?
    } else {
      description_path
    };

    find_ancestor_description_data(entry_dir, |dir, parent| {
      let parent_name = parent.get("name").and_then(|n| n.as_str())?;
      let parent_version = parent.get("version").and_then(|v| v.as_str())?;
      let rel = entry_dir.strip_prefix(dir).ok()?;
      let rel_posix: String = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
      let expected_key = format!("{parent_name}/{rel_posix}");
      (share_key == expected_key).then(|| parent_version.to_string())
    })
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn provide_shared_module(
    &self,
    config_id: usize,
    key: &str,
    share_key: &str,
    share_scope: &ShareScope,
    version: Option<&ProvideVersion>,
    eager: bool,
    singleton: Option<bool>,
    required_version: Option<ConsumeVersion>,
    strict_version: Option<bool>,
    tree_shaking_mode: Option<String>,
    layer: Option<String>,
    resource: &str,
    resource_data: &ResourceData,
    mut add_diagnostic: impl FnMut(Diagnostic),
  ) {
    let title = "rspack.ProvideSharedPlugin";
    let error_header = "No version specified and unable to automatically determine one.";
    let lookup_key = RequestMatchKey::new(resource, layer.as_deref());
    if let Some(version) = version {
      let mut resolved_provide_map = self.resolved_provide_map.write().await;
      insert_resolved_config(
        &mut resolved_provide_map,
        lookup_key.clone(),
        VersionedProvideOptions {
          config_id,
          request: Some(resource.to_string()),
          layer: layer.clone(),
          share_key: share_key.to_string(),
          share_scope: share_scope.clone(),
          version: version.to_owned(),
          eager,
          singleton,
          strict_version,
          required_version,
          tree_shaking_mode: tree_shaking_mode.clone(),
        },
      );
    } else if let Some(description) = resource_data.description() {
      let version = description
        .json()
        .as_object()
        .and_then(|d| d.get("version"))
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .or_else(|| Self::find_parent_package_version(description.path(), share_key));

      if let Some(version) = version {
        let mut resolved_provide_map = self.resolved_provide_map.write().await;
        insert_resolved_config(
          &mut resolved_provide_map,
          lookup_key.clone(),
          VersionedProvideOptions {
            config_id,
            request: Some(resource.to_string()),
            layer: layer.clone(),
            share_key: share_key.to_string(),
            share_scope: share_scope.clone(),
            version: ProvideVersion::Version(version),
            eager,
            singleton,
            strict_version,
            required_version,
            tree_shaking_mode: tree_shaking_mode.clone(),
          },
        );
      } else {
        add_diagnostic(Diagnostic::warn(
          title.to_string(),
          format!(
            "{error_header} No version in description file (usually package.json). Add version to description file {}, or manually specify version in shared config. shared module {key} -> {resource}",
            description.path().display()
          ),
        ));
      }
    } else {
      add_diagnostic(Diagnostic::warn(
        title.to_string(),
        format!(
          "{error_header} No description file (usually package.json) found. Add description file with name and version, or manually specify version in shared config. shared module {key} -> {resource}"
        ),
      ));
    }
  }
}

#[plugin_hook(CompilerCompilation for ProvideSharedPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::ProvideModuleForShared,
    params.normal_module_factory.clone(),
  );
  compilation.set_dependency_factory(
    DependencyType::ProvideSharedModule,
    Arc::new(ProvideSharedModuleFactory::default()),
  );

  let mut resolved_provide_map = self.resolved_provide_map.write().await;
  let mut match_provides = self.match_provides.write().await;
  let mut prefix_match_provides = self.prefix_match_provides.write().await;
  match_provides.clear();
  prefix_match_provides.clear();
  for (request, config) in &self.provides {
    let actual_request = config.request.as_deref().unwrap_or(request);
    let lookup_key = RequestMatchKey::new(actual_request, config.layer.as_deref());
    if RELATIVE_REQUEST.is_match(actual_request) || ABSOLUTE_REQUEST.is_match(actual_request) {
      insert_resolved_config(&mut resolved_provide_map, lookup_key, config.to_versioned());
    } else if actual_request.ends_with('/') {
      insert_unique_prefix_config(&mut prefix_match_provides, lookup_key, config.clone());
    } else {
      insert_unique_config(&mut match_provides, lookup_key, config.clone());
    }
  }
  Ok(())
}

#[plugin_hook(CompilerFinishMake for ProvideSharedPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let resolved_provide_map = self.resolved_provide_map.read().await;
  let entries = provide_dependencies(&resolved_provide_map)
    .into_iter()
    .map(|dependency| {
      (
        Box::new(dependency) as BoxDependency,
        EntryOptions {
          name: None,
          ..Default::default()
        },
      )
    })
    .collect::<Vec<_>>();
  compilation.add_include(entries).await?;
  Ok(())
}

#[plugin_hook(NormalModuleFactoryModule for ProvideSharedPlugin)]
async fn normal_module_factory_module(
  &self,
  data: &mut ModuleFactoryCreateData,
  create_data: &NormalModuleCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  let resource = create_data.resource_resolve_data.resource();
  let resource_data = create_data.resource_resolve_data.as_ref();
  let effective_layer = module
    .get_layer()
    .cloned()
    .or_else(|| data.issuer_layer.clone());
  let request = &data.request;
  let matched = {
    let match_provides = self.match_provides.read().await;
    find_exact_match(&match_provides, request, effective_layer.as_deref()).cloned()
  };
  if let Some(configs) = matched {
    for config in configs {
      self
        .provide_shared_module(
          config.config_id,
          request,
          &config.share_key,
          &config.share_scope,
          config.version.as_ref(),
          config.eager,
          config.singleton,
          config.required_version.clone(),
          config.strict_version,
          config.tree_shaking_mode.clone(),
          config.layer.clone(),
          resource,
          resource_data,
          |d| data.diagnostics.push(d),
        )
        .await;
    }
  }
  let prefix_match = {
    let prefix_match_provides = self.prefix_match_provides.read().await;
    find_prefix_match(&prefix_match_provides, request, effective_layer.as_deref())
      .map(|(config, remainder)| (config.clone(), remainder.to_string()))
  };
  if let Some((configs, remainder)) = prefix_match {
    for config in configs {
      self
        .provide_shared_module(
          config.config_id,
          request,
          &(config.share_key.clone() + remainder.as_str()),
          &config.share_scope,
          config.version.as_ref(),
          config.eager,
          config.singleton,
          config.required_version.clone(),
          config.strict_version,
          config.tree_shaking_mode.clone(),
          config.layer.clone(),
          resource,
          resource_data,
          |d| data.diagnostics.push(d),
        )
        .await;
    }
  }
  Ok(())
}

impl Plugin for ProvideSharedPlugin {
  fn name(&self) -> &'static str {
    "rspack.ProvideSharedPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));
    ctx
      .normal_module_factory_hooks
      .module
      .tap(normal_module_factory_module::new(self));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use rustc_hash::FxHashMap;

  use super::{
    ProvideOptions, ProvideVersion, insert_resolved_config, insert_unique_config,
    insert_unique_prefix_config,
  };
  use crate::{
    ShareScope,
    sharing::{RequestMatchKey, find_exact_match, find_prefix_match},
  };

  fn provide_options(share_key: &str, share_scope: &str, layer: Option<&str>) -> ProvideOptions {
    ProvideOptions {
      config_id: 0,
      request: Some("pkg".to_string()),
      layer: layer.map(str::to_string),
      share_key: share_key.to_string(),
      share_scope: ShareScope::Single(share_scope.to_string()),
      version: Some(ProvideVersion::Version("1.0.0".to_string())),
      eager: false,
      singleton: None,
      required_version: None,
      strict_version: None,
      tree_shaking_mode: None,
    }
  }

  #[test]
  fn same_request_and_layer_keep_every_provider_identity() {
    let key = RequestMatchKey::new("pkg", Some("server"));
    let mut matches = FxHashMap::default();
    insert_unique_config(
      &mut matches,
      key.clone(),
      provide_options("pkg-a", "scope-a", Some("server")),
    );
    insert_unique_config(
      &mut matches,
      key,
      provide_options("pkg-b", "scope-b", Some("server")),
    );

    let matched = find_exact_match(&matches, "pkg", Some("server")).expect("match");
    assert_eq!(matched.len(), 2);

    let resolved = matched
      .iter()
      .cloned()
      .enumerate()
      .map(|(config_id, mut options)| {
        options.config_id = config_id;
        options.to_versioned()
      })
      .fold(FxHashMap::default(), |mut resolved, options| {
        insert_resolved_config(
          &mut resolved,
          RequestMatchKey::new("/resolved/pkg.js", Some("server")),
          options,
        );
        resolved
      });
    let resolved = resolved
      .get(&RequestMatchKey::new("/resolved/pkg.js", Some("server")))
      .expect("resolved providers");
    assert_eq!(resolved.len(), 2);
    assert!(resolved.iter().any(|options| {
      options.share_key == "pkg-a"
        && options.share_scope == ShareScope::Single("scope-a".to_string())
    }));
    assert!(resolved.iter().any(|options| {
      options.share_key == "pkg-b"
        && options.share_scope == ShareScope::Single("scope-b".to_string())
    }));
  }

  #[test]
  fn resolved_provider_version_is_replaced_without_dropping_other_providers() {
    let key = RequestMatchKey::new("/resolved/pkg.js", Some("server"));
    let mut resolved = FxHashMap::default();
    let first = provide_options("pkg-a", "scope-a", Some("server")).to_versioned();
    let mut other_options = provide_options("pkg-b", "scope-b", Some("server"));
    other_options.config_id = 1;
    let other = other_options.to_versioned();
    insert_resolved_config(&mut resolved, key.clone(), first.clone());
    insert_resolved_config(&mut resolved, key.clone(), other.clone());

    let mut updated = first;
    updated.version = ProvideVersion::Version("2.0.0".to_string());
    insert_resolved_config(&mut resolved, key.clone(), updated);

    let providers = resolved.get(&key).expect("resolved providers");
    assert_eq!(providers.len(), 2);
    assert!(providers.iter().any(|provider| {
      provider.share_key == "pkg-a"
        && provider.version == ProvideVersion::Version("2.0.0".to_string())
    }));
    assert!(providers.iter().any(|provider| provider == &other));
  }

  #[test]
  fn configured_versions_of_the_same_shared_identity_coexist() {
    let key = RequestMatchKey::new("/resolved/pkg.js", None);
    let mut resolved = FxHashMap::default();
    let first = provide_options("pkg", "default", None).to_versioned();
    let mut second_options = provide_options("pkg", "default", None);
    second_options.config_id = 1;
    second_options.version = Some(ProvideVersion::Version("2.0.0".to_string()));
    let second = second_options.to_versioned();

    insert_resolved_config(&mut resolved, key.clone(), first);
    insert_resolved_config(&mut resolved, key.clone(), second);

    let providers = resolved.get(&key).expect("resolved providers");
    assert_eq!(providers.len(), 2);
    assert!(
      providers
        .iter()
        .any(|provider| provider.version == ProvideVersion::Version("1.0.0".to_string()))
    );
    assert!(
      providers
        .iter()
        .any(|provider| provider.version == ProvideVersion::Version("2.0.0".to_string()))
    );
  }

  #[test]
  fn exact_layer_precedes_fallback_and_prefix_order_stays_deterministic() {
    let mut matches = FxHashMap::default();
    insert_unique_config(
      &mut matches,
      RequestMatchKey::new("pkg", None),
      provide_options("fallback", "default", None),
    );
    insert_unique_config(
      &mut matches,
      RequestMatchKey::new("pkg", Some("server")),
      provide_options("exact", "server", Some("server")),
    );
    assert_eq!(
      find_exact_match(&matches, "pkg", Some("server")).expect("exact match")[0].share_key,
      "exact"
    );
    assert_eq!(
      find_exact_match(&matches, "pkg", Some("client")).expect("fallback match")[0].share_key,
      "fallback"
    );

    let mut prefixes = Vec::new();
    insert_unique_prefix_config(
      &mut prefixes,
      RequestMatchKey::new("pkg/", Some("server")),
      provide_options("short", "server", Some("server")),
    );
    insert_unique_prefix_config(
      &mut prefixes,
      RequestMatchKey::new("pkg/feature/", None),
      provide_options("long", "default", None),
    );
    let (matched, remainder) =
      find_prefix_match(&prefixes, "pkg/feature/button", Some("server")).expect("prefix match");
    assert_eq!(matched[0].share_key, "long");
    assert_eq!(remainder, "button");
  }
}
