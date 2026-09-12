use std::{
  fmt,
  sync::{Arc, LazyLock, OnceLock},
};

use regex::Regex;
use rspack_cacheable::cacheable;
use rspack_core::{
  BoxModule, ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilationParams, CompilerThisCompilation, Context, DependencyCategory, DependencyType,
  ModuleExt, ModuleFactoryCreateData, NormalModuleCreateData, NormalModuleFactoryCreateModule,
  NormalModuleFactoryFactorize, Plugin, ResolveOptionsWithDependencyType, ResolveResult, Resolver,
  RuntimeGlobals, RuntimeModule, runtime_mode::RuntimeMode,
};
use rspack_error::{Diagnostic, Result, error};
use rspack_hash::{RspackHash, RspackHasher};
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap;

use super::{
  RequestMatchKey, consume_shared_module::ConsumeSharedModule,
  consume_shared_runtime_module::ConsumeSharedRuntimeModule, find_exact_match, find_prefix_match,
  get_description_file,
};
use crate::ShareScope;

#[cacheable]
#[derive(Debug, Clone, rspack_hash::RspackHash)]
pub struct ConsumeOptions {
  pub request: Option<String>,
  pub issuer_layer: Option<String>,
  pub layer: Option<String>,
  pub import: Option<String>,
  pub import_resolved: Option<String>,
  pub share_key: String,
  pub share_scope: ShareScope,
  pub required_version: Option<ConsumeVersion>,
  pub package_name: Option<String>,
  pub strict_version: bool,
  pub singleton: bool,
  pub eager: bool,
  pub tree_shaking_mode: Option<String>,
}

#[cacheable]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConsumeVersion {
  Version(String),
  False,
}

impl RspackHash for ConsumeVersion {
  fn hash(&self, state: &mut RspackHasher) {
    match self {
      ConsumeVersion::Version(version) => version.hash(state),
      ConsumeVersion::False => "false".hash(state),
    }
  }
}

impl fmt::Display for ConsumeVersion {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ConsumeVersion::Version(v) => write!(f, "{v}"),
      ConsumeVersion::False => write!(f, "*"),
    }
  }
}

pub static RELATIVE_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^\.\.?(\/|$)").expect("Invalid regex"));
pub static ABSOLUTE_REQUEST: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^(\/|[A-Za-z]:\\|\\\\)").expect("Invalid regex"));
pub static PACKAGE_NAME: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"^((?:@[^\\/]+[\\/])?[^\\/]+)").expect("Invalid regex"));

#[derive(Debug)]
pub struct MatchedConsumes {
  pub resolved: FxHashMap<RequestMatchKey, Arc<ConsumeOptions>>,
  pub unresolved: FxHashMap<RequestMatchKey, Arc<ConsumeOptions>>,
  pub prefixed: Vec<(RequestMatchKey, Arc<ConsumeOptions>)>,
}

pub async fn resolve_matched_configs(
  compilation: &mut Compilation,
  resolver: Arc<Resolver>,
  configs: &[(String, Arc<ConsumeOptions>)],
  enhanced: bool,
) -> MatchedConsumes {
  let mut resolved = FxHashMap::default();
  let mut unresolved = FxHashMap::default();
  let mut prefixed = Vec::new();
  for (request, config) in configs {
    let request = config.request.as_deref().unwrap_or(request);
    let lookup_key = RequestMatchKey::new(request, config.issuer_layer.as_deref());
    if RELATIVE_REQUEST.is_match(request) && enhanced {
      unresolved.insert(lookup_key, config.clone());
    } else if RELATIVE_REQUEST.is_match(request) {
      let Ok(ResolveResult::Resource(resource)) = resolver
        .resolve(compilation.options.context.as_ref(), request)
        .await
      else {
        compilation.push_diagnostic(error!("Can't resolve shared module {request}").into());
        continue;
      };
      let resource_key =
        RequestMatchKey::new(resource.path.as_str(), config.issuer_layer.as_deref());
      resolved.insert(resource_key, config.clone());
      compilation
        .file_dependencies
        .insert(resource.path.as_path().into());
    } else if ABSOLUTE_REQUEST.is_match(request) {
      resolved.insert(lookup_key, config.clone());
    } else if request.ends_with('/') {
      prefixed.push((lookup_key, config.clone()));
    } else {
      unresolved.insert(lookup_key, config.clone());
    }
  }
  MatchedConsumes {
    resolved,
    unresolved,
    prefixed,
  }
}

