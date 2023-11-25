use std::{fmt, path::Path, sync::Arc};

use async_trait::async_trait;
use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use rspack_error::internal_error;
use rustc_hash::FxHashMap;
use semver::VersionReq;

use super::{
  consume_shared_fallback_dependency::ConsumeSharedFallbackDependency,
  provide_for_shared_dependency::ProvideForSharedDependency,
};
use crate::{
  AdditionalChunkRuntimeRequirementsArgs, BoxModule, Compilation, Context, DependencyCategory,
  DependencyType, FactorizeArgs, NormalModuleCreateData, NormalModuleFactoryContext, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginFactorizeHookOutput,
  PluginNormalModuleFactoryCreateModuleHookOutput, PluginThisCompilationHookOutput,
  ResolveOptionsWithDependencyType, ResolveResult, Resolver, RuntimeGlobals, ThisCompilationArgs,
};

#[derive(Debug, Clone)]
pub struct ConsumeOptions {
  pub import: Option<String>,
  pub import_resolved: Option<String>,
  pub share_key: String,
  pub share_scope: String,
  pub required_version: Option<ConsumeVersion>,
  pub package_name: Option<String>,
  pub strict_version: bool,
  pub singleton: bool,
  pub eager: bool,
}

#[derive(Debug, Clone)]
pub enum ConsumeVersion {
  Version(VersionReq),
  False,
}

impl fmt::Display for ConsumeVersion {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ConsumeVersion::Version(v) => write!(f, "{}", v),
      ConsumeVersion::False => write!(f, "*"),
    }
  }
}

static RELATIVE_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\.\.?(\/|$)").expect("Invalid regex"));
static ABSOLUTE_REQUEST: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(\/|[A-Za-z]:\\|\\\\)").expect("Invalid regex"));
static PACKAGE_NAME: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^((?:@[^\\/]+[\\/])?[^\\/]+)").expect("Invalid regex"));

#[derive(Debug)]
struct MatchedConsumes {
  resolved: FxHashMap<String, Arc<ConsumeOptions>>,
  unresolved: FxHashMap<String, Arc<ConsumeOptions>>,
  prefixed: FxHashMap<String, Arc<ConsumeOptions>>,
}

fn resolve_matched_configs(
  compilation: &mut Compilation,
  resolver: &Resolver,
  configs: &[(String, Arc<ConsumeOptions>)],
) -> MatchedConsumes {
  let mut resolved = FxHashMap::default();
  let mut unresolved = FxHashMap::default();
  let mut prefixed = FxHashMap::default();
  for (request, config) in configs {
    if RELATIVE_REQUEST.is_match(&request) {
      let Ok(ResolveResult::Resource(resource)) =
        resolver.resolve(compilation.options.context.as_ref(), &request)
      else {
        compilation
          .push_batch_diagnostic(internal_error!("Can't resolve shared module {request}").into());
        continue;
      };
      resolved.insert(resource.path.to_string_lossy().into_owned(), config.clone());
      compilation.file_dependencies.insert(resource.path);
    } else if ABSOLUTE_REQUEST.is_match(&request) {
      resolved.insert(request.to_owned(), config.clone());
    } else if request.ends_with('/') {
      prefixed.insert(request.to_owned(), config.clone());
    } else {
      unresolved.insert(request.to_owned(), config.clone());
    }
  }
  MatchedConsumes {
    resolved,
    unresolved,
    prefixed,
  }
}

fn get_description_file(dir: &Path) -> serde_json::Value {
  todo!()
}

#[derive(Debug)]
pub struct ConsumeSharedPlugin {
  consumes: Vec<(String, Arc<ConsumeOptions>)>,
  resolver: OnceCell<Arc<Resolver>>,
  compiler_context: OnceCell<Context>,
  matched_consumes: OnceCell<MatchedConsumes>,
}

impl ConsumeSharedPlugin {
  fn init_context(&self, compilation: &Compilation) {
    self
      .compiler_context
      .set(compilation.options.context.clone());
  }

  fn get_context(&self) -> &Context {
    &self.compiler_context.get().expect("init_context first")
  }

