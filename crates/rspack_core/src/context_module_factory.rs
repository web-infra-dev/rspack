use std::sync::Arc;

use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use tracing::instrument;

use crate::{
  cache::Cache, resolve, BoxModule, ContextModule, ContextModuleOptions, MissingModule, ModuleExt,
  ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, ModuleIdentifier,
  NormalModuleBeforeResolveArgs, RawModule, ResolveArgs, ResolveError, ResolveResult,
  SharedPluginDriver,
};

pub struct ContextModuleFactory {
  plugin_driver: SharedPluginDriver,
  cache: Arc<Cache>,
}

#[async_trait::async_trait]
impl ModuleFactory for ContextModuleFactory {
  #[instrument(name = "context_module_factory:create", skip_all)]
  async fn create(
    mut self,
    mut data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    if let Ok(Some(before_resolve_result)) = self.before_resolve(&mut data).await {
      return Ok(before_resolve_result);
    }
    Ok(self.resolve(data).await?)
  }
}

impl ContextModuleFactory {
  pub fn new(plugin_driver: SharedPluginDriver, cache: Arc<Cache>) -> Self {
    Self {
      plugin_driver,
      cache,
    }
  }

  pub async fn before_resolve(
    &mut self,
    data: &mut ModuleFactoryCreateData,
  ) -> Result<Option<TWithDiagnosticArray<ModuleFactoryResult>>> {
    let dependency = data
      .dependency
      .as_module_dependency_mut()
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
      let specifier = dependency.request();
      let ident = format!("{}{specifier}", data.context);

      let module_identifier = ModuleIdentifier::from(format!("missing|{ident}"));

      let missing_module = MissingModule::new(
        module_identifier,
        format!("{ident} (missing)"),
        format!("Failed to resolve {specifier}"),
      )
      .boxed();
      return Ok(Some(
        ModuleFactoryResult::new(missing_module).with_empty_diagnostic(),
      ));
    }
    data.context = before_resolve_args.context.into();
    dependency.set_request(before_resolve_args.request);
    Ok(None)
  }

  pub async fn resolve(
    &self,
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    let dependency = data
      .dependency
      .as_module_dependency()
      .expect("should be module dependency");
    let factory_meta = Default::default();
    let mut file_dependencies = Default::default();
    let mut missing_dependencies = Default::default();
    let context_dependencies = Default::default();
    let specifier = dependency.request();
    let resolve_args = ResolveArgs {
      context: data.context.clone(),
      importer: None,
      specifier,
      dependency_type: dependency.dependency_type(),
      dependency_category: dependency.category(),
      span: dependency.span().cloned(),
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
          resource: resource.path.to_string_lossy().to_string(),
          resource_query: resource.query,
          resource_fragment: resource.fragment,
          resolve_options: data.resolve_options,
          context_options: dependency.options().expect("should has options").clone(),
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

        return Ok(ModuleFactoryResult::new(raw_module).with_empty_diagnostic());
      }
      Err(ResolveError(runtime_error, internal_error)) => {
        let ident = format!("{}{specifier}", data.context);
        let module_identifier = ModuleIdentifier::from(format!("missing|{ident}"));

        let missing_module = MissingModule::new(
          module_identifier,
          format!("{ident} (missing)"),
          runtime_error,
        )
        .boxed();

        return Ok(ModuleFactoryResult::new(missing_module).with_diagnostic(internal_error.into()));
      }
    };

    Ok(
      ModuleFactoryResult {
        module,
        file_dependencies,
        missing_dependencies,
        context_dependencies,
        factory_meta,
        from_cache,
      }
      .with_empty_diagnostic(),
    )
  }
}
