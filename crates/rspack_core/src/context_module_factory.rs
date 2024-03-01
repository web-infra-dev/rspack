use std::sync::Arc;

use rspack_error::Result;
use tracing::instrument;

use crate::{
  cache::Cache, resolve, BoxModule, ContextModule, ContextModuleOptions, ModuleExt, ModuleFactory,
  ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier, NormalModuleAfterResolveArgs,
  NormalModuleBeforeResolveArgs, PluginNormalModuleFactoryAfterResolveOutput, RawModule,
  ResolveArgs, ResolveResult, SharedPluginDriver,
};

#[derive(Debug)]
pub struct ContextModuleFactory {
  plugin_driver: SharedPluginDriver,
  cache: Arc<Cache>,
}

#[async_trait::async_trait]
impl ModuleFactory for ContextModuleFactory {
  #[instrument(name = "context_module_factory:create", skip_all)]
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    if let Ok(Some(before_resolve_result)) = self.before_resolve(data).await {
      return Ok(before_resolve_result);
    }

    let factorize_result = self.resolve(data).await?;

    if let Some(false) = self.after_resolve(data, &factorize_result).await? {
      return Ok(ModuleFactoryResult::default());
    }

    Ok(factorize_result)
  }
}

impl ContextModuleFactory {
  pub fn new(plugin_driver: SharedPluginDriver, cache: Arc<Cache>) -> Self {
    Self {
      plugin_driver,
      cache,
    }
  }

  async fn before_resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<Option<ModuleFactoryResult>> {
    let dependency = data
      .dependency
      .as_context_dependency_mut()
      .expect("should be module dependency");
    let mut before_resolve_args = NormalModuleBeforeResolveArgs {
      request: dependency.request().to_string(),
      context: data.context.to_string(),
    };
    if let Ok(Some(false)) = self
      .plugin_driver
      .context_module_before_resolve(&mut before_resolve_args)
      .await
    {
      // ignored
      // See https://github.com/webpack/webpack/blob/6be4065ade1e252c1d8dcba4af0f43e32af1bdc1/lib/ContextModuleFactory.js#L115
      return Ok(Some(ModuleFactoryResult::default()));
    }
    data.context = before_resolve_args.context.into();
    dependency.set_request(before_resolve_args.request);
    Ok(None)
  }

  async fn resolve(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let dependency = data
      .dependency
      .as_context_dependency()
      .expect("should be context dependency");
    let factory_meta = Default::default();
    let mut file_dependencies = Default::default();
    let mut missing_dependencies = Default::default();
    // let context_dependencies = Default::default();
    let request = dependency.request();
    let (loader_request, specifier) = match request.rfind('!') {
      Some(idx) => request.split_at(idx + 1),
      None => ("", request),
    };

    let resolve_args = ResolveArgs {
      context: data.context.clone(),
      importer: None,
      issuer: data.issuer.as_deref(),
      specifier,
      dependency_type: dependency.dependency_type(),
      dependency_category: dependency.category(),
      span: dependency.span(),
      resolve_options: data.resolve_options.clone(),
      resolve_to_context: true,
      optional: false,
      file_dependencies: &mut file_dependencies,
      missing_dependencies: &mut missing_dependencies,
    };
    let plugin_driver = &self.plugin_driver;

    let (resource_data, from_cache) = match self
      .cache
      .resolve_module_occasion
      .use_cache(resolve_args, |args| resolve(args, plugin_driver))
      .await
    {
      Ok(result) => result,
      Err(err) => (Err(err), false),
    };

    let module = match resource_data {
      Ok(ResolveResult::Resource(resource)) => Box::new(ContextModule::new(
        ContextModuleOptions {
          addon: loader_request.to_string(),
          resource: resource.path.to_string_lossy().to_string(),
          resource_query: resource.query,
          resource_fragment: resource.fragment,
          resolve_options: data.resolve_options.clone(),
          context_options: dependency.options().clone(),
        },
        plugin_driver.resolver_factory.clone(),
      )) as BoxModule,
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
        return Ok(ModuleFactoryResult::new_with_module(raw_module));
      }
      Err(err) => {
        return Err(err);
      }
    };

    data.add_file_dependencies(file_dependencies);
    data.add_missing_dependencies(missing_dependencies);
    // data.add_context_dependencies(context_dependencies);

    Ok(ModuleFactoryResult {
      module: Some(module),
      factory_meta,
      from_cache,
    })
  }

  async fn after_resolve(
    &self,
    data: &mut ModuleFactoryCreateData,
    factory_result: &ModuleFactoryResult,
  ) -> PluginNormalModuleFactoryAfterResolveOutput {
    let dependency = data
      .dependency
      .as_context_dependency()
      .expect("should be module dependency");

    self
      .plugin_driver
      .context_module_after_resolve(&mut NormalModuleAfterResolveArgs {
        request: dependency.request(),
        context: data.context.as_ref(),
        file_dependencies: &data.file_dependencies,
        context_dependencies: &data.context_dependencies,
        missing_dependencies: &data.missing_dependencies,
        diagnostics: &mut data.diagnostics,
        factory_meta: &factory_result.factory_meta,
      })
      .await
  }
}
