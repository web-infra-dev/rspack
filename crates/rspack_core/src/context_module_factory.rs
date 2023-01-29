use std::sync::Arc;

use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray};
use tracing::instrument;

use crate::{
  cache::Cache, resolve, BoxModule, CompilerOptions, ContextModule, ContextModuleOptions,
  ModuleDependency, ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, ResolveArgs,
  ResolveResult, SharedPluginDriver,
};

pub struct ContextModuleFactory {
  compiler_options: Arc<CompilerOptions>,
  plugin_driver: SharedPluginDriver,
  cache: Arc<Cache>,
  diagnostics: Vec<Diagnostic>,
}

#[async_trait::async_trait]
impl ModuleFactory for ContextModuleFactory {
  #[instrument(name = "context_module_factory:create", skip_all)]
  async fn create(
    mut self,
    data: ModuleFactoryCreateData,
  ) -> Result<TWithDiagnosticArray<ModuleFactoryResult>> {
    Ok((self.resolve(data).await?).with_diagnostic(self.diagnostics))
  }
}

impl ContextModuleFactory {
  pub fn new(
    compiler_options: Arc<CompilerOptions>,
    plugin_driver: SharedPluginDriver,
    cache: Arc<Cache>,
  ) -> Self {
    Self {
      compiler_options,
      plugin_driver,
      cache,
      diagnostics: Default::default(),
    }
  }

  pub async fn resolve(&self, data: ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    let mut file_dependencies = Default::default();
    let mut missing_dependencies = Default::default();
    let context_dependencies = Default::default();
    let dependency = data.dependencies[0].clone();
    let resolve_args = ResolveArgs {
      context: data.context,
      importer: None,
      specifier: dependency.request(),
      dependency_type: dependency.dependency_type(),
      dependency_category: dependency.category(),
      span: dependency.span().cloned(),
      compiler_options: self.compiler_options.as_ref(),
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
      Ok(ResolveResult::Info(info)) => {
        let uri = info.join();

        Box::new(ContextModule::new(ContextModuleOptions {
          resource: uri,
          resource_query: (!info.query.is_empty()).then_some(info.query),
          resource_fragment: (!info.fragment.is_empty()).then_some(info.fragment),
          context_options: dependency.options().expect("should has options").clone(),
        })) as BoxModule
      }
      _ => {
        todo!()
      }
    };

    Ok(ModuleFactoryResult {
      module,
      file_dependencies,
      missing_dependencies,
      context_dependencies,
    })
  }
}
