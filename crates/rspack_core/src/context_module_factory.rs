use std::{borrow::Cow, sync::Arc};

use cow_utils::CowUtils;
use rspack_error::{error, Result};
use rspack_hook::define_hook;
use rspack_paths::Utf8PathBuf;
use rspack_regex::RspackRegex;
use tracing::instrument;

use crate::{
  resolve, BoxDependency, ContextModule, ContextModuleOptions, DependencyCategory, ErrorSpan,
  ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier,
  RawModule, ResolveArgs, ResolveOptionsWithDependencyType, ResolveResult, Resolver,
  ResolverFactory, SharedPluginDriver,
};

#[derive(Debug)]
pub enum BeforeResolveResult {
  Ignored,
  Data(Box<BeforeResolveData>),
}

#[derive(Debug)]
pub struct BeforeResolveData {
  // context_info
  // resolve_options
  pub context: String,
  pub request: String,
  // assertions
  // dependencies
  // dependency_type
  // file_dependencies
  // missing_dependencies
  // context_dependencies
  // create_data
  // cacheable
  pub recursive: bool,
  pub reg_exp: Option<RspackRegex>,
  // FIX: This field is used by ContextReplacementPlugin to ignore errors collected during the build phase of Context modules.
  // In Webpack, the ContextModuleFactory's beforeResolve hook directly traverses dependencies in context and modifies the ContextDependency's critical field.
  // Since Rspack currently has difficulty passing the dependencies field, an additional field is used to indicate whether to ignore the collected errors.
  pub critical: bool,
}

#[derive(Clone)]
pub enum AfterResolveResult {
  Ignored,
  Data(Box<AfterResolveData>),
}

#[derive(Debug, Clone)]
pub struct AfterResolveData {
  pub resource: Utf8PathBuf,
  pub context: String,
  // dependencies
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

  // FIX: This field is used by ContextReplacementPlugin to ignore errors collected during the build phase of Context modules.
  // In Webpack, the ContextModuleFactory's beforeResolve hook directly traverses dependencies in context and modifies the ContextDependency's critical field.
  // Since Rspack currently has difficulty passing the dependencies field, an additional field is used to indicate whether to ignore the collected errors.
  pub critical: bool,
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
  resolver_factory: Arc<ResolverFactory>,
  loader_resolver_factory: Arc<ResolverFactory>,
  plugin_driver: SharedPluginDriver,
}

#[async_trait::async_trait]
impl ModuleFactory for ContextModuleFactory {
  #[instrument(name = "context_module_factory:create", skip_all)]
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    match self.before_resolve(data).await? {
      BeforeResolveResult::Ignored => return Ok(ModuleFactoryResult::default()),
      BeforeResolveResult::Data(before_resolve_result) => {
        let (factorize_result, context_module_options) =
          self.resolve(data, before_resolve_result).await?;

        if let Some(context_module_options) = context_module_options {
          if let Some(factorize_result) = self
            .after_resolve(context_module_options, &mut data.dependencies)
            .await?
          {
            return Ok(factorize_result);
          }
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
    Self {
      resolver_factory,
      loader_resolver_factory,
      plugin_driver,
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
      critical: true,
    };

    match self
      .plugin_driver
      .context_module_factory_hooks
      .before_resolve
      .call(BeforeResolveResult::Data(Box::new(before_resolve_data)))
      .await?
    {
      BeforeResolveResult::Ignored => Ok(BeforeResolveResult::Ignored),
      BeforeResolveResult::Data(result) => {
        if !result.critical {
          *dependency.critical_mut() = None;
        }
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
            .map_err(|err| {
              let context = data.context.to_string();
              error!("Failed to resolve loader: {loader_request} in {context} {err:?}")
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
      span: dependency
        .range()
        .map(|range| ErrorSpan::new(range.start, range.end)),
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
        dependency_options.reg_exp = before_resolve_data.reg_exp.clone();

        let options = ContextModuleOptions {
          addon: loader_request.to_string(),
          resource: resource.path,
          resource_query: resource.query,
          resource_fragment: resource.fragment,
          layer: data.issuer_layer.clone(),
          resolve_options: data.resolve_options.clone(),
          context_options: dependency.options().clone(),
          type_prefix: dependency.type_prefix(),
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
          format!("{specifier} (ignored)"),
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
    mut context_module_options: ContextModuleOptions,
    dependencies: &mut [BoxDependency],
  ) -> Result<Option<ModuleFactoryResult>> {
    let context_options = &context_module_options.context_options;
    let after_resolve_data = AfterResolveData {
      resource: context_module_options.resource.clone(),
      context: context_options.context.clone(),
      request: context_options.request.clone(),
      reg_exp: context_options.reg_exp.clone(),
      recursive: context_options.recursive,
      critical: true,
    };

    match self
      .plugin_driver
      .context_module_factory_hooks
      .after_resolve
      .call(AfterResolveResult::Data(Box::new(after_resolve_data)))
      .await?
    {
      AfterResolveResult::Ignored => Ok(Some(ModuleFactoryResult::default())),
      AfterResolveResult::Data(result) => {
        context_module_options.resource = result.resource;
        context_module_options.context_options.context = result.context;
        context_module_options.context_options.reg_exp = result.reg_exp;
        context_module_options.context_options.recursive = result.recursive;

        let dependency = dependencies[0]
          .as_context_dependency_mut()
          .expect("should be context dependency");
        if !result.critical {
          *dependency.critical_mut() = None;
        }

        let module = ContextModule::new(
          context_module_options.clone(),
          self.resolver_factory.clone(),
        );

        Ok(Some(ModuleFactoryResult::new_with_module(Box::new(module))))
      }
    }
  }
}
