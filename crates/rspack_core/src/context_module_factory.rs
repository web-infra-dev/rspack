use std::{borrow::Cow, sync::Arc};

use async_recursion::async_recursion;
use cow_utils::CowUtils;
use derive_more::Debug;
use rspack_error::{Result, ToStringResultToRspackResultExt, error};
use rspack_fs::ReadableFileSystem;
use rspack_hook::define_hook;
use rspack_loader_runner::parse_resource;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_regex::RspackRegex;
use swc_core::common::util::take::Take;
use tracing::instrument;

use crate::{
  BoxDependency, CompilationId, ContextElementDependency, ContextModule, ContextModuleOptions,
  DependencyCategory, DependencyId, DependencyType, ModuleExt, ModuleFactory,
  ModuleFactoryCreateData, ModuleFactoryResult, ResolveArgs, ResolveContextModuleDependencies,
  ResolveInnerOptions, ResolveOptionsWithDependencyType, ResolveResult, Resolver, ResolverFactory,
  SharedPluginDriver, resolve,
};

#[derive(Debug)]
pub enum BeforeResolveResult {
  Ignored,
  Data(Box<BeforeResolveData>),
}

#[derive(Debug, Clone)]
pub struct BeforeResolveData {
  // context_info
  // resolve_options
  pub context: String,
  pub request: String,
  // assertions
  pub dependencies: Vec<BoxDependency>,
  // dependency_type
  // file_dependencies
  // missing_dependencies
  // context_dependencies
  // create_data
  // cacheable
  pub recursive: bool,
  pub reg_exp: Option<RspackRegex>,
}

#[derive(Clone)]
pub enum AfterResolveResult {
  Ignored,
  Data(Box<AfterResolveData>),
}

#[derive(Debug, Clone)]
pub struct AfterResolveData {
  pub compilation_id: CompilationId,
  pub resource: Utf8PathBuf,
  pub context: String,
  pub dependencies: Vec<BoxDependency>,
  // layer
  // resolve_options
  // file_dependencies: HashSet<String>,
  // missing_dependencies: HashSet<String>,
  // context_dependencies: HashSet<String>,
  pub request: String,
  // mode
  pub recursive: bool,
  pub reg_exp: Option<RspackRegex>,
  // namespace_object
  // addon: String,
  // chunk_name: String,
  // include
  // exclude
  // group_options
  // type_prefix: String,
  // category: String,
  // referenced_exports
  #[debug(skip)]
  pub resolve_dependencies: ResolveContextModuleDependencies,
}

define_hook!(ContextModuleFactoryBeforeResolve: SeriesWaterfall(data: BeforeResolveResult) -> BeforeResolveResult);
define_hook!(ContextModuleFactoryAfterResolve: SeriesWaterfall(data: AfterResolveResult) -> AfterResolveResult);

#[derive(Debug, Default)]
pub struct ContextModuleFactoryHooks {
  pub before_resolve: ContextModuleFactoryBeforeResolveHook,
  pub after_resolve: ContextModuleFactoryAfterResolveHook,
}

#[derive(Debug)]
pub struct ContextModuleFactory {
  loader_resolver_factory: Arc<ResolverFactory>,
  plugin_driver: SharedPluginDriver,
  #[debug(skip)]
  resolve_dependencies: ResolveContextModuleDependencies,
}

#[async_trait::async_trait]
impl ModuleFactory for ContextModuleFactory {
  #[instrument("context_module_factory:create", skip_all)]
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    match self.before_resolve(data).await? {
      BeforeResolveResult::Ignored => return Ok(ModuleFactoryResult::default()),
      BeforeResolveResult::Data(before_resolve_result) => {
        let (factorize_result, context_module_options) =
          self.resolve(data, before_resolve_result).await?;
        if let Some(context_module_options) = context_module_options
          && let Some(factorize_result) = self.after_resolve(data, context_module_options).await?
        {
          return Ok(factorize_result);
        }

        Ok(factorize_result)
      }
    }
  }
}

