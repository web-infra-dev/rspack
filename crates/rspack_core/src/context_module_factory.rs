use std::sync::Arc;

use rspack_error::{error, Result};
use rspack_hook::define_hook;
use rspack_regex::RspackRegex;
use tracing::instrument;

use crate::{
  resolve, ContextModule, ContextModuleOptions, DependencyCategory, ModuleExt, ModuleFactory,
  ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier, RawModule, ResolveArgs,
  ResolveOptionsWithDependencyType, ResolveResult, Resolver, ResolverFactory, SharedPluginDriver,
};

#[derive(Clone)]
pub enum BeforeResolveResult {
  Ignored,
  Data(Box<BeforeResolveData>),
}

#[derive(Clone)]
pub struct BeforeResolveData {
  // context_info
  // resolve_options
  pub context: String,
  pub request: Option<String>,
  // assertions
  // dependencies
  // dependency_type
  // file_dependencies
  // missing_dependencies
  // context_dependencies
  // create_data
  // cacheable
}

#[derive(Clone)]
pub enum AfterResolveResult {
  Ignored,
  Data(Box<AfterResolveData>),
}

#[derive(Clone)]
pub struct AfterResolveData {
  pub resource: String,
  pub context: String,
  // dependencies
  // layer
  // resolve_options
  // file_dependencies: HashSet<String>,
  // missing_dependencies: HashSet<String>,
  // context_dependencies: HashSet<String>,
  pub request: String,
  // mode
  // recursive: bool,
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
}

define_hook!(ContextModuleFactoryBeforeResolve: AsyncSeriesWaterfall(data: BeforeResolveResult) -> BeforeResolveResult);
define_hook!(ContextModuleFactoryAfterResolve: AsyncSeriesWaterfall(data: AfterResolveResult) -> AfterResolveResult);

#[derive(Debug, Default)]
pub struct ContextModuleFactoryHooks {
  pub before_resolve: ContextModuleFactoryBeforeResolveHook,
  pub after_resolve: ContextModuleFactoryAfterResolveHook,
}

#[derive(Debug)]
pub struct ContextModuleFactory {
  loader_resolver_factory: Arc<ResolverFactory>,
  plugin_driver: SharedPluginDriver,
}

#[async_trait::async_trait]
impl ModuleFactory for ContextModuleFactory {
  #[instrument(name = "context_module_factory:create", skip_all)]
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    if let Some(before_resolve_result) = self.before_resolve(data).await? {
      return Ok(before_resolve_result);
    }

    let (factorize_result, mut context_module_options) = self.resolve(data).await?;

    if let Some(context_module_options) = context_module_options.as_mut() {
      if let Some(factorize_result) = self.after_resolve(context_module_options).await? {
        return Ok(factorize_result);
      }
    }

    Ok(factorize_result)
  }
}

impl ContextModuleFactory {
  pub fn new(
    loader_resolver_factory: Arc<ResolverFactory>,
    plugin_driver: SharedPluginDriver,
  ) -> Self {
    Self {
      loader_resolver_factory,
      plugin_driver,
    }
  }

