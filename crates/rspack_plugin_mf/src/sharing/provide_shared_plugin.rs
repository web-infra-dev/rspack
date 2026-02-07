use std::{
  fmt,
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
  provide_shared_dependency::ProvideSharedDependency,
  provide_shared_module_factory::ProvideSharedModuleFactory,
};
use crate::ConsumeVersion;

static RELATIVE_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(\/|[A-Za-z]:\\|\\\\|\.\.?(\/|$))").expect("Invalid regex"));
static ABSOLUTE_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(\/|[A-Za-z]:\\|\\\\)").expect("Invalid regex"));

#[derive(Debug, Clone)]
pub struct ProvideOptions {
  pub request: Option<String>,
  pub layer: Option<String>,
  pub share_key: String,
  pub share_scope: Vec<String>,
  pub version: Option<ProvideVersion>,
  pub eager: bool,
  pub singleton: Option<bool>,
  pub required_version: Option<ConsumeVersion>,
  pub strict_version: Option<bool>,
  pub tree_shaking_mode: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VersionedProvideOptions {
  pub request: Option<String>,
  pub layer: Option<String>,
  pub share_key: String,
  pub share_scope: Vec<String>,
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

fn create_lookup_key_for_sharing(request: &str, layer: Option<&str>) -> String {
  if let Some(layer) = layer {
    return format!("({layer}){request}");
  }
  request.to_string()
}

fn strip_lookup_layer_prefix(lookup: &str) -> &str {
  if lookup.starts_with('(') && let Some(index) = lookup.find(')') {
    return &lookup[index + 1..];
  }
  lookup
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

#[plugin]
#[derive(Debug)]
pub struct ProvideSharedPlugin {
  provides: Vec<(String, ProvideOptions)>,
  resolved_provide_map: RwLock<FxHashMap<String, VersionedProvideOptions>>,
  match_provides: RwLock<FxHashMap<String, ProvideOptions>>,
  prefix_match_provides: RwLock<FxHashMap<String, ProvideOptions>>,
}

impl ProvideSharedPlugin {
  pub fn new(provides: Vec<(String, ProvideOptions)>) -> Self {
    Self::new_inner(
      provides,
      Default::default(),
      Default::default(),
      Default::default(),
    )
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn provide_shared_module(
    &self,
    key: &str,
    share_key: &str,
    share_scope: &[String],
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
    let lookup_key = create_lookup_key_for_sharing(resource, layer.as_deref());
    if let Some(version) = version {
      self.resolved_provide_map.write().await.insert(
        lookup_key.clone(),
        VersionedProvideOptions {
          request: Some(key.to_string()),
          layer: layer.clone(),
          share_key: share_key.to_string(),
          share_scope: share_scope.to_vec(),
          version: version.to_owned(),
          eager,
          singleton,
          strict_version,
          required_version,
          tree_shaking_mode: tree_shaking_mode.clone(),
        },
      );
    } else if let Some(description) = resource_data.description() {
      if let Some(description) = description.json().as_object()
        && let Some(version) = description.get("version")
        && let Some(version) = version.as_str()
      {
        self.resolved_provide_map.write().await.insert(
          lookup_key.clone(),
          VersionedProvideOptions {
            request: Some(key.to_string()),
            layer: layer.clone(),
            share_key: share_key.to_string(),
            share_scope: share_scope.to_vec(),
            version: ProvideVersion::Version(version.to_string()),
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
  for (request, config) in &self.provides {
    let actual_request = config.request.as_deref().unwrap_or(request);
    let lookup_key = create_lookup_key_for_sharing(actual_request, config.layer.as_deref());
    if RELATIVE_REQUEST.is_match(actual_request) || ABSOLUTE_REQUEST.is_match(actual_request) {
      resolved_provide_map.insert(lookup_key, config.to_versioned());
    } else if actual_request.ends_with('/') {
      prefix_match_provides.insert(lookup_key, config.clone());
    } else {
      match_provides.insert(lookup_key, config.clone());
    }
  }
  Ok(())
}

#[plugin_hook(CompilerFinishMake for ProvideSharedPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let entries = self
    .resolved_provide_map
    .read()
    .await
    .iter()
    .map(|(lookup_key, config)| {
      let request = config
        .request
        .clone()
        .unwrap_or_else(|| strip_lookup_layer_prefix(lookup_key).to_string());
      (
        Box::new(ProvideSharedDependency::new(
          config.share_scope.clone(),
          config.share_key.clone(),
          config.version.clone(),
          request,
          config.eager,
          config.singleton,
          config.required_version.clone(),
          config.strict_version,
          config.layer.clone(),
          config.tree_shaking_mode.clone(),
        )) as BoxDependency,
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
  create_data: &mut NormalModuleCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  let resource = create_data.resource_resolve_data.resource();
  let resource_data = &create_data.resource_resolve_data;
  let effective_layer = module.get_layer().cloned().or_else(|| data.issuer_layer.clone());
  let resource_lookup = create_lookup_key_for_sharing(resource, effective_layer.as_deref());
  let fallback_resource_lookup = create_lookup_key_for_sharing(resource, None);
  if self
    .resolved_provide_map
    .read()
    .await
    .contains_key(&resource_lookup)
  {
    return Ok(());
  }
  if effective_layer.is_none()
    && self
      .resolved_provide_map
      .read()
      .await
      .contains_key(&fallback_resource_lookup)
  {
    return Ok(());
  }
  let request = &data.request;
  let request_lookup = create_lookup_key_for_sharing(request, effective_layer.as_deref());
  let fallback_request_lookup = create_lookup_key_for_sharing(request, None);
  {
    let match_provides = self.match_provides.read().await;
    if let Some(config) = match_provides
      .get(&request_lookup)
      .or_else(|| match_provides.get(&fallback_request_lookup))
    {
      self
        .provide_shared_module(
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
  for (prefix_lookup, config) in self.prefix_match_provides.read().await.iter() {
    if let Some(config_layer) = config.layer.as_deref()
      && effective_layer.as_deref() != Some(config_layer)
    {
      continue;
    }
    let prefix = config
      .request
      .as_deref()
      .unwrap_or_else(|| strip_lookup_layer_prefix(prefix_lookup));
    if request.starts_with(prefix) {
      let remainder = &request[prefix.len()..];
      self
        .provide_shared_module(
          request,
          &(config.share_key.clone() + remainder),
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