impl ContextModuleFactory {
  pub fn new(
    resolver_factory: Arc<ResolverFactory>,
    loader_resolver_factory: Arc<ResolverFactory>,
    plugin_driver: SharedPluginDriver,
  ) -> Self {
    let resolve_dependencies: ResolveContextModuleDependencies = Arc::new(move |options| {
      let resolver_factory = resolver_factory.clone();
      Box::pin(async move {
        tracing::trace!("resolving context module path {}", options.resource);
        if options.resource.as_str().is_empty() {
          return Ok(vec![]);
        }

        let resolver = &resolver_factory.get(ResolveOptionsWithDependencyType {
          resolve_options: options
            .resolve_options
            .clone()
            .map(|r| Box::new(Arc::unwrap_or_clone(r))),
          resolve_to_context: false,
          dependency_category: options.context_options.category,
        });
        let mut context_element_dependencies = vec![];
        visit_dirs(
          options.resource.as_str(),
          &options.resource,
          &mut context_element_dependencies,
          &options,
          &resolver.options(),
          resolver.inner_fs(),
        )
        .await?;
        context_element_dependencies.sort_by_cached_key(|d| d.user_request.clone());

        tracing::trace!(
          "resolving dependencies for {:?}",
          context_element_dependencies
        );

        Ok(context_element_dependencies)
      })
    });

    Self {
      loader_resolver_factory,
      plugin_driver,
      resolve_dependencies,
    }
  }

  async fn before_resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<BeforeResolveResult> {
    let dependency = data.dependencies[0]
      .as_context_dependency_mut()
      .expect("should be context dependency");
    let dependency_options = dependency.options();

    let before_resolve_data = BeforeResolveData {
      context: data.context.to_string(),
      request: dependency.request().to_string(),
      recursive: dependency_options.recursive,
      reg_exp: dependency_options.reg_exp.clone(),
      dependencies: data.dependencies.clone(),
    };

    match self
      .plugin_driver
      .context_module_factory_hooks
      .before_resolve
      .call(BeforeResolveResult::Data(Box::new(before_resolve_data)))
      .await?
    {
      BeforeResolveResult::Ignored => Ok(BeforeResolveResult::Ignored),
      BeforeResolveResult::Data(mut result) => {
        // The dependencies can be modified  in the before resolve hook
        data.dependencies = result.dependencies.take();
        Ok(BeforeResolveResult::Data(result))
      }
    }
  }

  fn get_loader_resolver(&self) -> Arc<Resolver> {
    self
      .loader_resolver_factory
      .get(ResolveOptionsWithDependencyType {
        resolve_options: None,
        resolve_to_context: false,
        dependency_category: DependencyCategory::CommonJS,
      })
  }