  async fn before_resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<Option<ModuleFactoryResult>> {
    let before_resolve_data = BeforeResolveData {
      context: data.context.to_string(),
      request: data.request().map(|r| r.to_string()),
    };

    match self
      .plugin_driver
      .context_module_factory_hooks
      .before_resolve
      .call(BeforeResolveResult::Data(Box::new(before_resolve_data)))
      .await?
    {
      BeforeResolveResult::Ignored => Ok(Some(ModuleFactoryResult::default())),
      BeforeResolveResult::Data(d) => {
        data.context = d.context.into();
        Ok(None)
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
  ) -> Result<(ModuleFactoryResult, Option<ContextModuleOptions>)> {
    let plugin_driver = &self.plugin_driver;
    let dependency = data
      .dependency
      .as_context_dependency()
      .expect("should be context dependency");
    let mut file_dependencies = Default::default();
    let mut missing_dependencies = Default::default();
    // let context_dependencies = Default::default();
    let request = dependency.request();
    let (loader_request, specifier) = match request.rfind('!') {
      Some(idx) => {
        let mut loaders_prefix = String::new();
        let mut loaders_request = request[..idx + 1].to_string();
        let mut i = 0;
        while i < loaders_request.len() && loaders_request.chars().nth(i) == Some('!') {
          loaders_prefix.push('!');
          i += 1;
        }
        loaders_request = loaders_request[i..]
          .trim_end_matches('!')
          .replace("!!", "!");

        let loaders = if loaders_request.is_empty() {
          vec![]
        } else {
          loaders_request.split('!').collect()
        };
        let resource = &request[idx + 1..];

        let mut loader_result = Vec::with_capacity(loaders.len());
        let loader_resolver = self.get_loader_resolver();
        for loader_request in loaders {
          let resolve_result = loader_resolver
            .resolve(data.context.as_ref(), loader_request)
            .map_err(|err| {
              let context = data.context.to_string();
              error!("Failed to resolve loader: {loader_request} in {context} {err:?}")
            })?;
          match resolve_result {
            ResolveResult::Resource(resource) => {
              let resource = resource.full_path().to_string_lossy().to_string();
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
      None => ("".to_string(), request),
    };

    let resolve_args = ResolveArgs {
      context: data.context.clone(),
      importer: data.issuer_identifier.as_ref(),
      issuer: data.issuer.as_deref(),
      specifier,
      dependency_type: dependency.dependency_type(),
      dependency_category: dependency.category(),
      span: dependency.span(),
      resolve_options: data.resolve_options.clone(),
      resolve_to_context: true,
      optional: dependency.get_optional(),
      file_dependencies: &mut file_dependencies,
      missing_dependencies: &mut missing_dependencies,
    };

    let resource_data = resolve(resolve_args, plugin_driver).await;

    let (module, context_module_options) = match resource_data {
      Ok(ResolveResult::Resource(resource)) => {
        let options = ContextModuleOptions {
          addon: loader_request.to_string(),
          resource: resource.path.to_string_lossy().to_string(),
          resource_query: resource.query,
          resource_fragment: resource.fragment,
          resolve_options: data.resolve_options.clone(),
          context_options: dependency.options().clone(),
        };
        let module = Box::new(ContextModule::new(
          options.clone(),
          plugin_driver.resolver_factory.clone(),
        ));
        (module, Some(options))
      }
      Ok(ResolveResult::Ignored) => {
        let ident = format!("{}/{}", data.context, specifier);
        let module_identifier = ModuleIdentifier::from(format!("ignored|{ident}"));
        let raw_module = RawModule::new(
          "/* (ignored) */".to_owned(),
          module_identifier,
          format!("{ident} (ignored)"),
          Default::default(),
        )
        .boxed();
        return Ok((ModuleFactoryResult::new_with_module(raw_module), None));
      }
      Err(err) => {
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
    context_module_options: &mut ContextModuleOptions,
  ) -> Result<Option<ModuleFactoryResult>> {
    let context_options = &context_module_options.context_options;
    let after_resolve_data = AfterResolveData {
      resource: context_module_options.resource.to_owned(),
      context: context_options.context.to_owned(),
      request: context_options.request.to_owned(),
      reg_exp: context_options.reg_exp.clone(),
    };

    match self
      .plugin_driver
      .context_module_factory_hooks
      .after_resolve
      .call(AfterResolveResult::Data(Box::new(after_resolve_data)))
      .await?
    {
      AfterResolveResult::Ignored => Ok(Some(ModuleFactoryResult::default())),
      AfterResolveResult::Data(d) => {
        context_module_options.resource = d.resource;
        context_module_options.context_options.reg_exp = d.reg_exp;

        let module = ContextModule::new(
          context_module_options.clone(),
          self.loader_resolver_factory.clone(),
        );
        Ok(Some(ModuleFactoryResult::new_with_module(Box::new(module))))
      }
    }
  }
}
