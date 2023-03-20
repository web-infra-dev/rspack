use std::sync::Arc;

use rspack_error::{IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use tracing::instrument;

use crate::{
  cache::Cache, resolve, BoxModule, ContextModule, ContextModuleOptions, MissingModule,
  ModuleDependency, ModuleExt, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult,
  ModuleIdentifier, RawModule, ResolveArgs, ResolveError, ResolveResult, SharedPluginDriver,
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
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
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

  pub async fn resolve(
    &self,
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    let mut file_dependencies = Default::default();
    let mut missing_dependencies = Default::default();
    let context_dependencies = Default::default();
    let specifier = data.dependency.request();
    let resolve_args = ResolveArgs {
      context: data.context.clone(),
      importer: None,
      specifier,
      dependency_type: data.dependency.dependency_type(),
      dependency_category: data.dependency.category(),
      span: data.dependency.span().cloned(),
      resolve_options: data.resolve_options,
      resolve_to_context: true,
      file_dependencies: &mut file_dependencies,
      missing_dependencies: &mut missing_dependencies,
    };
    let plugin_driver = &self.plugin_driver;
    let resource_data = self
      .cache
      .resolve_module_occasion
      .use_cache(resolve_args, |args| resolve(args, plugin_driver))
      .await;
    let module = match resource_data {
      Ok(ResolveResult::Resource(resource)) => {
        let uri = resource.join().display().to_string();

        Box::new(ContextModule::new(ContextModuleOptions {
          resource: uri,
          resource_query: resource.query,
          resource_fragment: resource.fragment,
          context_options: data
            .dependency
            .options()
            .expect("should has options")
            .clone(),
        })) as BoxModule
      }
      Ok(ResolveResult::Ignored) => {
        let ident = format!(
          "{}/{}",
          data.context.expect("should have context"),
          specifier
        );
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
        let ident = format!("{}{specifier}", data.context.expect("should have context"));
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
      }
      .with_empty_diagnostic(),
    )
  }
}
