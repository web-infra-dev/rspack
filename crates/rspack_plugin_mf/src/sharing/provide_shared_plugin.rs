use std::{fmt, sync::Arc};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ApplyContext, BoxModule, Compilation, CompilationParams, CompilerOptions, DependencyType,
  EntryOptions, NormalModuleCreateData, Plugin, PluginContext,
  PluginNormalModuleFactoryModuleHookOutput,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook, AsyncSeries, AsyncSeries2};
use rspack_loader_runner::ResourceData;
use rustc_hash::FxHashMap;
use tokio::sync::RwLock;

use super::{
  provide_shared_dependency::ProvideSharedDependency,
  provide_shared_module_factory::ProvideSharedModuleFactory,
};

static RELATIVE_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(\/|[A-Za-z]:\\|\\\\|\.\.?(\/|$))").expect("Invalid regex"));
static ABSOLUTE_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(\/|[A-Za-z]:\\|\\\\)").expect("Invalid regex"));

#[derive(Debug, Clone)]
pub struct ProvideOptions {
  pub share_key: String,
  pub share_scope: String,
  pub version: Option<ProvideVersion>,
  pub eager: bool,
}

#[derive(Debug, Clone)]
pub struct VersionedProvideOptions {
  pub share_key: String,
  pub share_scope: String,
  pub version: ProvideVersion,
  pub eager: bool,
}

impl ProvideOptions {
  fn to_versioned(&self) -> VersionedProvideOptions {
    VersionedProvideOptions {
      share_key: self.share_key.clone(),
      share_scope: self.share_scope.clone(),
      version: self.version.clone().unwrap_or_default(),
      eager: self.eager,
    }
  }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub enum ProvideVersion {
  Version(String),
  #[default]
  False,
}

impl fmt::Display for ProvideVersion {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ProvideVersion::Version(v) => write!(f, "{}", v),
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
    share_scope: &str,
    version: Option<&ProvideVersion>,
    eager: bool,
    resource: &str,
    resource_data: &ResourceData,
    mut add_diagnostic: impl FnMut(Diagnostic),
  ) {
    let title = "rspack.ProvideSharedPlugin";
    let error_header = "No version specified and unable to automatically determine one.";
    if let Some(version) = version {
      self.resolved_provide_map.write().await.insert(
        resource.to_string(),
        VersionedProvideOptions {
          share_key: share_key.to_string(),
          share_scope: share_scope.to_string(),
          version: version.to_owned(),
          eager,
        },
      );
    } else if let Some(description) = &resource_data.resource_description {
      if let Some(description) = description.json().as_object()
        && let Some(version) = description.get("version")
        && let Some(version) = version.as_str()
      {
        self.resolved_provide_map.write().await.insert(
          resource.to_string(),
          VersionedProvideOptions {
            share_key: share_key.to_string(),
            share_scope: share_scope.to_string(),
            version: ProvideVersion::Version(version.to_string()),
            eager,
          },
        );
      } else {
        add_diagnostic(Diagnostic::warn(title.to_string(), format!("{error_header} No version in description file (usually package.json). Add version to description file {}, or manually specify version in shared config. shared module {key} -> {resource}", description.path().display())));
      }
    } else {
      add_diagnostic(Diagnostic::warn(title.to_string(), format!("{error_header} No description file (usually package.json) found. Add description file with name and version, or manually specify version in shared config. shared module {key} -> {resource}")));
    }
  }
}

#[plugin_hook(AsyncSeries2<Compilation, CompilationParams> for ProvideSharedPlugin)]
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
    if RELATIVE_REQUEST.is_match(request) || ABSOLUTE_REQUEST.is_match(request) {
      resolved_provide_map.insert(request.to_string(), config.to_versioned());
    } else if request.ends_with('/') {
      prefix_match_provides.insert(request.to_string(), config.clone());
    } else {
      match_provides.insert(request.to_string(), config.clone());
    }
  }
  Ok(())
}

#[plugin_hook(AsyncSeries<Compilation> for ProvideSharedPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  for (resource, config) in self.resolved_provide_map.read().await.iter() {
    compilation
      .add_include(
        Box::new(ProvideSharedDependency::new(
          config.share_scope.to_string(),
          config.share_key.to_string(),
          config.version.clone(),
          resource.to_string(),
          config.eager,
        )),
        EntryOptions {
          name: None,
          ..Default::default()
        },
      )
      .await?;
  }
  self.resolved_provide_map.write().await.clear();
  self.match_provides.write().await.clear();
  self.prefix_match_provides.write().await.clear();
  Ok(())
}

#[async_trait]
impl Plugin for ProvideSharedPlugin {
  fn name(&self) -> &'static str {
    "rspack.ProvideSharedPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));
    Ok(())
  }

  async fn normal_module_factory_module(
    &self,
    _ctx: PluginContext,
    module: BoxModule,
    args: &mut NormalModuleCreateData<'_>,
  ) -> PluginNormalModuleFactoryModuleHookOutput {
    let resource = &args.resource_resolve_data.resource;
    let resource_data = &args.resource_resolve_data;
    if self
      .resolved_provide_map
      .read()
      .await
      .contains_key(resource)
    {
      return Ok(module);
    }
    let request = args.resolve_data_request;
    {
      let match_provides = self.match_provides.read().await;
      if let Some(config) = match_provides.get(request) {
        self
          .provide_shared_module(
            request,
            &config.share_key,
            &config.share_scope,
            config.version.as_ref(),
            config.eager,
            resource,
            resource_data,
            |d| args.diagnostics.push(d),
          )
          .await;
      }
    }
    for (prefix, config) in self.prefix_match_provides.read().await.iter() {
      if request.starts_with(prefix) {
        let remainder = &request[prefix.len()..];
        self
          .provide_shared_module(
            request,
            &(config.share_key.to_string() + remainder),
            &config.share_scope,
            config.version.as_ref(),
            config.eager,
            resource,
            resource_data,
            |d| args.diagnostics.push(d),
          )
          .await;
      }
    }
    Ok(module)
  }
}