pub fn get_required_version_from_description_file(
  data: &serde_json::Value,
  package_name: &str,
) -> Option<ConsumeVersion> {
  let data = data.as_object()?;
  let get_version_from_dependencies = |dependencies: &str| {
    data
      .get(dependencies)
      .and_then(|d| d.as_object())
      .and_then(|deps| deps.get(package_name))
      .and_then(|v| v.as_str())
      .map(|v| ConsumeVersion::Version(v.to_string()))
  };
  get_version_from_dependencies("optionalDependencies")
    .or_else(|| get_version_from_dependencies("dependencies"))
    .or_else(|| get_version_from_dependencies("peerDependencies"))
    .or_else(|| get_version_from_dependencies("devDependencies"))
}

#[derive(Debug)]
pub struct ConsumeSharedPluginOptions {
  pub consumes: Vec<(String, Arc<ConsumeOptions>)>,
  pub enhanced: bool,
}

#[plugin]
#[derive(Debug)]
pub struct ConsumeSharedPlugin {
  options: ConsumeSharedPluginOptions,
  resolver: OnceLock<Arc<Resolver>>,
  compiler_context: OnceLock<Context>,
  matched_consumes: OnceLock<Arc<MatchedConsumes>>,
}

impl ConsumeSharedPlugin {
  pub fn new(options: ConsumeSharedPluginOptions) -> Self {
    Self::new_inner(
      options,
      Default::default(),
      Default::default(),
      Default::default(),
    )
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
    let config = resolve_matched_configs(
      compilation,
      resolver,
      &self.options.consumes,
      self.options.enhanced,
    )
    .await;
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

  async fn get_required_version(
    &self,
    context: &Context,
    request: &str,
    config: Arc<ConsumeOptions>,
    mut add_diagnostic: impl FnMut(Diagnostic),
  ) -> Option<ConsumeVersion> {
    let mut required_version_warning = |details: &str| {
      add_diagnostic(Diagnostic::warn(
        self.name().into(),
        format!(
          "No required version specified and unable to automatically determine one. {details} file: shared module {request}"
        ),
      ))
    };
    if let Some(version) = config.required_version.as_ref() {
      Some(version.clone())
    } else {
      let package_name = if let Some(name) = &config.package_name {
        Some(name.as_str())
      } else if RELATIVE_REQUEST.is_match(request) || ABSOLUTE_REQUEST.is_match(request) {
        return None;
      } else if let Some(caps) = PACKAGE_NAME.captures(request)
        && let Some(mat) = caps.get(0)
      {
        Some(mat.as_str())
      } else {
        required_version_warning("Unable to extract the package name from request.");
        return None;
      };

      if let Some(package_name) = package_name {
        let fs = self.get_resolver().inner_fs();
        let (data, checked_description_file_paths) = get_description_file(
          fs,
          context.as_path(),
          Some(|data: &serde_json::Value| {
            let name_matches = data.get("name").and_then(|n| n.as_str()) == Some(package_name);
            let version_matches = get_required_version_from_description_file(data, package_name)
              .is_some_and(|version| matches!(version, ConsumeVersion::Version(_)));
            name_matches || version_matches
          }),
        )
        .await;

        if let Some(data) = data {
          if let Some(name) = data.get("name").and_then(|n| n.as_str())
            && name == package_name
          {
            // Package self-referencing
            return None;
          }
          return get_required_version_from_description_file(&data, package_name);
        } else {
          if let Some(file_paths) = checked_description_file_paths
            && !file_paths.is_empty()
          {
            required_version_warning(&format!(
              "Unable to find required version for \"{package_name}\" in description file/s\n{}\nIt need to be in dependencies, devDependencies or peerDependencies.",
              file_paths.join("\n")
            ));
          } else {
            required_version_warning(&format!(
              "Unable to find description file in {}",
              context.as_str()
            ));
          }
          return None;
        }
      }

      None
    }
  }

  async fn create_consume_shared_module(
    &self,
    context: &Context,
    request: &str,
    config: Arc<ConsumeOptions>,
    runtime_mode: RuntimeMode,
    mut add_diagnostic: impl FnMut(Diagnostic),
  ) -> ConsumeSharedModule {
    let direct_fallback = matches!(
      &config.import,
      Some(i) if ABSOLUTE_REQUEST.is_match(i)
        || (!self.options.enhanced && RELATIVE_REQUEST.is_match(i))
    );
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
    let required_version = self
      .get_required_version(context, request, config.clone(), add_diagnostic)
      .await;
    ConsumeSharedModule::new(
      if direct_fallback {
        self.get_context()
      } else {
        context.clone()
      },
      ConsumeOptions {
        request: config.request.clone(),
        issuer_layer: config.issuer_layer.clone(),
        layer: config.layer.clone(),
        import: import_resolved
          .is_some()
          .then(|| config.import.clone())
          .and_then(|i| i),
        import_resolved,
        share_key: config.share_key.clone(),
        share_scope: config.share_scope.clone(),
        required_version,
        package_name: config.package_name.clone(),
        strict_version: config.strict_version,
        singleton: config.singleton,
        eager: config.eager,
        tree_shaking_mode: config.tree_shaking_mode.clone(),
      },
      runtime_mode,
    )
  }
}

#[plugin_hook(CompilerThisCompilation for ConsumeSharedPlugin)]
async fn this_compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::ConsumeSharedFallback,
    params.normal_module_factory.clone(),
  );
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