  async fn resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
    before_resolve_data: Box<BeforeResolveData>,
  ) -> Result<(ModuleFactoryResult, Option<ContextModuleOptions>)> {
    let plugin_driver = &self.plugin_driver;
    let dependency = data.dependencies[0]
      .as_context_dependency()
      .expect("should be context dependency");
    let mut file_dependencies = Default::default();
    let mut missing_dependencies = Default::default();

    let request = before_resolve_data.request;
    let (loader_request, specifier) = match request.rfind('!') {
      Some(idx) => {
        let mut loaders_prefix = String::new();
        let mut i = 0;

        let loaders_request = Cow::Borrowed(&request[..idx + 1]);
        while i < loaders_request.len() && loaders_request.chars().nth(i) == Some('!') {
          loaders_prefix.push('!');
          i += 1;
        }
        let loaders_request = loaders_request.as_ref()[i..]
          .trim_end_matches('!')
          .cow_replace("!!", "!");

        let loaders = if loaders_request.is_empty() {
          vec![]
        } else {
          loaders_request.split('!').collect()
        };
        let resource = request[idx + 1..].to_string();

        let mut loader_result = Vec::with_capacity(loaders.len());
        let loader_resolver = self.get_loader_resolver();
        for loader_request in loaders {
          let resolve_result = loader_resolver
            .resolve(data.context.as_ref(), loader_request)
            .await
            .to_rspack_result_with_message(|e| {
              format!(
                "Failed to resolve loader: {loader_request} in {} {e}",
                data.context
              )
            })?;
          match resolve_result {
            ResolveResult::Resource(resource) => {
              let resource = resource.full_path();
              loader_result.push(resource);
            }
            ResolveResult::Ignored => {
              let context = data.context.to_string();
              return Err(error!(
                "Failed to resolve loader: loader_request={loader_request}, context={context}"
              ));
            }
          }
        }
        let request = format!(
          "{}{}{}",
          loaders_prefix,
          loader_result.join("!"),
          if loader_result.is_empty() { "" } else { "!" }
        );
        (request, resource)
      }
      None => (String::new(), request),
    };

    let resolve_args = ResolveArgs {
      context: before_resolve_data.context.into(),
      importer: data.issuer_identifier.as_ref(),
      issuer: data.issuer.as_deref(),
      specifier: specifier.as_str(),
      dependency_type: dependency.dependency_type(),
      dependency_category: dependency.category(),
      span: dependency.range(),
      resolve_options: data.resolve_options.clone(),
      resolve_to_context: true,
      optional: dependency.get_optional(),
      file_dependencies: &mut file_dependencies,
      missing_dependencies: &mut missing_dependencies,
    };

    let resource_data = resolve(resolve_args, plugin_driver).await;

    let (module, context_module_options) = match resource_data {
      Ok(ResolveResult::Resource(resource)) => {
        let mut dependency_options = dependency.options().clone();
        dependency_options.recursive = before_resolve_data.recursive;
        dependency_options
          .reg_exp
          .clone_from(&before_resolve_data.reg_exp);

        let options = ContextModuleOptions {
          addon: loader_request.clone(),
          resource: resource.path,
          resource_query: resource.query,
          resource_fragment: resource.fragment,
          layer: data.issuer_layer.clone(),
          resolve_options: data.resolve_options.clone(),
          context_options: dependency_options,
          type_prefix: dependency.type_prefix(),
        };
        let module = ContextModule::new(self.resolve_dependencies.clone(), options.clone()).boxed();
        (module, Some(options))
      }
      Ok(ResolveResult::Ignored) => {
        // should create an empty context module when ignored
        let mut dependency_options = dependency.options().clone();
        dependency_options.recursive = before_resolve_data.recursive;
        dependency_options
          .reg_exp
          .clone_from(&before_resolve_data.reg_exp);

        let options = ContextModuleOptions {
          addon: loader_request.clone(),
          resource: Default::default(),
          resource_query: Default::default(),
          resource_fragment: Default::default(),
          layer: data.issuer_layer.clone(),
          resolve_options: data.resolve_options.clone(),
          context_options: dependency_options,
          type_prefix: dependency.type_prefix(),
        };
        let module = ContextModule::new(self.resolve_dependencies.clone(), options.clone()).boxed();
        (module, Some(options))
      }
      Err(err) => {
        data.add_file_dependencies(file_dependencies);
        data.add_missing_dependencies(missing_dependencies);
        return Err(err);
      }
    };

    data.add_file_dependencies(file_dependencies);
    data.add_missing_dependencies(missing_dependencies);
    // data.add_context_dependencies(context_dependencies);

    let module_factory_result = ModuleFactoryResult {
      module: Some(module),
    };
    Ok((module_factory_result, context_module_options))
  }

  async fn after_resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
    mut context_module_options: ContextModuleOptions,
  ) -> Result<Option<ModuleFactoryResult>> {
    let context_options = &context_module_options.context_options;
    let after_resolve_data = AfterResolveData {
      compilation_id: data.compilation_id,
      resource: context_module_options.resource.clone(),
      context: context_options.context.clone(),
      dependencies: data.dependencies.clone(),
      request: context_options.request.clone(),
      reg_exp: context_options.reg_exp.clone(),
      recursive: context_options.recursive,
      resolve_dependencies: self.resolve_dependencies.clone(),
    };

    match self
      .plugin_driver
      .context_module_factory_hooks
      .after_resolve
      .call(AfterResolveResult::Data(Box::new(after_resolve_data)))
      .await?
    {
      AfterResolveResult::Ignored => Ok(Some(ModuleFactoryResult::default())),
      AfterResolveResult::Data(mut after_resolve_data) => {
        // The dependencies can be modified  in the after resolve hook
        data.dependencies = after_resolve_data.dependencies.take();

        let parsed_resource = parse_resource(after_resolve_data.resource.as_str());
        if let Some(parsed_resource) = parsed_resource {
          if let Some(query) = &parsed_resource.query {
            context_module_options.resource_query.clone_from(query);
          }
          if let Some(fragment) = &parsed_resource.fragment {
            context_module_options
              .resource_fragment
              .clone_from(fragment);
          }
        }

        context_module_options.resource = after_resolve_data.resource;
        context_module_options.context_options.context = after_resolve_data.context;
        context_module_options.context_options.reg_exp = after_resolve_data.reg_exp;
        context_module_options.context_options.recursive = after_resolve_data.recursive;

        let module = ContextModule::new(
          after_resolve_data.resolve_dependencies,
          context_module_options.clone(),
        )
        .boxed();

        Ok(Some(ModuleFactoryResult::new_with_module(module)))
      }
    }
  }
}

