use std::sync::Arc;

use once_cell::sync::Lazy;
use rspack_core::{
  ApplyContext, BoxModule, Compilation, CompilationParams, CompilerOptions, DependencyType,
  ModuleFactory, ModuleFactoryCreateData, NormalModuleCreateData, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook, AsyncSeries2, AsyncSeries3};
use rspack_regex::RspackRegex;
use tokio::sync::Mutex;

use crate::{
  backend::Backend, factory::LazyCompilationDependencyFactory, module::LazyCompilationProxyModule,
};

static WEBPACK_DEV_SERVER_CLIENT_RE: Lazy<RspackRegex> = Lazy::new(|| {
  RspackRegex::new(
    r#"(webpack|rspack)[/\\]hot[/\\]|(webpack|rspack)-dev-server[/\\]client|(webpack|rspack)-hot-middleware[/\\]client"#,
  )
  .expect("should compile regex")
});

#[plugin]
#[derive(Debug)]
pub struct LazyCompilationPlugin<T: Backend> {
  backend: Mutex<T>,
  entries: bool, // enable for entries
  imports: bool, // enable for imports
  test: Option<RspackRegex>,
  cacheable: bool,
}

impl<T: Backend> LazyCompilationPlugin<T> {
  pub fn new(
    cacheable: bool,
    backend: T,
    test: Option<RspackRegex>,
    entries: bool,
    imports: bool,
  ) -> Self {
    Self::new_inner(Mutex::new(backend), entries, imports, test, cacheable)
  }

  fn check_test(&self, module: &BoxModule) -> bool {
    if let Some(test) = &self.test {
      test.test(&module.name_for_condition().unwrap_or("".into()))
    } else {
      true
    }
  }
}

#[plugin_hook(AsyncSeries2<Compilation, CompilationParams> for LazyCompilationPlugin <T: Backend>)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::LazyImport,
    Arc::new(LazyCompilationDependencyFactory::new(
      params.normal_module_factory.clone(),
    )) as Arc<dyn ModuleFactory>,
  );

  Ok(())
}

#[plugin_hook(AsyncSeries3<ModuleFactoryCreateData, NormalModuleCreateData, BoxModule> for LazyCompilationPlugin<T: Backend>)]
async fn normal_module_factory_module(
  &self,
  module_factory_create_data: &mut ModuleFactoryCreateData,
  create_data: &mut NormalModuleCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  if let Some(query) = &create_data.resource_resolve_data.resource_query
    && query.contains("lazy-compilation-proxy-dep")
  {
    let remaining_query = query.clone().replace("lazy-compilation-proxy-dep", "");

    create_data.resource_resolve_data.resource_query =
      if remaining_query.is_empty() || remaining_query == "?" {
        None
      } else {
        Some(remaining_query)
      };

    return Ok(());
  }

  let dep_type = module_factory_create_data.dependency.dependency_type();

  let is_imports = matches!(
    dep_type,
    DependencyType::DynamicImport
      | DependencyType::DynamicImportEager
      | DependencyType::ContextElement
  );
  let is_entries = matches!(dep_type, DependencyType::Entry);

  #[allow(clippy::if_same_then_else)]
  if matches!(
    dep_type,
    DependencyType::ModuleHotAccept
      | DependencyType::ModuleHotDecline
      | DependencyType::ImportMetaHotAccept
      | DependencyType::ImportMetaHotDecline
  ) {
    // TODO: we cannot access module graph at this stage
    // if hmr point to a module that is already been dyn imported
    // eg: import('./foo'); module.hot.accept('./foo')
    // however we cannot access module graph at this time, so we cannot
    // detect this case easily
    return Ok(());
  } else if !is_entries && !is_imports {
    return Ok(());
  }

  if !self.entries && is_entries {
    return Ok(());
  }
  if !self.imports && is_imports {
    return Ok(());
  }

  if WEBPACK_DEV_SERVER_CLIENT_RE.test(&create_data.resource_resolve_data.resource)
    || !self.check_test(module)
  {
    return Ok(());
  }

  let mut backend = self.backend.lock().await;
  let module_identifier = module.identifier();
  let info = backend
    .module(
      module_identifier,
      create_data.resource_resolve_data.resource.clone(),
    )
    .await?;

  *module = Box::new(LazyCompilationProxyModule::new(
    module_identifier,
    module_factory_create_data.clone(),
    create_data.resource_resolve_data.resource.clone(),
    self.cacheable,
    info.active,
    info.data,
    info.client,
  ));

  Ok(())
}

#[async_trait::async_trait]
impl<T: Backend + 'static> Plugin for LazyCompilationPlugin<T> {
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
      .normal_module_factory_hooks
      .module
      .tap(normal_module_factory_module::new(self));
    Ok(())
  }
}