#[plugin_hook(NormalModuleFactoryFactorize for ConsumeSharedPlugin)]
async fn factorize(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<BoxModule>> {
  let dep = data.dependencies[0]
    .as_module_dependency()
    .expect("should be module dependency");
  if matches!(
    dep.dependency_type(),
    DependencyType::ConsumeSharedFallback
      | DependencyType::ProvideModuleForShared
      | DependencyType::ShareContainerFallback
  ) {
    return Ok(None);
  }
  let request = &data.request;
  let consumes = self.get_matched_consumes();
  if let Some(matched) =
    find_exact_match(&consumes.unresolved, request, data.issuer_layer.as_deref())
  {
    let module = self
      .create_consume_shared_module(
        &data.context,
        request,
        matched.clone(),
        data.options.experiments.runtime_mode,
        |d| data.diagnostics.push(d),
      )
      .await;
    return Ok(Some(module.boxed()));
  }
  if let Some((options, remainder)) =
    find_prefix_match(&consumes.prefixed, request, data.issuer_layer.as_deref())
  {
    let module = self
      .create_consume_shared_module(
        &data.context,
        request,
        Arc::new(ConsumeOptions {
          request: Some(request.to_owned()),
          issuer_layer: options.issuer_layer.clone(),
          layer: options.layer.clone(),
          import: options.import.as_ref().map(|i| i.to_owned() + remainder),
          import_resolved: options.import_resolved.clone(),
          share_key: options.share_key.clone() + remainder,
          share_scope: options.share_scope.clone(),
          required_version: options.required_version.clone(),
          package_name: options.package_name.clone(),
          strict_version: options.strict_version,
          singleton: options.singleton,
          eager: options.eager,
          tree_shaking_mode: options.tree_shaking_mode.clone(),
        }),
        data.options.experiments.runtime_mode,
        |d| data.diagnostics.push(d),
      )
      .await;
    return Ok(Some(module.boxed()));
  }
  Ok(None)
}

#[plugin_hook(NormalModuleFactoryCreateModule for ConsumeSharedPlugin)]
async fn create_module(
  &self,
  data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
) -> Result<Option<BoxModule>> {
  if matches!(
    data.dependencies[0].dependency_type(),
    DependencyType::ConsumeSharedFallback
      | DependencyType::ProvideModuleForShared
      | DependencyType::ShareContainerFallback
  ) {
    return Ok(None);
  }
  let resource = create_data.resource_resolve_data.resource();
  let consumes = self.get_matched_consumes();
  if let Some(options) =
    find_exact_match(&consumes.resolved, resource, data.issuer_layer.as_deref())
  {
    let module = self
      .create_consume_shared_module(
        &data.context,
        resource,
        options.clone(),
        data.options.experiments.runtime_mode,
        |d| data.diagnostics.push(d),
      )
      .await;
    return Ok(Some(module.boxed()));
  }
  Ok(None)
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ConsumeSharedPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  _runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  runtime_modules.push(Box::new(ConsumeSharedRuntimeModule::new(
    &compilation.runtime_template,
    self.options.enhanced,
  )));
  Ok(())
}

impl Plugin for ConsumeSharedPlugin {
  fn name(&self) -> &'static str {
    "rspack.ConsumeSharedPlugin"
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
      .normal_module_factory_hooks
      .create_module
      .tap(create_module::new(self));
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    Ok(())
  }
}