#[async_recursion]
async fn visit_dirs(
  ctx: &str,
  dir: &Utf8Path,
  dependencies: &mut Vec<ContextElementDependency>,
  options: &ContextModuleOptions,
  resolve_options: &ResolveInnerOptions<'_>,
  fs: Arc<dyn ReadableFileSystem>,
) -> Result<()> {
  if !fs
    .metadata(dir)
    .await
    .map(|m| m.is_directory)
    .unwrap_or(false)
  {
    return Ok(());
  }
  let include = &options.context_options.include;
  let exclude = &options.context_options.exclude;
  for filename in fs.read_dir(dir).await? {
    let path = dir.join(&filename);
    let path_str = path.as_str();

    if let Some(exclude) = exclude
      && exclude.test(path_str)
    {
      // ignore excluded files
      continue;
    }

    if fs
      .metadata(&path)
      .await
      .map(|m| m.is_directory)
      .unwrap_or(false)
    {
      if options.context_options.recursive {
        visit_dirs(
          ctx,
          &path,
          dependencies,
          options,
          resolve_options,
          fs.clone(),
        )
        .await?;
      }
    } else if filename.starts_with('.') {
      // ignore hidden files
    } else {
      if let Some(include) = include
        && !include.test(path_str)
      {
        // ignore not included files
        continue;
      }

      // FIXME: nodejs resolver return path of context, sometimes is '/a/b', sometimes is '/a/b/'
      let relative_path = {
        let path_str = path_str.to_owned().drain(ctx.len()..).collect::<String>();
        let p = path_str.cow_replace('\\', "/");
        if p.as_ref().starts_with('/') {
          format!(".{p}")
        } else {
          format!("./{p}")
        }
      };

      let requests = alternative_requests(
        resolve_options,
        vec![AlternativeRequest::new(ctx.to_string(), relative_path)],
      );

      let Some(reg_exp) = &options.context_options.reg_exp else {
        return Ok(());
      };

      requests.iter().for_each(|r| {
        if !reg_exp.test(&r.request) {
          return;
        }
        let request = format!(
          "{}{}{}{}",
          options.addon,
          r.request,
          options.resource_query.clone(),
          options.resource_fragment.clone(),
        );
        let resource_identifier = ContextElementDependency::create_resource_identifier(
          options.resource.as_str(),
          &request,
          options.context_options.attributes.as_ref(),
        );

        dependencies.push(ContextElementDependency {
          id: DependencyId::new(),
          request,
          user_request: r.request.clone(),
          category: options.context_options.category,
          context: options.resource.clone().into(),
          layer: options.layer.clone(),
          options: options.context_options.clone(),
          resource_identifier,
          attributes: options.context_options.attributes.clone(),
          referenced_exports: options.context_options.referenced_exports.clone(),
          dependency_type: DependencyType::ContextElement(options.type_prefix),
          factorize_info: Default::default(),
        });
      })
    }
  }
  Ok(())
}

#[derive(Debug, Clone)]
pub struct AlternativeRequest {
  pub context: String,
  pub request: String,
}

impl AlternativeRequest {
  pub fn new(context: String, request: String) -> Self {
    Self { context, request }
  }
}

fn alternative_requests(
  resolve_options: &ResolveInnerOptions,
  mut items: Vec<AlternativeRequest>,
) -> Vec<AlternativeRequest> {
  // TODO: should respect fullySpecified resolve options
  for item in std::mem::take(&mut items) {
    if !resolve_options.is_enforce_extension_enabled() {
      items.push(item.clone());
    }
    for ext in resolve_options.extensions() {
      if item.request.ends_with(ext) {
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item.request[..(item.request.len() - ext.len())].to_string(),
        ));
      }
    }
  }

  for item in std::mem::take(&mut items) {
    items.push(item.clone());
    for main_file in resolve_options.main_files() {
      if item.request.ends_with(&format!("/{main_file}")) {
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item.request[..(item.request.len() - main_file.len())].to_string(),
        ));
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item.request[..(item.request.len() - main_file.len() - 1)].to_string(),
        ));
      }
    }
  }

  for item in std::mem::take(&mut items) {
    items.push(item.clone());
    for module in resolve_options.modules() {
      let dir = module.cow_replace('\\', "/");
      if item.request.starts_with(&format!("./{dir}/")) {
        items.push(AlternativeRequest::new(
          item.context.clone(),
          item.request[dir.len() + 3..].to_string(),
        ));
      }
    }
  }

  items
}