  fn init_resolver(&self, compilation: &Compilation) {
    self.resolver.set(
      compilation
        .resolver_factory
        .get(ResolveOptionsWithDependencyType {
          resolve_options: None,
          resolve_to_context: false,
          dependency_type: DependencyType::Unknown,
          dependency_category: DependencyCategory::Esm,
        }),
    );
  }

  fn get_resolver(&self) -> &Resolver {
    &self.resolver.get().expect("init_resolver first")
  }

  fn init_matched_consumes(&self, compilation: &mut Compilation, resolver: &Resolver) {
    self.matched_consumes.set(resolve_matched_configs(
      compilation,
      resolver,
      &self.consumes,
    ));
  }

  fn get_matched_consumes(&self) -> &MatchedConsumes {
    &self
      .matched_consumes
      .get()
      .expect("init_matched_consumes first")
  }

  fn create_consume_shared_module(
    &self,
    context: &Context,
    request: &str,
    config: Arc<ConsumeOptions>,
  ) -> BoxModule {
    let direct_fallback = matches!(&config.import, Some(i) if RELATIVE_REQUEST.is_match(i) | ABSOLUTE_REQUEST.is_match(i));
    let import_resolved = config.import.as_ref().and_then(|import| {
      let resolver = self.get_resolver();
      resolver
        .resolve(
          if direct_fallback {
            self.get_context()
          } else {
            context
          }
          .as_ref(),
          &import,
        )
        .ok()
    });
    let required_version = config.required_version.as_ref().or_else(|| {
      let package_name = if let Some(name) = &config.package_name {
        name
      } else {
        if ABSOLUTE_REQUEST.is_match(request) {
          return None;
        }
        if let Some(caps) = PACKAGE_NAME.captures(request)
          && let Some(mat) = caps.get(0)
        {
          mat.as_str()
        } else {
          return None;
        }
      };
      todo!()
    });
    todo!()
  }
}

#[async_trait]
impl Plugin for ConsumeSharedPlugin {
  fn name(&self) -> &'static str {
    "rspack.ConsumeSharedPlugin"
  }

  async fn this_compilation(
    &self,
    args: ThisCompilationArgs<'_>,
  ) -> PluginThisCompilationHookOutput {
    self.init_context(&args.this_compilation);
    self.init_resolver(&args.this_compilation);
    self.init_matched_consumes(args.this_compilation, self.get_resolver());
    Ok(())
  }

  async fn factorize(
    &self,
    _ctx: PluginContext,
    args: FactorizeArgs<'_>,
    _job_ctx: &mut NormalModuleFactoryContext,
  ) -> PluginFactorizeHookOutput {
    let dep = args.dependency;
    if matches!(
      dep.dependency_type(),
      DependencyType::ConsumeSharedFallback | DependencyType::ProvideModuleForShared
    ) {
      return Ok(None);
    }
    let request = dep.request();
    let consumes = self.get_matched_consumes();
    if let Some(matched) = consumes.unresolved.get(request) {}
    for (prefix, options) in &consumes.prefixed {
      if request.starts_with(prefix) {
        let remainder = &request[prefix.len()..];
      }
    }
    Ok(None)
  }

  async fn normal_module_factory_create_module(
    &self,
    _ctx: PluginContext,
    args: &NormalModuleCreateData,
  ) -> PluginNormalModuleFactoryCreateModuleHookOutput {
    if matches!(
      args.dependency_type,
      DependencyType::ConsumeSharedFallback | DependencyType::ProvideModuleForShared
    ) {
      return Ok(None);
    }
    let resource = &args.resource_resolve_data.resource;
    let consumes = self.get_matched_consumes();
    if let Some(options) = consumes.resolved.get(resource) {}
    Ok(None)
  }

  fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    args.runtime_requirements.insert(RuntimeGlobals::MODULE);
    args
      .runtime_requirements
      .insert(RuntimeGlobals::MODULE_CACHE);
    args
      .runtime_requirements
      .insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
    args
      .runtime_requirements
      .insert(RuntimeGlobals::SHARE_SCOPE_MAP);
    args
      .runtime_requirements
      .insert(RuntimeGlobals::INITIALIZE_SHARING);
    args
      .runtime_requirements
      .insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    args.compilation.add_runtime_module(args.chunk, todo!());
    Ok(())
  }
}
